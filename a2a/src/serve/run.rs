use std::{io::Write, sync::Arc};

use a2a_tojson::FromJsonValue;
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
  config: Option<Value>,
  params: Value,
}

#[derive(Debug, Deserialize, Serialize)]
struct ParamFile {
  kind: String,
  name: String,
  content: Value,
}

pub async fn post_json_handle(
  State(state): State<Arc<AppState>>,
  Json(req): Json<OneShotRequest>,
) -> (StatusCode, Json<Value>) {
  match post_json_handle_impl(&state, req).await {
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

fn config_or_default(state: &AppState, conf: Option<&Value>) -> Value {
  match conf {
    Some(Value::Object(m)) => Value::Object(m.clone()),
    _ => match state.conf.read() {
      Ok(c) => c.clone(),
      Err(e) => {
        warn!(?e, "Failed to read configuration, using default");
        Value::Null
      }
    },
  }
}

async fn post_json_handle_impl(state: &AppState, mut req: OneShotRequest) -> Result<Value> {
  let base = tempfile::Builder::new().prefix("a2a-").tempdir()?;
  debug!(?base, "oneshot start");

  req.params.as_object_mut().map(|m| {
    m.iter_mut().for_each(
      |(_k, v)| match serde_json::from_value::<ParamFile>(v.clone()) {
        Ok(pf) => {
          let mut file = base.path().to_path_buf();
          file.push(pf.name);
          match Vec::<u8>::from_json(&pf.content).map(|data| std::fs::write(&file, data)) {
            Ok(_) => {
              *v = Value::String(file.to_string_lossy().to_string());
            }
            Err(err) => {
              warn!(?err, "save param file failed");
            }
          }
        }
        Err(_) => {}
      },
    );
  });

  let conf = config_or_default(state, req.config.as_ref());
  execute_js_code(&req.script, &conf, &req.params, None).await
}

async fn post_form_handle_impl(state: &AppState, mut form: Multipart) -> Result<Value> {
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
  let conf = config_or_default(state, Some(&conf));
  execute_js_code(&code, &conf, &Value::Object(params), None).await
}
