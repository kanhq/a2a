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
  cmd.arg(action.command);
  cmd.args(action.args.as_ref().unwrap_or(&vec![]));

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
  let mimetype = action
    .override_result_mimetype
    .unwrap_or("text/plain".to_string());

  bytes_to_json(output.stdout.into(), mimetype, None)
}
