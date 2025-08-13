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
use rmcp::transport::{
  sse_server::SseServerConfig, streamable_http_server::session::local::LocalSessionManager,
  SseServer, StreamableHttpService,
};
use scheduler::ScheduleAdminSender;
use tokio_util::sync::CancellationToken;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};

mod admin;
mod api;
mod mcp;
mod run;
mod scheduler;
mod workspace;
mod writer;

use mcp::A2AMcp;
pub use scheduler::test_scheduler;

struct AppState {
  // maybe change at runtime
  pub conf: Arc<RwLock<Value>>,
  pub root_path: PathBuf,
  pub api_root_path: PathBuf,
  pub scheduler_admin: ScheduleAdminSender,
  //pub mcp_state: McpState,
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

  let mcp_sse_config = SseServerConfig {
    bind: arg.listen.parse()?,
    sse_keep_alive: None,
    sse_path: format!("{}/sse", mcp_path),
    post_path: format!("{}/messages", mcp_path),
    ct: ct.clone(),
  };
  show_runtime_info(arg, &mcp_sse_config);
  let (mcp_sse_server, mcp_sse_router) = SseServer::new(mcp_sse_config);

  let state = Arc::new(AppState {
    conf: Arc::new(RwLock::new(conf)),
    root_path: arg.root_path.clone(),
    api_root_path: arg.api_root_path.clone(),
    scheduler_admin,
    //mcp_state,
  });

  let state_for_mcp_sse = state.clone();
  let state_for_mcp_http = state.clone();
  mcp_sse_server.with_service(move || A2AMcp::new(state_for_mcp_sse.clone()));
  let mcp_http_service = StreamableHttpService::new(
    move || Ok(A2AMcp::new(state_for_mcp_http.clone())),
    LocalSessionManager::default().into(),
    Default::default(),
  );

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
    .nest_service(mcp_path, mcp_http_service)
    .with_state(state)
    .merge(mcp_sse_router)
    .layer(cors);

  let listener = tokio::net::TcpListener::bind(&arg.listen).await?;

  if !arg.no_ui {
    let work_dir_html = arg.html_root_path.clone();
    let listen_addr = arg.listen.to_string();
    tokio::spawn(async move {
      // ensure the server is ready before opening the browser
      tokio::time::sleep(std::time::Duration::from_secs(1)).await;
      setup_ui_files(work_dir_html);
      open_browser(listen_addr);
    });
  }

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

fn show_runtime_info(arg: &Serve, mcp_arg: &SseServerConfig) {
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
    (
      arg.mcp_path.as_deref().unwrap_or("/mcp"),
      "MCP StreamHTTP endpoint",
    ),
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

/// copy ui files from current 'html' directory to the work dir
///
fn setup_ui_files(dest_dir: PathBuf) {
  let src_dir = PathBuf::from("html");

  let src_version = std::fs::read_to_string(src_dir.join("version.txt")).unwrap_or_default();
  let dest_version = std::fs::read_to_string(dest_dir.join("version.txt")).unwrap_or_default();

  if src_version != dest_version {
    copy_dir(&src_dir, &dest_dir).unwrap_or_else(|err| {
      error!(%err, "Failed to copy UI files");
    });
  }
}

fn open_browser(listen_addr: String) {
  let port = listen_addr.split(':').last().unwrap_or("30030");
  let url = format!("http://localhost:{}", port);

  webbrowser::open(&url).unwrap_or_else(|err| {
    error!(%err, "Failed to open browser");
  });
}

fn copy_dir(src: &PathBuf, dest: &PathBuf) -> Result<()> {
  if !dest.exists() {
    std::fs::create_dir_all(dest)?;
  }
  for entry in std::fs::read_dir(src)? {
    let entry = entry?;
    let path = entry.path();
    if path.is_dir() {
      copy_dir(&path, &dest.join(entry.file_name()))?;
    } else {
      std::fs::copy(&path, dest.join(entry.file_name()))?;
    }
  }
  Ok(())
}
