use std::str::FromStr;

use a2a_tojson::to_json;
use a2a_types::Value;
use serde_json::json;

#[test]
fn test_yaml() {
  let input = r#"
- name: alice
  age: 20
- name: bob
  age: 30
- foo:
    bar: 40
"#;
  let result = to_json(input.to_string(), "text/yaml", None).unwrap();
  let expected = json!(
    [
      {"name": "alice", "age": 20},
      {"name": "bob", "age": 30},
      {"foo": {"bar": 40}}
    ]
  );
  assert_eq!(result, expected);
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
enum Message {
  Request { id: String, method: String },
  Response { id: String },
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Action {
  #[serde(flatten)]
  message: Message,
  timestamp: i64,
}

#[test]
fn test_serde_enum() {
  let action = Action {
    message: Message::Request {
      id: "123".to_string(),
      method: "GET".to_string(),
    },
    timestamp: 123456,
  };

  let body = serde_json::to_string_pretty(&action).unwrap();

  println!("{}", body);
}

#[test]
fn test_from_str() {
  assert_eq!(json!(true), Value::from_str("true").unwrap());
  assert_eq!(json!(123), Value::from_str("123").unwrap());
  assert_eq!(json!("abc"), Value::from_str("abc").unwrap());
  assert_eq!(
    json!({"a":123, "b":true, "c":"123"}),
    Value::from_str(r#"{"a":123, "b":true, "c":"123"}"#).unwrap()
  );
}
