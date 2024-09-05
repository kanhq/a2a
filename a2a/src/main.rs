use anyhow::Result;
use tracing_subscriber::EnvFilter;

mod app_conf;
mod config_loader;
mod run;

fn setup_logging() {
  let filter = EnvFilter::from_default_env()
    .add_directive("a2a=trace".parse().unwrap())
    .add_directive("a2a_execute=trace".parse().unwrap())
    .add_directive("a2a_render=trace".parse().unwrap());
  tracing_subscriber::fmt()
    .with_env_filter(filter)
    .with_level(true)
    .init();
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
  setup_logging();

  let app = app_conf::app_conf();
  let config_pattern = app
    .conf_dir
    .join("**/*.{json,yaml,ini,env}")
    .to_str()
    .map(str::to_string)
    .ok_or(anyhow::anyhow!("invalid config pattern"))?;

  let conf = config_loader::load_configs(&config_pattern)?;

  run::execute_js(&app.file, conf).await?;

  Ok(())
}
