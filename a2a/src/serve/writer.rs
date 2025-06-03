use std::sync::{Arc, OnceLock};

use axum::{
  body::Body,
  extract::State,
  response::{IntoResponse, Response},
  Json,
};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::{error, warn};

use super::AppState;
use crate::coder::{default_system_prompt, write_code_stream, DEFAULT_SYSTEM_PROMPT};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteRequest {
  name: String,
  prompt: String,
  provider: String,
  model: String,
}

pub async fn coder_handle(
  State(_state): State<Arc<AppState>>,
  Json(req): Json<WriteRequest>,
) -> Response<Body> {
  if writer_conf().base_url.is_empty() || writer_conf().api_key.is_empty() {
    error!("server LLM configuration is missing, please ensure OPENAI_BASE_URL and OPENAI_API_KEY environment variables have been set");
    return (
      StatusCode::INTERNAL_SERVER_ERROR,
      "Server LLM configuration is missing",
    )
      .into_response();
  }

  let code = crate::coder::WriteCode {
    system: writer_conf().system.clone(),
    user: req.prompt,
    output: None,
    provider: req.provider,
    model: req.model,
    base_url: writer_conf().base_url.clone(),
    api_key: writer_conf().api_key.clone(),
    ..Default::default()
  };

  match write_code_stream(&code).await {
    Ok(r) => r,
    Err(err) => {
      warn!("coder {}", err);
      (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()
    }
  }
}

pub async fn system_prompt_handle(State(_state): State<Arc<AppState>>) -> Response<Body> {
  let system = default_system_prompt();
  if system.is_empty() {
    (StatusCode::NOT_FOUND, DEFAULT_SYSTEM_PROMPT).into_response()
  } else {
    (StatusCode::OK, system).into_response()
  }
}

struct WriteConf {
  system: String,
  base_url: String,
  api_key: String,
}

fn writer_conf() -> &'static WriteConf {
  static APP_CONF: OnceLock<WriteConf> = OnceLock::new();
  APP_CONF.get_or_init(|| WriteConf {
    system: default_system_prompt().to_string(),
    base_url: std::env::var("OPENAI_BASE_URL").unwrap_or_default(),
    api_key: std::env::var("OPENAI_API_KEY").unwrap_or_default(),
  })
}
