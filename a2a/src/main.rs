use anyhow::Result;
use app_conf::Commands;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

mod app_conf;
mod coder;
mod config_loader;
mod run;

fn setup_logging() {
  let filter = EnvFilter::from_default_env();
  tracing_subscriber::fmt()
    .with_env_filter(filter)
    .with_level(true)
    .with_writer(std::io::stderr)
    .init();
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
  dotenvy::dotenv().unwrap_or_default();

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
  }

  Ok(())
}
