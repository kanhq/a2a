use std::{
  path::PathBuf,
  sync::{Arc, RwLock},
};

use crate::{app_conf::Serve, config_loader::load_conf_dir};
use a2a_types::Value;
use anyhow::Result;
use axum::{
  http::HeaderName,
  routing::{get, post},
};
use scheduler::ScheduleAdminSender;
use tokio_util::sync::CancellationToken;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

mod admin;
mod api;
mod mcp;
mod run;
mod scheduler;
mod workspace;
mod writer;

use mcp::{A2AMcp, McpState};
pub use scheduler::test_scheduler;

struct AppState {
  // maybe change at runtime
  pub conf: Arc<RwLock<Value>>,
  pub root_path: PathBuf,
  pub api_root_path: PathBuf,
  pub scheduler_admin: ScheduleAdminSender,
  pub mcp_state: McpState,
}

pub(crate) async fn execute(arg: &Serve) -> Result<()> {
  let conf = load_conf_dir(&arg.conf_dir_path)?;

  let ct = tokio_util::sync::CancellationToken::new();

  let scheduler_admin = tokio::spawn(scheduler::start(
    arg.api_root_path.clone(),
    arg.scheduler_path.clone(),
    conf.clone(),
  ))
  .await??;
  let mcp_path = arg.mcp_path.as_ref().map_or("/mcp", |p| p.as_str());

  let mcp_sse_config = mcp::SseServerConfig {
    sse_path: format!("{}/sse", mcp_path),
    post_path: format!("{}/messages", mcp_path),
    ct: ct.clone(),
  };
  show_runtime_info(arg, &mcp_sse_config);
  let (mcp_server, mcp_router, mcp_state) = mcp::SseServer::new(mcp_sse_config);

  let state = Arc::new(AppState {
    conf: Arc::new(RwLock::new(conf)),
    root_path: arg.root_path.clone(),
    api_root_path: arg.api_root_path.clone(),
    scheduler_admin,
    mcp_state,
  });

  let state_for_mcp = state.clone();
  mcp_server.with_service(move || A2AMcp::new(state_for_mcp.clone()));

  let admin_path = arg.admin_path.as_ref().map_or("/admin", |p| p.as_str());
  let cors = CorsLayer::new()
    .allow_methods(Any)
    .allow_headers(vec![
      HeaderName::from_static("content-type"),
      HeaderName::from_static("authorization"),
    ])
    .allow_origin(Any);
  let app = axum::Router::new()
    .fallback_service(tower_http::services::ServeDir::new(
      arg.html_root_path.clone(),
    ))
    .route(
      "/api/{*file}",
      post(api::post_handler).get(api::get_handler),
    )
    .route("/code", post(writer::coder_handle))
    .route("/code/prompt", get(writer::system_prompt_handle))
    .route("/run/json", post(run::post_json_handle))
    .route("/run/form", post(run::post_form_handle))
    .route(admin_path, post(admin::post_handler))
    .merge(mcp_router)
    .with_state(state)
    .layer(cors);

  let listener = tokio::net::TcpListener::bind(&arg.listen).await?;

  axum::serve(listener, app)
    .with_graceful_shutdown(shutdown_signal(ct))
    .await?;

  Ok(())
}

async fn shutdown_signal(ct: CancellationToken) {
  let ctrl_c = async {
    tokio::signal::ctrl_c()
      .await
      .expect("failed to install Ctrl+C handler");
  };

  #[cfg(unix)]
  let terminate = async {
    tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
      .expect("failed to install signal handler")
      .recv()
      .await;
  };

  #[cfg(not(unix))]
  let terminate = std::future::pending::<()>();

  tokio::select! {
      _ = ctrl_c => {},
      _ = terminate => {},
  }
  ct.cancel();
  // Wait for the cancellation token to be cancelled
  ct.cancelled().await;
}

fn show_runtime_info(arg: &Serve, mcp_arg: &mcp::SseServerConfig) {
  let workspace_dir = arg.root_path.display().to_string();
  info!(workspace_dir, "A2A starting server");
  let url = format!("http://{}", local_ip(arg));
  info!(url, "A2A listening on");
  let admin_path = arg.admin_path.as_ref().map_or("/admin", |p| p.as_str());
  let routers = vec![
    ("/api/{*file}", "API endpoint"),
    ("/code", "Write code endpoint"),
    ("/code/prompt", "Get system prompt endpoint"),
    ("/run/json", "Run code from JSON request endpoint"),
    ("/run/form", "Run code from form request endpoint"),
    (mcp_arg.sse_path.as_ref(), "MCP SSE endpoint"),
    (admin_path, "Admin"),
  ];

  routers.iter().for_each(|(path, desc)| {
    let endpoint = format!("{}{}", url, path);
    info!("router {:<width$} {}", endpoint, desc, width = 40);
  });
}

fn local_ip(arg: &Serve) -> String {
  if arg.listen.starts_with("0.0.0.0") {
    let port = arg.listen.split(':').last().unwrap_or("30030");
    // enumerate all local IPs
    local_ip_address::local_ip()
      .map(|ip| format!("{}:{}", ip, port))
      .unwrap_or_else(|_| arg.listen.to_string())
  } else {
    return arg.listen.to_string();
  }
}
