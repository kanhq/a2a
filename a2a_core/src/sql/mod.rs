use a2a_types::{SqlAction, SqlActionResult, Value};
use anyhow::Result;

mod mysql;
mod pgsql;
mod sqlite;

fn sql_driver(conn: &str) -> (&'static str, String) {
  if let Some((schema, conn)) = conn.split_once("://") {
    match schema {
      "mysql" | "my" => ("mysql", format!("mysql://{}", conn)),
      "postgres" | "pgsql" | "pg" | "postgresql" => ("postgres", format!("postgres://{}", conn)),
      "sqlite" => ("sqlite", format!("{}", conn)),
      _ => ("", "".to_string()),
    }
  } else {
    ("", "".to_string())
  }
}

pub async fn do_sql_action(mut action: SqlAction) -> Result<SqlActionResult> {
  let (schema, conn) = sql_driver(&action.connection);

  // normalize connection string
  action.connection = conn;

  match schema {
    "mysql" => mysql::do_sql_action(action).await,
    "postgres" => pgsql::do_sql_action(action).await,
    "sqlite" => sqlite::do_sql_action(action).await,
    _ => anyhow::bail!("Unsupported database driver"),
  }
}

fn array_dim(value: Option<&Value>) -> usize {
  if value.is_none() {
    return 0;
  }
  match value.unwrap() {
    Value::Array(a) => {
      let a = a.iter().all(|aa| aa.is_array());
      if a {
        2
      } else {
        1
      }
    }
    _ => 0,
  }
}
