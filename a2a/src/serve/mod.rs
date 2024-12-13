use std::{path::PathBuf, sync::Arc};

use crate::{app_conf::Serve, config_loader::load_conf_dir};
use a2a_types::Value;
use anyhow::Result;
use axum::{http::HeaderName, routing::post};
use tower_http::cors::{Any, CorsLayer};
use scheduler::ScheduleAdminSender;
use tracing::info;

mod admin;
mod api;
mod run;
mod scheduler;
mod writer;

pub use scheduler::test_scheduler;

struct AppState {
  pub conf: Value,
  pub api_root_path: PathBuf,
  pub scheduler_admin: ScheduleAdminSender,
}

pub(crate) async fn execute(arg: &Serve) -> Result<()> {
  let conf = load_conf_dir(&arg.conf_dir_path)?;
  let scheduler_admin = tokio::spawn(scheduler::start(
    arg.api_root_path.clone(),
    arg.scheduler_path.clone(),
    conf.clone(),
  ))
  .await??;
  let state = Arc::new(AppState {
    conf,
    api_root_path: arg.api_root_path.clone(),
    scheduler_admin,
  });
  let admin_path = arg.admin_path.as_ref().map_or("/admin", |p| p.as_str());
  let cors = CorsLayer::new()
    .allow_methods(Any)
    .allow_headers(vec![HeaderName::from_static("content-type"), HeaderName::from_static("authorization")])
    .allow_origin(Any);
  let app = axum::Router::new()
    .nest_service(
      "/",
      tower_http::services::ServeDir::new(arg.html_root_path.clone()),
    )
    .route("/api/*file", post(api::post_handler).get(api::get_handler))
    //.route("/code", post(writer::coder_handle))
    .route("/run/json", post(run::post_json_handle))
    .route("/run/form", post(run::post_form_handle))
    .route(admin_path, post(admin::post_handler))
    .with_state(state)
    .layer(cors)
    ;

  let listener = tokio::net::TcpListener::bind(&arg.listen).await?;

  info!("listening on {}", arg.listen);
  axum::serve(listener, app).await?;

  Ok(())
}
