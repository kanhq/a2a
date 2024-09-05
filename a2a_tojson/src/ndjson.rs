use crate::{Value, Result};


pub(crate) fn to_json(input: String, _options: Option<&Value>) -> Result<Value> {

  let a: Vec<Value> = input.lines()
  .filter_map(|s| serde_json::from_str(s).ok())
  .collect();

  if a.len() == 1 {
    Ok(a.into_iter().next().unwrap())
  }else{
    Ok(Value::Array(a))
  }
}