use serde_json::json;
use a2a_tojson::to_json;


#[test]
fn test_ini() {
  let input = r#"
pgconn=postgres://localhost:5432
alice=20
bob=30
[foo]
bar=40
"#;
  let result = to_json(input.to_string(), "text/ini", None).unwrap();
  let expected = json!(
    {
      "pgconn": "postgres://localhost:5432",
      "alice": 20,
      "bob": 30,
      "foo": {
        "bar": 40
      }
    }
  );
  assert_eq!(result, expected);
}
