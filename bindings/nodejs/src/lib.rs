#![deny(clippy::all)]

use a2a_types::{Action, Value};
use napi_derive::napi;

#[napi]
pub async fn do_action(action: Value) -> napi::Result<Value> {
  let action: Action = serde_json::from_value(action)?;
  a2a_core::do_action(action).await.map_err(|err| err.into())
}

#[napi]
pub fn load_config(conf_dir: String) -> napi::Result<Value> {
  a2a_tojson::load_conf_dir(conf_dir).map_err(|err| err.into())
}
