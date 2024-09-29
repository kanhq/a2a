use std::sync::Arc;

use a2a_types::Value;
use axum::{
  extract::{Path, Query, State},
  Json,
};
use reqwest::StatusCode;
use serde_json::json;
use tracing::{debug, info, warn};

use super::AppState;

#[axum::debug_handler]
pub(crate) async fn post_handler(
  State(state): State<Arc<AppState>>,
  Path(file): Path<String>,
  query: Query<Value>,
  Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
  let state = state.as_ref();
  let file = file.as_str();
  api_handler(state, file, query.0, body).await
}

#[axum::debug_handler]
pub(crate) async fn get_handler(
  State(state): State<Arc<AppState>>,
  Path(file): Path<String>,
  query: Query<Value>,
) -> (StatusCode, Json<Value>) {
  let state = state.as_ref();
  let file = file.as_str();
  api_handler(state, file, query.0, Value::Null).await
}

async fn api_handler(
  state: &AppState,
  file: &str,
  mut query: Value,
  body: Value,
) -> (StatusCode, Json<Value>) {
  info!("api {} {} {}", file, query, body);

  let mut script_file = state.api_root_path.join(file);
  if script_file.extension().is_none() {
    script_file.set_extension("js");
  }
  if !script_file.exists() {
    debug!("script file not found: {:?}", script_file);
    return (StatusCode::NOT_FOUND, Json(Value::Null));
  }
  let filename = script_file.to_str().unwrap_or_default();
  if filename.is_empty() {
    warn!("invalid script file: {:?}", script_file);
    return (StatusCode::INTERNAL_SERVER_ERROR, Json(Value::Null));
  }
  let params = if let Some(q) = query.as_object_mut() {
    q.insert("body".to_string(), body);
    query
  } else {
    json!({"body": body, "query": query})
  };
  let conf = state.conf.clone();

  match crate::run::execute_js(filename, &conf, &params, None).await {
    Ok(val) => (StatusCode::OK, Json(val)),
    Err(err) => {
      debug!("api error: {}", err);
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"error": err.to_string()})),
      )
    }
  }
}
