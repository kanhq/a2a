use clap::Parser;

use std::{path::PathBuf, sync::OnceLock};

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct AppConf {
  /// config dir, will be used to find the config file with any extension of .toml, .yaml, .json, .ini
  /// all the config files will be merged
  /// directory will be scanned recursively
  #[clap(short, long, default_value = "conf")]
  pub conf_dir: PathBuf,

  /// the script file to be executed
  pub file: String,
}

pub fn app_conf() -> &'static AppConf {
  static APP_CONF: OnceLock<AppConf> = OnceLock::new();
  APP_CONF.get_or_init(|| {
    let mut app = AppConf::parse();

    app.conf_dir = app.conf_dir.canonicalize().unwrap_or_default();

    app
  })
}
