use serde_json::json;
use a2a_tojson::to_json;


#[test]
fn test_csv_object() {
  let input = r#"
name,age
alice,20
bob,30
"#;
  let conf = json!({
    "has_header": true,
    "as_object": true,
  });
  let result = to_json(input.to_string(), "text/csv", Some(&conf)).unwrap();
  let expected = json!(
    [
      {"name": "alice", "age": 20},
      {"name": "bob", "age": 30}
    ]
  );
  assert_eq!(result, expected);
}

#[test]
fn test_csv_array() {
  let input = r#"
name,age
alice,20
bob,30
"#;
  let conf = json!({
    "has_header": true,
  });
  let result = to_json(input.to_string(), "text/csv", Some(&conf)).unwrap();
  let expected = json!(
    [
      ["name", "age"],
      ["alice", 20],
      ["bob", 30]
    ]
  );
  assert_eq!(result, expected);
}