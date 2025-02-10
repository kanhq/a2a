use anyhow::Result;
use app_conf::Commands;
use tracing::{info, warn};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt::writer::MakeWriterExt, EnvFilter};

mod app_conf;
mod coder;
mod config_loader;
mod run;
mod serve;

fn setup_logging() {
  let filter = EnvFilter::from_default_env();
  let log_writer = std::io::stderr;
  if let Some(log_base_dir) = std::env::var("A2A_LOG_BASE_DIR")
    .ok()
    .filter(|s| !s.is_empty())
  {
    std::fs::create_dir_all(&log_base_dir).unwrap_or_default();
    let log_file = RollingFileAppender::builder()
      .rotation(Rotation::DAILY)
      .filename_prefix("a2a")
      .filename_suffix("log")
      .max_log_files(20)
      .build(&log_base_dir)
      .expect("initializing rolling file ");
    log_writer.and(log_file);
  }

  tracing_subscriber::fmt()
    .with_env_filter(filter)
    .with_ansi(false)
    .with_level(true)
    .with_writer(log_writer)
    .init();
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
  dotenvy::dotenv_override().unwrap_or_default();

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
  }

  Ok(())
}
