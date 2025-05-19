use crate::{Result, Value};

pub(crate) fn to_json(input: String, _options: Option<&Value>) -> Result<Value> {
  let a: Vec<Value> = input
    .lines()
    .filter_map(|s| serde_json::from_str(s).ok())
    .collect();

  if a.len() == 1 {
    Ok(a.into_iter().next().unwrap())
  } else {
    Ok(Value::Array(a))
  }
}

pub(crate) fn to_mimetype_bytes(input: &Value) -> Result<bytes::Bytes> {
  if input.is_array() {
    let mut result = String::new();
    for item in input.as_array().unwrap() {
      result.push_str(&serde_json::to_string(item)?);
      result.push('\n');
    }
    Ok(bytes::Bytes::from(result))
  } else {
    serde_json::to_string(input)
      .map(|s| bytes::Bytes::from(s))
      .map_err(|err| err.into())
  }
}
