use a2a_tojson::{ToJsonValue, FromJsonValue};

#[test]
fn test_bytes() {

  let input = &b"hello"[..];
  let json_value = input.to_json(None).unwrap();
  let expected = serde_json::json!("hex:68656c6c6f");
  assert_eq!(json_value, expected);

  let input = serde_json::json!("hex:68656c6c6f");
  let bytes = Vec::<u8>::from_json(&input).unwrap();
  let expected = b"hello".to_vec();
  assert_eq!(bytes, expected);
}