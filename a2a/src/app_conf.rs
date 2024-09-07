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
            let mut parts = model.splitn(2, ':');
            let model = parts.next().unwrap_or_default();
            let provider = parts.next().unwrap_or("openai");
            (provider.to_string(), model.to_string())
          })
          .collect();
      }
      Commands::Run(ref mut runner) => {
        runner.conf_dir = runner.conf_dir.canonicalize().unwrap_or_default();
      }
    }
    app
  })
}
