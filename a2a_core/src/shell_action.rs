use std::{arch, os};

use a2a_tojson::bytes_to_json;
use a2a_types::{ShellAction, ShellActionResult};
use anyhow::Result;

pub async fn do_action(action: ShellAction) -> Result<ShellActionResult> {
  let program = if cfg!(windows) { "cmd.exe" } else { "sh" };
  let mut cmd = tokio::process::Command::new(program);
  cmd.kill_on_drop(true);
  cmd.arg(if cfg!(windows) { "/C" } else { "-c" });
  cmd.arg(&action.command);
  if let Some(args) = action.args {
    cmd.args(args);
  }
  if let Some(env) = action.env {
    cmd.envs(env);
  }
  if let Some(cwd) = action.cwd {
    cmd.current_dir(cwd);
  }
  let output = cmd.output().await?;
  let mimetype = action
    .override_result_mimetype
    .unwrap_or("text/plain".to_string());

  bytes_to_json(output.stdout.into(), mimetype, None)
}
