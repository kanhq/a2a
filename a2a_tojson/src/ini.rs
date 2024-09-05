use configparser::ini::Ini;
use crate::{Value, Result};

use crate::utils::json_typed;

pub(crate) fn to_json(input: String, _options: Option<&Value>) -> Result<Value> {

  let mut config = Ini::new_cs();
  config.read(input).map_err(|err| anyhow::anyhow!(err))?;

  let mut map = serde_json::Map::new();

  config.get_map_ref().iter().for_each(|(section, props)| {
    if section.eq("default") {
      props.iter().for_each(|(key, value)| {
        map.insert(key.to_string(), json_typed(value.clone().unwrap_or_default()));
      });
    }else{
      let mut obj = serde_json::Map::new();
      props.iter().for_each(|(key, value)| {
        obj.insert(key.to_string(), json_typed(value.clone().unwrap_or_default()));
      });
      map.insert(section.to_string(), Value::Object(obj));
    }
  });

  Ok(Value::Object(map))
}