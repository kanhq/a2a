use std::io::Write;

use a2a_tojson::bytes_to_json;
use a2a_types::{ShellAction, ShellActionResult};
use anyhow::Result;

pub async fn do_action(action: ShellAction) -> Result<ShellActionResult> {
  let program = if cfg!(windows) { "cmd.exe" } else { "sh" };
  let mut cmd = tokio::process::Command::new(program);
  cmd.kill_on_drop(true);

  if cfg!(windows) {
    cmd.arg("/C");
  } else {
    cmd.arg("-c");
  }
  let command = command_for_builtin(&action)?;
  cmd.arg(command);
  if let Some((arg_as_file, args)) = action.arg_as_file.as_ref().zip(action.args.as_ref()) {
    let mut fp = std::fs::File::create(arg_as_file)
      .map_err(|e| anyhow::anyhow!("Failed to create args file '{}': {}", arg_as_file, e))?;

    args.iter().try_for_each(|arg| {
      fp.write_all(arg.as_bytes())
        .map_err(|e| anyhow::anyhow!("Failed to write to args file '{}': {}", arg_as_file, e))
    })?;
    cmd.arg(arg_as_file);
  } else {
    cmd.args(action.args.as_ref().unwrap_or(&vec![]));
  }

  if let Some(env) = action.env {
    cmd.envs(env);
  }
  if let Some(cwd) = action.cwd {
    cmd.current_dir(cwd);
  }
  let output = cmd
    .output()
    .await
    .map_err(|e| anyhow::anyhow!("Failed to execute command '{:#?}': {}", cmd, e))?;

  if !output.status.success() {
    return Err(anyhow::anyhow!(
      "Command '{:#?}' failed with status: {}",
      cmd,
      output.status
    ));
  }
  let mimetype = action
    .override_result_mimetype
    .unwrap_or("text/plain".to_string());

  let result = output.stdout.into();
  let result = bytes_to_json(result, mimetype, None);
  // // Clean up the args file if it was created
  // if let Some(args_as_file) = action.arg_as_file {
  //   std::fs::remove_file(args_as_file).unwrap_or_default();
  // }
  result
}

fn command_for_builtin(action: &ShellAction) -> Result<&str> {
  let cmd = action.command.as_str();
  if action.command.eq_ignore_ascii_case("open") {
    let file_name = action
      .arg_as_file
      .as_ref()
      .or(action.args.as_ref().and_then(|args| args.first()))
      .map(|s| s.as_str())
      .ok_or(anyhow::anyhow!("No file name provided for 'open' command"))?;

    if let Some(ext) = file_name.split('.').last() {
      if ext.eq_ignore_ascii_case("py") {
        return Ok("python");
      } else if ext.eq_ignore_ascii_case("js") {
        return Ok("node");
      } else if ext.eq_ignore_ascii_case("sh") {
        return Ok("bash");
      } else if ext.eq_ignore_ascii_case("rb") {
        return Ok("ruby");
      } else if ext.eq_ignore_ascii_case("pl") {
        return Ok("perl");
      }
    }
    if cfg!(windows) {
      Ok("start")
    } else if cfg!(target_os = "macos") {
      Ok("open")
    } else if cfg!(target_os = "linux") {
      Ok("xdg-open")
    } else {
      Ok(cmd)
    }
  } else {
    Ok(cmd)
  }
}
