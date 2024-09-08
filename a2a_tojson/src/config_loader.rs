use std::path::Path;

use super::to_json;
use anyhow::{bail, Result};
use serde_json::{Map, Value};
use tracing::{trace, warn};

pub fn load_conf_dir<P: AsRef<Path>>(conf_dir: P) -> Result<Value> {
  let conf_dir = conf_dir.as_ref().canonicalize()?;
  if !conf_dir.exists() {
    bail!("config directory not found");
  }

  if conf_dir.is_file() {
    return load_configs(conf_dir.to_str().unwrap_or_default());
  } else {
    let pattern = conf_dir.join("**/*.{json,yaml,ini,env}");
    load_configs(pattern.to_str().unwrap_or_default())
  }
}

pub(crate) fn load_configs(pattern: &str) -> Result<Value> {
  let walker = globwalk::glob(pattern)?;

  let mut conf = Default::default();

  for entry in walker {
    match entry {
      Ok(entry) => {
        let file_name = entry.path();
        trace!(?file_name, "load config");
        let ext = file_name.extension().unwrap_or_default();
        let mimetype = ext_to_mime(ext.to_str().unwrap_or_default());
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
