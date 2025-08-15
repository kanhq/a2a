use clap::{ArgAction, Args, Parser, Subcommand};
use tracing::{debug, info};

use std::{
  fs,
  path::{Path, PathBuf},
  sync::OnceLock,
};

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct AppConf {
  #[command(subcommand)]
  pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
  /// write code by llm
  Coder(Coder),
  /// run the code
  Run(Runner),
  /// serve the code
  Serve(Serve),
  /// schedule the task
  Scheduler(Scheduler),
  /// initialize the work directory
  Init(InitWorkDir),
}

#[derive(Debug, Args)]
pub struct Runner {
  #[clap(skip)]
  pub conf_dir: PathBuf,

  /// the script file to be executed
  pub file: String,

  /// the working directory to run the script
  /// set to '.' to run in the current directory
  /// default is the [default_work_dir]
  #[clap(long, short)]
  pub work_dir: Option<String>,

  /// clean up script after run, will be executed after each run
  #[clap(long)]
  pub clean: Option<String>,
}

#[derive(Debug, Args)]
pub struct Coder {
  /// user prompt to write code, can be a file or a string
  pub user: String,

  /// name of the file to save the generated code, can have placeholder {provider} and {model},
  /// default is to `{user_prompt_name}.{provider}.{model}.js` when user prompt is a file, or stdout when user prompt is just some text
  #[clap(short, long)]
  pub out_file: Option<String>,

  /// system prompt to write code, default use embedded system prompt
  #[clap(short, long)]
  pub system: Option<String>,

  /// no system prompt, only use user prompt
  #[clap(long, action=ArgAction::SetTrue)]
  pub no_system: Option<bool>,

  /// llm model, can be multiple, with format of model[:provider]
  #[clap(short, long)]
  pub model: Vec<String>,

  /// llm models file, each line is a model, with format of model[:provider]
  #[clap(short = 'M', long)]
  pub models_file: Option<String>,

  /// llm base url
  #[clap(short, long, env = "OPENAI_BASE_URL")]
  pub base_url: String,

  /// llm api key
  #[clap(short, long, env = "OPENAI_API_KEY")]
  pub api_key: String,

  /// run the code after generation
  #[clap(short, long, action=ArgAction::SetTrue)]
  pub run: Option<bool>,

  /// when run, the work dir to be used
  #[clap(short, long)]
  pub work_dir: Option<String>,

  /// clean up script after run, will be executed after each run
  #[clap(long)]
  pub clean: Option<String>,

  #[clap(skip)]
  pub models: Vec<(String, String)>,
}

#[derive(Debug, Args)]
pub struct Serve {
  /// the address to listen
  #[clap(short, long, default_value = "127.0.0.1:30030")]
  pub listen: String,

  /// base dir of the server, there are some special sub directories
  #[clap(short, long)]
  pub work_dir: Option<String>,

  /// don't start ui when serve startup
  #[clap(long, default_value = "false")]
  pub no_ui: bool,

  /// the service path of admin api
  #[clap(long, default_value = "/admin")]
  pub admin_path: Option<String>,

  /// the service path of mcp
  #[clap(long, default_value = "/mcp")]
  pub mcp_path: Option<String>,

  #[clap(skip)]
  pub conf_dir_path: PathBuf,

  #[clap(skip)]
  pub api_root_path: PathBuf,

  #[clap(skip)]
  pub html_root_path: PathBuf,

  #[clap(skip)]
  pub scheduler_path: PathBuf,

  #[clap(skip)]
  pub root_path: PathBuf,
}

#[derive(Debug, Args)]
pub struct Scheduler {
  /// the config file of the scheduler
  #[clap(short, long)]
  pub config: PathBuf,
  /// the name of the task to be scheduled
  #[clap(short, long)]
  pub task: String,
  #[clap(short, long, default_value = "10")]
  pub num: Option<usize>,
  #[clap(short, long)]
  pub start: Option<String>,
}

