use std::path::Path;

use a2a_tojson::to_json;
use anyhow::{bail, Result};
use serde_json::{Map, Value};
use tracing::{debug, trace, warn};

pub(crate) fn load_conf_dir(conf_dir: &Path) -> Result<Value> {
  if !conf_dir.exists() {
    bail!("config directory not found");
  }

  if conf_dir.is_file() {
    return load_configs(conf_dir.to_str().unwrap_or_default());
  } else {
    let pattern = conf_dir.join("**/*.*");
    // replace / with \ on windows
    if cfg!(windows) {
      let pattern = pattern.to_string_lossy().replace("/", "\\");
      load_configs(&pattern)
    } else {
      load_configs(pattern.to_str().unwrap_or_default())
    }
  }
}

pub(crate) fn load_configs(pattern: &str) -> Result<Value> {
  debug!(?pattern, "load config dir");
  let walker = glob::glob(pattern)?;

  let mut conf = Default::default();

  for entry in walker {
    match entry {
      Ok(entry) => {
        let file_name = entry.as_path();
        trace!(?file_name, "load config");
        let ext = file_name.extension().unwrap_or_default();
        let mimetype = ext_to_mime(ext.to_str().unwrap_or_default());
        if mimetype.is_empty() {
          debug!(?file_name, "unsupported file type");
          continue;
        }
        if let Err(err) = merge_config_file(&mut conf, file_name, mimetype) {
          warn!(?file_name, mimetype, ?err, "load config failed");
        }
      }
      Err(err) => {
        warn!(?err, "load config failed");
      }
    }
  }

  Ok(Value::Object(conf))
}

fn merge_config_file(
  conf: &mut Map<String, Value>,
  file_name: &Path,
  mimetype: &str,
) -> Result<()> {
  let content = std::fs::read_to_string(file_name)?;
  let value = to_json(content, mimetype, None)?;
  if let Value::Object(mut obj) = value {
    conf.append(&mut obj);
    Ok(())
  } else {
    bail!("content should be a map");
  }
}

pub(crate) fn ext_to_mime(ext: &str) -> &str {
  match ext {
    "json" => "application/json",
    "yaml" | "yml" => "application/yaml",
    "conf" | "ini" | "env" => "text/ini",
    "csv" => "text/csv",
    _ => "",
  }
}
