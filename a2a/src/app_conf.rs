use clap::{ArgAction, Args, Parser, Subcommand};

use std::{path::PathBuf, sync::OnceLock};

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
}

#[derive(Debug, Args)]
pub struct Runner {
  /// config dir, will be used to find the config file with any extension of .toml, .yaml, .json, .ini
  /// all the config files will be merged
  /// directory will be scanned recursively
  #[clap(short, long, default_value = "conf")]
  pub conf_dir: PathBuf,

  /// the script file to be executed
  pub file: String,

  /// the working directory to run the script
  /// default is the current directory when '-p' is not set or the parent directory of the script file when '-p' is set
  #[clap(long, short)]
  pub work_dir: Option<String>,

  /// is project mode, script file will be executed in the parent directory of the script file
  #[clap(long, short, action=ArgAction::SetTrue)]
  pub project_mode: Option<bool>,

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

  /// when run, the config to be used
  #[clap(short, long)]
  pub conf_dir: Option<String>,

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
  /// - html: all files under this directory will be served as static files
  /// - api: all js files under this directory will be served as api
  /// - conf: all config files under this directory will be loaded
  /// - scheduler: the scheduler config file, task will be scheduled execution
  #[clap(short, long, default_value = ".")]
  pub root: Option<String>,

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
        if runner.project_mode.unwrap_or(false) {
          let script_path = PathBuf::from(&runner.file);
          runner.work_dir = script_path
            .parent()
            .map(|p| p.to_string_lossy().to_string());
          runner.file = script_path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default();
        }
      }
      Commands::Serve(ref mut serve) => {
        serve.root_path = serve
          .root
          .as_ref()
          .ok_or(anyhow::anyhow!("root is required"))
          .and_then(|p| PathBuf::from(p).canonicalize().map_err(|e| e.into()))
          .unwrap();

        serve.conf_dir_path = serve.root_path.join("conf");
        serve.api_root_path = serve.root_path.join("api");
        serve.html_root_path = serve.root_path.join("html");
        serve.scheduler_path = serve.root_path.join("scheduler");
      }
      Commands::Scheduler(ref mut scheduler) => {
        scheduler.config = scheduler.config.canonicalize().unwrap_or_default();
      }
    }
    app
  })
}
