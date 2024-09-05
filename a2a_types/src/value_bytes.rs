use serde_json::{Number, Value};

pub fn is_bytes(value: &Value) -> bool {
  match value {
    Value::Array(a) => a.iter().all(|v| match v {
      Value::Number(n) => n.as_u64().map(|n| n < 256).unwrap_or(false),
      _ => false,
    }),
    _ => false,
  }
}

pub fn json_to_bytes(value: &Value) -> Option<Vec<u8>> {
  match value {
    Value::Array(a) => a
      .iter()
      .map(|v| match v {
        Value::Number(n) => n
          .as_u64()
          .and_then(|n| if n < 256 { Some(n as u8) } else { None }),
        _ => None,
      })
      .collect::<Option<Vec<u8>>>(),
    _ => None,
  }
}

pub fn json_from_bytes(bytes: &[u8]) -> Value {
  Value::Array(
    bytes
      .iter()
      .map(|b| Value::Number(Number::from(*b)))
      .collect(),
  )
}

#[cfg(test)]
mod tests {

  #[test]
  fn test_bytes() {
    let bytes = vec![1, 2, 3];
    let json = serde_json::json!([1, 2, 3]);
    assert_eq!(super::is_bytes(&json), true);
    assert_eq!(super::json_to_bytes(&json), Some(bytes.clone()));
    assert_eq!(super::json_from_bytes(&bytes), json);

    let json = serde_json::json!([1, 2, 256]);
    assert_ne!(super::is_bytes(&json), true);
    assert_eq!(super::json_to_bytes(&json), None);
  }
}
