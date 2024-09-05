use crate::Value;

pub(crate) fn json_typed(text: String) -> Value {
  if text == "true" || text == "false" {
    Value::Bool(text.parse().unwrap())
  } else if let Ok(num) = text.trim().parse::<i64>() {
    Value::Number(serde_json::Number::from(num))
  } else if let Ok(num) = text.trim().parse::<f64>() {
    Value::Number(serde_json::Number::from_f64(num).unwrap())
  } else {
    Value::String(text)
  }
}
