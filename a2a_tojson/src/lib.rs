//! # tojson
//!
//! convert any supported format to json
//!
//! ## JSON
//! - mime type: application/json
//! ## NDJSON
//! - mime type: application/ndjson
//! ## CSV
//! - mime type: text/csv
//! - options:
//!   - has_header: bool, default false
//!   - delimiter: string, default ','
//!   - as_object: bool, default false, if true, each row will be converted to object with header as key, only works if has_header is true, otherwise each row will be converted to array
//! ## INI
//! - mime type: text/ini
//! ## YAML
//! - mime type: text/yaml | application/yaml | application/x-yaml
//!
mod config_loader;
mod csv;
mod data_bytes;
mod ini;
mod ndjson;
mod utils;
mod yaml;

use anyhow::Result;
use serde_json::{json, Value};
use utils::json_typed;

pub use config_loader::load_conf_dir;

/// convert any supported format to json
///
/// see module documentation for supported format
pub fn to_json<S: AsRef<str>>(input: String, mimetype: S, conf: Option<&Value>) -> Result<Value> {
  let mimetype = mimetype.as_ref();

  match mimetype {
    "text/csv" => csv::to_json(input, conf),
    "application/json" => serde_json::from_str(&input).map_err(|err| err.into()),
    "application/ndjson" => ndjson::to_json(input, conf),
    "text/ini" => ini::to_json(input, conf),
    "text/yaml" | "application/yaml" | "application/x-yaml" => yaml::to_json(input, conf),
    _ => Ok(Value::String(input)),
  }
}

pub fn bytes_to_json<S: AsRef<str>>(
  input: bytes::Bytes,
  mimetype: S,
  conf: Option<&Value>,
) -> Result<Value> {
  let mimetype = mimetype.as_ref();
  match mimetype {
    // pass all text based mime type to to_json
    "text/csv" | "application/json" | "application/ndjson" | "text/ini" | "text/yaml"
    | "plain/text" | "application/yaml" | "application/x-yaml" => {
      to_json(String::from_utf8(input.into())?, mimetype, conf)
    }
    // else convert to bytes
    _ => {
      let conf = json!({
        "mimetype": mimetype,
      });
      input.as_ref().to_json(Some(&conf))
    }
  }
}

pub fn to_json_value(text: String) -> Value {
  json_typed(text)
}

/// A trait to convert any supported format to json
pub trait ToJsonValue {
  fn to_json(&self, conf: Option<&Value>) -> Result<Value>;
}

pub trait FromJsonValue: Sized {
  fn from_json(value: &Value) -> Result<Self>;
}