#[derive(Debug, Args)]
pub struct InitWorkDir {
  /// the directory to be initialized
  #[clap(short, long)]
  pub work_dir: Option<String>,

  #[clap(skip)]
  pub root_path: PathBuf,
}

pub fn app_conf() -> &'static AppConf {
  static APP_CONF: OnceLock<AppConf> = OnceLock::new();
  APP_CONF.get_or_init(|| {
    let mut app = AppConf::parse();
    match app.command {
      Commands::Coder(ref mut coder) => {
        coder.models = coder
          .model
          .iter()
          .filter_map(|model| {
            model
              .rsplit_once(':')
              .map(|t| (t.1.to_string(), t.0.to_string()))
          })
          .collect();
        if let Some(file_name) = &coder.models_file {
          let file = std::fs::read_to_string(file_name).unwrap();
          coder.models.extend(
            file
              .lines()
              .filter_map(|line| {
                line
                  .rsplit_once(':')
                  .map(|t| (t.1.to_string(), t.0.to_string()))
              })
              .collect::<Vec<_>>(),
          );
        }
      }
      Commands::Run(ref mut runner) => {
        let work_dir = runner
          .work_dir
          .as_ref()
          .map(|p| PathBuf::from(p))
          .unwrap_or(default_work_dir())
          .canonicalize()
          .unwrap_or_default();
        debug!(?work_dir, "current work dir");
        runner.conf_dir = work_dir.join("conf");
      }
      Commands::Serve(ref mut serve) => {
        serve.root_path = serve
          .work_dir
          .as_ref()
          .map(|p| PathBuf::from(p))
          .unwrap_or(default_work_dir());

        info!(pwd = ?serve.root_path, "current work dir");
        fs::create_dir_all(&serve.root_path).unwrap_or_default();

        serve.root_path = serve.root_path.canonicalize().unwrap_or_default();

        serve.conf_dir_path = serve.root_path.join("conf");
        serve.api_root_path = serve.root_path.join("api");
        serve.html_root_path = serve.root_path.join("html");
        serve.scheduler_path = serve.root_path.join("scheduler");
      }
      Commands::Scheduler(ref mut scheduler) => {
        scheduler.config = scheduler.config.canonicalize().unwrap_or_default();
      }
      Commands::Init(ref mut init) => {
        init.root_path = init
          .work_dir
          .as_ref()
          .map(|p| PathBuf::from(p))
          .unwrap_or(default_work_dir());
        info!(pwd = ?init.root_path, "current work dir");
        fs::create_dir_all(&init.root_path).unwrap_or_default();
        init.root_path = init.root_path.canonicalize().unwrap_or_default();
      }
    }
    app
  })
}

pub(crate) fn join_path<P: AsRef<Path>>(base: P, children: &[&str]) -> PathBuf {
  let mut p = base.as_ref().to_path_buf();
  for child in children {
    p.push(child);
  }
  p
}

/// get the default work directory
/// - on windows, it will be APPDATA/a2a
/// - on linux/macos, it will be $HOME/.config/a2a if not root, or /etc/a2a if root
/// - if A2A_BASE_DIR is set, it will be used as the work directory
pub(crate) fn default_work_dir() -> PathBuf {
  if let Some(dir) = std::env::var("A2A_BASE_DIR").ok() {
    return PathBuf::from(dir);
  }
  match std::env::consts::OS {
    "windows" => std::env::var("APPDATA")
      .map(|s| PathBuf::from(s).join("a2a"))
      .unwrap_or_else(|_| PathBuf::from(".")),
    _ => std::env::var("HOME")
      .map(PathBuf::from)
      .map(|base| {
        if base.ends_with("/root") {
          PathBuf::from("/etc/a2a")
        } else {
          join_path(base, &[".config", "a2a"])
        }
      })
      .unwrap_or_else(|_| PathBuf::from(".")),
  }
}
