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

  /// clean up script after run, will be executed after each run
  #[clap(long)]
  pub clean: Option<String>,
}

#[derive(Debug, Args)]
pub struct Coder {
  /// user prompt to write code, can be a file or a string
  pub user: String,

  /// name of the file to save the generated code, can have placeholder {provider} and {model},
  /// default is to print to stdout
  #[clap(short, long)]
  pub file: Option<String>,

  /// system prompt to write code, default use embedded system prompt
  #[clap(short, long)]
  pub system: Option<String>,

  /// no system prompt, only use user prompt
  #[clap(long, action=ArgAction::SetTrue)]
  pub no_system: Option<bool>,

  /// llm model, can be multiple, with format of model[:provider]
  #[clap(short, long)]
  pub model: Vec<String>,

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

pub fn app_conf() -> &'static AppConf {
  static APP_CONF: OnceLock<AppConf> = OnceLock::new();
  APP_CONF.get_or_init(|| {
    let mut app = AppConf::parse();
    match app.command {
      Commands::Coder(ref mut coder) => {
        coder.models = coder
          .model
          .iter()
          .map(|model| {
            let mut parts = model.rsplitn(2, ':');
            let provider = parts.next().unwrap_or("openai");
            let model = parts.next().unwrap_or_default();
            (provider.to_string(), model.to_string())
          })
          .collect();
      }
      Commands::Run(ref mut runner) => {
        runner.conf_dir = runner.conf_dir.canonicalize().unwrap_or_default();
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
    }
    app
  })
}
