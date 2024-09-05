
use crate::{Value, Result};

pub(crate) fn to_json(input: String, _options: Option<&Value>) -> Result<Value> {
  serde_yml::from_str(&input).map_err(|err| anyhow::anyhow!(err))
}


//use yaml_rust2::{Yaml, YamlLoader};


/*
pub(crate) fn to_json(input: String, _options: Option<&Value>) -> Result<Value> {

  let docs = YamlLoader::load_from_str(&input).map_err(|err| anyhow::anyhow!(err))?;

  if docs.len() == 1 {
    docs.into_iter().next()
      .map(|doc| yaml_to_json(doc)) 
      .ok_or(anyhow::anyhow!("No document found"))
  }else{
    let a: Vec<Value> = docs.into_iter().map(|doc| yaml_to_json(doc)).collect();
    Ok(Value::Array(a))

  }
}

fn yaml_to_json(y : Yaml) -> Value {
  match y {
    Yaml::Real(s) => Value::Number(serde_json::Number::from_f64(s.parse().unwrap()).unwrap()),
    Yaml::Integer(i) => Value::Number(serde_json::Number::from(i)),
    Yaml::String(s) => Value::String(s),
    Yaml::Boolean(b) => Value::Bool(b),
    Yaml::Array(a) => Value::Array(a.into_iter().map(|y| yaml_to_json(y)).collect()),
    Yaml::Hash(h) => {
      let mut map = serde_json::Map::new();
      for (k, v) in h {
        map.insert(k.as_str().unwrap().to_string(), yaml_to_json(v));
      }
      Value::Object(map)
    },
    Yaml::Null => Value::Null,
    _ => Value::Null
  }
} 
*/