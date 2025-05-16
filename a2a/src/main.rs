use anyhow::Result;
use app_conf::{default_work_dir, Commands};
use tracing::{info, warn};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt::writer::MakeWriterExt, EnvFilter};

mod app_conf;
mod coder;
mod config_loader;
mod init;
mod run;
mod serve;

fn setup_logging() {
  let filter = EnvFilter::from_default_env();
  let log_writer = std::io::stderr;
  let mut ansi_color = true;
  if let Some(log_base_dir) = std::env::var("A2A_LOG_DIR").ok().filter(|s| !s.is_empty()) {
    std::fs::create_dir_all(&log_base_dir).unwrap_or_default();
    let log_file = RollingFileAppender::builder()
      .rotation(Rotation::DAILY)
      .filename_prefix("a2a")
      .filename_suffix("log")
      .max_log_files(20)
      .build(&log_base_dir)
      .expect("initializing rolling file ");
    log_writer.and(log_file);
    ansi_color = false;
  }

  tracing_subscriber::fmt()
    .with_env_filter(filter)
    .with_ansi(ansi_color)
    .with_level(true)
    .with_writer(log_writer)
    .init();
}

fn setup_env() {
  let user_env = default_work_dir().join("a2a.env");
  // try to load user specified env file
  dotenvy::from_path_override(user_env).unwrap_or_default();
  // try to load .env file from current directory
  dotenvy::dotenv_override().unwrap_or_default();
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
  setup_env();
  setup_logging();

  let app = app_conf::app_conf();

  match app.command {
    Commands::Coder(ref coder) => {
      coder::execute(coder).await?;
    }
    Commands::Run(ref runner) => match run::execute(runner).await {
      Ok(val) => info!("run {}", serde_json::to_string_pretty(&val)?),
      Err(err) => warn!("run {}", err),
    },
    Commands::Serve(ref serve) => {
      serve::execute(serve).await?;
    }
    Commands::Scheduler(ref scheduler) => {
      let _ = serve::test_scheduler(scheduler).await?;
    }
    Commands::Init(ref init) => {
      init::init_workdir(init)?;
    }
  }

  Ok(())
}
