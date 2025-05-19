use std::{collections::HashMap, path, str::FromStr};

use a2a_tojson::{bytes_to_json, to_mimetype_bytes};
use a2a_types::{FileAction, FileActionResult};
use anyhow::{anyhow, Result};
use opendal::Scheme;
use serde_json::json;

fn split_schema_path(full: &str) -> (&str, String) {
  full
    .split_once("://")
    .map(|(schema, path)| {
      if schema.is_empty() || schema.eq("file") {
        let p = path::Path::new(path);
        match p.canonicalize() {
          Ok(p) => ("fs", p.to_string_lossy().to_string()),
          Err(_) => ("fs", path.to_string()),
        }
      } else {
        (schema, path.to_string())
      }
    })
    .unwrap_or(("fs", full.to_string()))
}

fn read_data_url(data_url: &str) -> Option<FileActionResult> {
  if !data_url.starts_with("data:") {
    return None;
  }

  let data_url = data_url.trim_start_matches("data:");

  let mut parts = data_url.splitn(2, ',');
  let (mime, data) = (parts.next()?, parts.next()?);
  let body = base64_simd::STANDARD.decode_to_vec(data.as_bytes()).ok()?;
  let mimetype = mime.split(';').next().unwrap_or("text/plain");
  bytes_to_json(body.into(), mimetype, None).ok()
}

pub async fn do_action(action: FileAction) -> Result<FileActionResult> {
  if action.method.eq_ignore_ascii_case("read") {
    if let Some(value) = read_data_url(&action.path) {
      return Ok(value);
    }
  }

  let (schema, mut path) = split_schema_path(&action.path);
  let scheme = opendal::Scheme::from_str(schema)?;
  let mut options = action
    .connection
    .and_then(|c| c.as_object().cloned())
    .map(|m| {
      m.into_iter()
        .map(|(k, v)| (k, v.to_string()))
        .collect::<HashMap<_, _>>()
    })
    .unwrap_or_default();

  if let Scheme::Fs = scheme {
    if !options.contains_key("root") {
      if path.starts_with("/") {
        options.insert("root".to_string(), "/".to_string());
      } else if path.contains(":\\") {
        let (driver, p) = path.split_at(3);
        options.insert("root".to_string(), format!("{}", driver));
        path = p.to_string();
      } else {
        options.insert("root".to_string(), ".".to_string());
      }
    }
  }

  let op = opendal::Operator::via_iter(scheme, options)?;

  let method = action.method.to_lowercase();

  match method.as_str() {
    "read" => {
      let body = op.read(&path).await?.to_bytes();
      let mimetype = action
        .override_result_mimetype
        .unwrap_or(mimetype_from_ext(&path));
      bytes_to_json(body, mimetype, None)
    }
    "write" => {
      if let Some(input) = action.body.as_ref() {
        let mimetype = action
          .override_result_mimetype
          .unwrap_or(mimetype_from_ext(&path));
        let body = to_mimetype_bytes(input, mimetype)?;
        op.write(&path, body).await?;
      }
      Ok(serde_json::Value::Null)
    }
    "delete" => {
      op.delete(&path).await?;
      Ok(serde_json::Value::Null)
    }
    "list" => {
      let (path, recursive, pattern) = match path.as_str().find("**") {
        Some(idx) => (
          &path.as_str()[..idx],
          true,
          Some(glob::Pattern::new(&path.as_str()[idx..])?),
        ),
        None => (path.as_str(), false, None),
      };

      let items = op.list_with(path).recursive(recursive).await?;

      let items = items
        .into_iter()
        .filter(|item| {
          if let Some(pattern) = &pattern {
            pattern.matches(item.path())
          } else {
            true
          }
        })
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
