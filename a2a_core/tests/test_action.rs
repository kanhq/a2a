use a2a_core::do_action;
use a2a_types::{Action, SqlAction};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize)]
struct TestConfig {
  pgsql: String,
  mysql: String,
  sqlite: String,
}

#[tokio::test]
async fn test_sql() {
  let conifg_data = include_str!("./config.json");

  let conf = serde_json::from_str::<TestConfig>(conifg_data).unwrap();

  //let connections = vec![conf.pgsql.clone(), conf.mysql.clone(), conf.sqlite.clone()];
  let connections = vec![conf.sqlite.clone()];

  let actions = vec![
    SqlAction {
      query: r#"
      CREATE TABLE IF NOT EXISTS a2a_test (
        id INT PRIMARY KEY,
        name TEXT NOT NULL,
        last_updated TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
      );
      "#
      .to_string(),
      ..Default::default()
    },
    SqlAction {
      query: r#"
      INSERT INTO a2a_test (id, name) VALUES (?, ?);
      "#
      .to_string(),
      rows: Some(json!([[1, "user1"], [2, "user2"]])),
      ..Default::default()
    },
    SqlAction {
      query: r#"
      SELECT * FROM a2a_test;
      "#
      .to_string(),
      ..Default::default()
    },
    SqlAction {
      query: r#"
      SELECT * FROM a2a_test WHERE id = ?;
      "#
      .to_string(),
      rows: Some(json!([1])),
      ..Default::default()
    },
    SqlAction {
      query: r#"
      DROP TABLE IF EXISTS a2a_test;
      "#
      .to_string(),
      ..Default::default()
    },
  ];

  for conn in connections {
    for action in actions.iter() {
      let action = action.clone();
      let action = Action::Sql(SqlAction {
        connection: conn.clone(),
        ..action
      });
      let result = do_action(action).await.unwrap();
      println!("{}", serde_json::to_string_pretty(&result).unwrap());
    }
  }
}
