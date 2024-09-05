use std::collections::HashMap;

use bytes::Bytes;
use serde::{Deserialize, Serialize};

mod convert;
mod value_bytes;
pub use serde_json::Value;

pub use value_bytes::{is_bytes, json_from_bytes, json_to_bytes};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HttpAction {
  // common fields
  pub override_result_mimetype: Option<String>,

  // http fields
  pub method: String,
  pub url: String,
  pub headers: Option<HashMap<String, String>>,
  // proxy to use for this request, otherwise use the default proxy, eg HTTP_PROXY from the environment
  pub proxy: Option<String>,
  pub body: Option<Bytes>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct HttpActionResult {
  pub status: u16,
  pub headers: Option<HashMap<String, String>>,
  pub body: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileAction {
  // common fields
  pub override_result_mimetype: Option<String>,

  // file fields
  pub method: String,
  pub path: String,
  pub body: Bytes,
  pub connection: Option<Value>,
}

pub type FileActionResult = Value;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SqlAction {
  // common fields
  pub override_result_mimetype: Option<String>,

  // sql fields
  pub query: String,
  pub rows: Option<Value>,
  pub connection: String,
}

pub type SqlActionResult = Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Action {
  Http(HttpAction),
  File(FileAction),
  Sql(SqlAction),
}
