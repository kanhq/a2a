use std::sync::Arc;

use a2a_types::Value;
use anyhow::Result;
use axum::{
  extract::{Query, State},
  Json,
};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::warn;

use crate::serve::AppState;

use super::{
  scheduler::ScheduleEvent,
  workspace::{handle_workspace_operation, WorkspaceOperation},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind", content = "data")]
enum AdminRequest {
  Schedule(ScheduleEvent),
  ChangeWorkDir(String),
  Workspace(WorkspaceOperation),
}

#[axum::debug_handler]
pub(crate) async fn post_handler(
  State(state): State<Arc<AppState>>,
  Query(query): Query<Value>,
  Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
  let state = state.as_ref();

  match admin_handler(state, query, body).await {
    Ok(val) => (StatusCode::OK, Json(val)),
    Err(err) => {
      warn!("admin {}", err);
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({ "error": err.to_string() })),
      )
    }
  }
}

async fn admin_handler(state: &AppState, _query: Value, body: Value) -> Result<Value> {
  let req = serde_json::from_value::<AdminRequest>(body)?;
  match req {
    AdminRequest::Schedule(event) => state
      .scheduler_admin
      .send(event)
      .await
      .map(|_| Value::Null)
      .map_err(Into::into),
    AdminRequest::ChangeWorkDir(new_path) => {
      if new_path.is_empty() {
        return Err(anyhow::anyhow!("New work directory cannot be empty"));
      }
      let mut lock = state
        .root_path
        .lock()
        .map_err(|_| anyhow::anyhow!("Failed to lock the root path mutex"))?;
      *lock = new_path;
      Ok(Value::Null)
    }
    AdminRequest::Workspace(op) => {
      let root_path = {
        let lock = state.root_path.lock().map_err(|_| {
          anyhow::anyhow!("Failed to lock the root path mutex for workspace operation")
        })?;
        lock.clone()
      };
      let response = handle_workspace_operation(root_path, op).await?;
      Ok(serde_json::to_value(response)?)
    }
  }
}
