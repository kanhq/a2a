use std::{
  io::Result,
  io::Write,
  path::{Path, PathBuf},
};

use tracing::info;

use crate::app_conf::InitWorkDir;

const ECHO_SCRIPT: &'static str = r#"
export async function main(config, params) {
  return {
    config,
    params
  };
}
"#;

const ECHO_SCHEDULER_SCRIPT: &'static str = r#"
[
  {
    "name": "echo",
    "crons": [
      "*/3 * * * * 1-5"
    ],
    "command": "echo.js",
    "env": {
      "a": "b"
    }
  },
  {
    "name": "echo",
    "crons": [
      "*/3 * * * * *"
    ],
    "command": "ls",
    "args": [
      "-la",
      "ddd"
    ],
    "cwd": "."
  }
]
"#;

const HTML_INDEX: &'static str = r#"
It's a2a server
"#;

fn join_path<P: AsRef<Path>>(base: P, children: &[&str]) -> PathBuf {
  let mut p = base.as_ref().to_path_buf();
  for child in children {
    p.push(child);
  }
  p
}

pub(crate) fn init_workdir(wd: &InitWorkDir) -> Result<()> {
  let sub_dirs = ["api", "conf", "html", "scheduler"];
  for sub_dir in sub_dirs {
    let dir = join_path(&wd.dir, &[sub_dir]);
    info!(?dir, "create sub dir");
    if !dir.exists() {
      std::fs::create_dir_all(&dir)?;
    }
  }

  // create a template config file
  {
    let config_template_path = join_path(&wd.dir, &["conf", "connection.template.yaml"]);
    let conf = a2a_core::default_connection();
    let file = std::fs::File::create(&config_template_path)?;
    serde_json::to_writer_pretty(file, &conf)?;
    info!(file=?config_template_path, "create config template");
  }

  // create a demo script
  {
    let script_path = join_path(&wd.dir, &["api", "echo.js"]);
    let mut file = std::fs::File::create(&script_path)?;
    file.write_all(ECHO_SCRIPT.as_bytes())?;
    info!(file=?script_path, "create demo script");
  }

  // create a demo scheduler script
  {
    let script_path = join_path(&wd.dir, &["scheduler", "echo.json"]);
    let mut file = std::fs::File::create(&script_path)?;
    file.write_all(ECHO_SCHEDULER_SCRIPT.as_bytes())?;
    info!(file=?script_path, "create demo scheduler script");
  }

  // create a demo html file
  {
    let html_path = join_path(&wd.dir, &["html", "index.html"]);
    let mut file = std::fs::File::create(&html_path)?;
    file.write_all(HTML_INDEX.as_bytes())?;
    info!(file = ?html_path, "create demo html file");
  }

  Ok(())
}
