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
  pub timeout: Option<f64>,
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
  pub body: Option<Value>,
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
  // save args as file specified by arg_as_file
  pub arg_as_file: Option<String>,
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

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CrawlAction {
  // common fields
  pub override_result_mimetype: Option<String>,

  // crawl fields
  /// the browser options used to crawl, default to find the browser from the environment
  /// see https://docs.rs/headless_chrome/latest/headless_chrome/browser/struct.LaunchOptions.html for more details
  pub browser: Option<Value>,
  /// the url used to crawl
  /// can be a string or a object with 'url', 'selector', 'wait' fields
  pub urls: Vec<Value>,
  /// the parallel number of the urls, default is 1
  pub parallel: Option<usize>,
  /// the llm used to extract data from crawl result by prompt
  pub llm: Option<Value>,
  /// the map of url pattern to extract prompt
  /// the key is the url pattern, the value is the prompt
  /// key can be
  /// - a string, eg "https://example.com"
  /// - string with wildcard, eg "https://example.com/*"
  /// - string start with '/' will be treated as regex, eg "/https://example.com/.*"
  /// the value is the is struct of data to extract, it can be
  /// - a string, full typescript type definition
  /// - a array of string, the fields to extract, each field will be treated as string type
  pub fields: Option<Value>,
}
pub type CrawlActionResult = Value;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WebSearchAction {
  // common fields
  pub override_result_mimetype: Option<String>,

  // web search fields
  /// the browser options used to crawl, default to find the browser from the environment
  /// see https://docs.rs/headless_chrome/latest/headless_chrome/browser/struct.LaunchOptions.html for more details
  pub browser: Option<Value>,
  /// keyword to search
  pub query: String,
  /// the search provider, may be a provider name or a url, when a url is used, the ${query} of url will be replaced with the query
  pub provider: String,
  /// the search provider options, will be passed to the provider as query params
  pub options: Option<Value>,
  /// how many pages to search, default is 3
  pub pages: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WebSearchResult {
  pub url: String,
  pub title: String,
  pub body: String,
  pub icon: String,
}

pub type WebSearchActionResult = Vec<WebSearchResult>;

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Action {
  Http(HttpAction),
  File(FileAction),
  Sql(SqlAction),
  EMail(EMailAction),
  Shell(ShellAction),
  Llm(LlmAction),
  Notify(NotifyAction),
  Enc(EncAction),
  Crawl(CrawlAction),
  WebSearch(WebSearchAction),
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

impl Action {
  pub fn get_kind(&self) -> &str {
    match self {
      Action::Http(_) => "http",
      Action::File(_) => "file",
      Action::Sql(_) => "sql",
      Action::EMail(_) => "email",
      Action::Shell(_) => "shell",
      Action::Llm(_) => "llm",
      Action::Notify(_) => "notify",
      Action::Enc(_) => "enc",
      Action::Crawl(_) => "crawl",
      Action::WebSearch(_) => "web_search",
    }
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
