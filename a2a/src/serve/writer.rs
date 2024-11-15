/*
use std::{
  convert::Infallible,
  sync::{Arc, OnceLock},
  vec,
};

use anyhow::Result;
use axum::{
  body::Body,
  extract::State,
  response::{
    sse::{Event, KeepAlive},
    IntoResponse, Sse,
  },
  Json,
};
use axum_streams::StreamBodyAs;
use futures::{stream, Stream, StreamExt};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::warn;

use super::AppState;
use crate::coder::{write_code_stream, DEFAULT_SYSTEM_PROMPT};

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
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
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
    Ok(s) => {
      let s = s.map(|v| Event::default().data(v)).map(Ok);
      Sse::new(s).keep_alive(KeepAlive::default())
    }
    Err(err) => {
      warn!("coder {}", err);
      let s = stream::once(async { Ok(Event::default().data(err.to_string())) });
      Sse::new(s).keep_alive(KeepAlive::default())
    }
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
    system: DEFAULT_SYSTEM_PROMPT.to_string(),
    base_url: std::env::var("OPENAI_BASE_URL").unwrap_or_default(),
    api_key: std::env::var("OPENAI_API_KEY").unwrap_or_default(),
  })
}
*/
