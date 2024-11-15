use std::{io::Write, sync::Arc};

use a2a_types::Value;
use anyhow::Result;
use axum::{extract::State, Json};
use axum_extra::extract::Multipart;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use crate::run::execute_js_code;

use super::AppState;

#[derive(Debug, Deserialize, Serialize)]
pub struct OneShotRequest {
  script: String,
  config: Value,
  params: Value,
}

pub async fn post_json_handle(
  State(_state): State<Arc<AppState>>,
  Json(req): Json<OneShotRequest>,
) -> (StatusCode, Json<Value>) {
  let result = execute_js_code(&req.script, &req.config, &req.params, None).await;

  match result {
    Ok(val) => (StatusCode::OK, Json(val)),
    Err(err) => {
      warn!("oneshot {}", err);
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(Value::String(err.to_string())),
      )
    }
  }
}

pub async fn post_form_handle(
  State(state): State<Arc<AppState>>,
  form: Multipart,
) -> (StatusCode, Json<Value>) {
  match post_form_handle_impl(&state, form).await {
    Ok(val) => (StatusCode::OK, Json(val)),
    Err(err) => {
      warn!(?err, "oneshot failed");
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(Value::String(err.to_string())),
      )
    }
  }
}

async fn post_form_handle_impl(_state: &AppState, mut form: Multipart) -> Result<Value> {
  let base = tempfile::Builder::new().prefix("a2a-").tempdir()?;

  debug!(?base, "oneshot start");

  let mut code = String::new();
  let mut conf = Value::Null;
  let mut params = serde_json::Map::new();

  while let Some(mut field) = form.next_field().await? {
    let name = field.name().unwrap().to_string();
    let file_name = field.file_name().map(|s| s.to_string());

    match name.as_str() {
      "script" => {
        code = field.text().await?;
      }
      "config" => {
        let data = field.bytes().await?;
        conf = serde_json::from_slice(&data)?;
      }
      _ => {
        if let Some(file_name) = file_name {
          let mut file = base.path().to_path_buf();
          if let Some(p) = file_name.rsplit_once('/') {
            file.push(p.1);
          } else {
            file.push(file_name);
          }
          let file_name = file.to_string_lossy().to_string();
          let mut file = std::fs::File::create(file)?;
          while let Some(chunk) = field.chunk().await? {
            file.write_all(&chunk)?;
          }
          params.insert(name, Value::String(file_name));
        } else {
          let data = field.text().await?;
          params.insert(name, Value::String(data));
        }
      }
    }
  }
  execute_js_code(&code, &conf, &Value::Object(params), None).await
}
