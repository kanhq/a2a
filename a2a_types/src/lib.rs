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
  pub body: Option<Bytes>,
  pub connection: Option<Value>,
  // read options
  pub options: Option<Value>,
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

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EMailAction {
  // common fields
  pub override_result_mimetype: Option<String>,

  // email fields

  // method: SEND, RECV, DELETE
  pub method: String,
  // account configuration
  pub account: Value,
  // folder to use for this request, otherwise use the default folder
  pub folder: Option<String>,
  // message to send/delete
  pub message: Option<Value>,
  // lastId to recv
  pub last_id: Option<u64>,
}

// EMailActionResult is a array of Message
pub type EMailActionResult = Value;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ShellAction {
  // common fields
  pub override_result_mimetype: Option<String>,

  // shell fields
  pub command: String,
  pub args: Option<Vec<String>>,
  pub env: Option<HashMap<String, String>>,
  pub cwd: Option<String>,
}

pub type ShellActionResult = Value;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LlmAction {
  // common fields
  pub override_result_mimetype: Option<String>,

  // provider
  pub connection: Option<Value>,
  // system prompt
  pub sys_prompt: Option<String>,
  // user prompt
  pub user_prompt: Option<String>,
  // user input image, should be `Data Url` format
  pub user_image: Option<String>,
}

pub type LlmActionResult = Value;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NotifyAction {
  // common fields
  pub override_result_mimetype: Option<String>,

  // notify fields
  pub url: String,
  // message to send
  pub message: Value,
  pub title: Option<String>,
}

pub type NotifyActionResult = Value;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EncAction {
  // common fields
  pub override_result_mimetype: Option<String>,

  // enc fields
  /// is_dec: true if the data is decode or encoded
  pub is_dec: Option<bool>,
  pub methods: Vec<String>,
  pub key: Option<String>,
  pub padding: Option<String>,
  pub data: String,
}

pub type EncActionResult = String;

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Action {
  Http(HttpAction),
  File(FileAction),
  Sql(SqlAction),
  EMail(EMailAction),
  Shell(ShellAction),
  Llm(LlmAction),
  Notify(NotifyAction),
  Enc(EncAction),
}

struct FormatterWriter<'a, 'b> {
  f: &'a mut std::fmt::Formatter<'b>,
}

impl std::io::Write for FormatterWriter<'_, '_> {
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    let s = unsafe { std::str::from_utf8_unchecked(buf) };
    match self.f.write_str(s) {
      Ok(()) => Ok(buf.len()),
      Err(_) => Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "write error",
      )),
    }
  }

  fn flush(&mut self) -> std::io::Result<()> {
    Ok(())
  }
}

impl std::fmt::Debug for Action {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    let mut w = FormatterWriter { f };
    serde_json::to_writer(&mut w, self).map_err(|_| std::fmt::Error)
  }
}

pub struct ActionResultWrapper<'a> {
  pub result: &'a anyhow::Result<Value>,
}

impl std::fmt::Debug for ActionResultWrapper<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self.result {
      Err(e) => write!(f, "{:?}", e),
      Ok(v) => {
        let mut w = FormatterWriter { f };
        serde_json::to_writer(&mut w, v).map_err(|_| std::fmt::Error)
      }
    }
  }
}
