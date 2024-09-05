use std::{collections::HashMap, str::FromStr};

use a2a_tojson::bytes_to_json;
use a2a_types::{FileAction, FileActionResult};
use anyhow::{anyhow, Result};
use serde_json::json;

fn split_schema_path(full: &str) -> (&str, &str) {
  full
    .split_once("://")
    .map(|(schema, path)| {
      if schema.is_empty() || schema.eq("file") {
        ("fs", path)
      } else {
        (schema, path)
      }
    })
    .unwrap_or(("fs", full))
}

pub async fn do_file_action(action: FileAction) -> Result<FileActionResult> {
  let (schema, path) = split_schema_path(&action.path);
  let scheme = opendal::Scheme::from_str(schema)?;
  let options = action
    .connection
    .and_then(|c| c.as_object().cloned())
    .map(|m| {
      m.into_iter()
        .map(|(k, v)| (k, v.to_string()))
        .collect::<HashMap<_, _>>()
    })
    .unwrap_or_default();

  let op = opendal::Operator::via_iter(scheme, options)?;

  let method = action.method.to_lowercase();

  match method.as_str() {
    "read" => {
      let body = op.read(path).await?.to_bytes();
      let mimetype = action
        .override_result_mimetype
        .ok_or(mimetype_from_ext(path))
        .unwrap_or_default();
      bytes_to_json(body, mimetype, None)
    }
    "write" => {
      let body = action.body.to_vec();
      op.write(path, body).await?;
      Ok(serde_json::Value::Null)
    }
    "delete" => {
      op.delete(path).await?;
      Ok(serde_json::Value::Null)
    }
    "list" => {
      let items = op.list(path).await?;

      let items = items
        .into_iter()
        .map(|item| {
          json!({
            "path": item.path(),
            "name": item.name(),
            "isDir": item.metadata().is_dir(),
            "size": item.metadata().content_length(),
            "contentMd5": item.metadata().content_md5(),
            "eTag": item.metadata().etag(),
            "lastModified": item.metadata().last_modified().map(|t| t.timestamp_micros()),
          })
        })
        .collect::<Vec<_>>();
      Ok(json!(items))
    }
    _ => Err(anyhow!("Unsupported file method: {}", method)),
  }
}

fn mimetype_from_ext(path: &str) -> String {
  let ext = path
    .split('.')
    .last()
    .map(|ext| ext.to_lowercase())
    .unwrap_or_default();
  let mimetype = match ext.as_str() {
    "json" => "application/json",
    "txt" => "text/plain",
    "html" => "text/html",
    "xml" => "application/xml",
    "csv" => "text/csv",
    "tsv" => "text/tab-separated-values",
    "png" => "image/png",
    "jpg" | "jpeg" => "image/jpeg",
    "gif" => "image/gif",
    "bmp" => "image/bmp",
    "tiff" => "image/tiff",
    "pdf" => "application/pdf",
    "doc" => "application/msword",
    "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    "xls" => "application/vnd.ms-excel",
    "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    "ppt" => "application/vnd.ms-powerpoint",
    "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
    _ => "application/octet-stream",
  };
  mimetype.to_string()
}
