use a2a_tojson::bytes_to_json;
use a2a_types::{SqlAction, SqlActionResult, Value};
use anyhow::Result;
use serde_json::json;
use sqlx::{
  sqlite::{SqliteArguments, SqliteConnectOptions, SqliteRow},
  Arguments, Column, Connection, Row, SqliteConnection, TypeInfo,
};
use tracing::debug;

use super::array_dim;

pub(crate) async fn do_sql_action(action: SqlAction) -> Result<SqlActionResult> {
  let options = SqliteConnectOptions::new()
    .filename(&action.connection)
    .create_if_missing(true);

  let mut conn = SqliteConnection::connect_with(&options).await?;
  let sql = &action.query;
  match array_dim(action.rows.as_ref()) {
    0 => {
      // no bind parameters
      let rows = sqlx::query(sql).fetch_all(&mut conn).await?;
      let mut result = Vec::new();
      for row in rows {
        result.push(row_to_value(row));
      }
      Ok(SqlActionResult::Array(result))
    }
    1 => {
      if let Some(args) = value_to_args(action.rows.as_ref()) {
        let rows = sqlx::query_with(sql, args).fetch_all(&mut conn).await?;
        let mut result = Vec::new();
        for row in rows {
          result.push(row_to_value(row));
        }
        Ok(SqlActionResult::Array(result))
      } else {
        anyhow::bail!("Unsupported bind parameters");
      }
    }
    2 => {
      if let Some(Value::Array(a)) = action.rows {
        for row in a {
          if let Some(args) = value_to_args(Some(&row)) {
            sqlx::query_with(sql, args).execute(&mut conn).await?;
          }
        }
      }
      Ok(SqlActionResult::Array(Vec::new()))
    }
    _ => anyhow::bail!("Unsupported rows value"),
  }
}

fn row_to_value(row: SqliteRow) -> Value {
  let mut val = serde_json::Map::new();
  row.columns().iter().enumerate().for_each(|(i, col)| {
    debug!(
      col_name = col.name(),
      col_type = col.type_info().name(),
      "Processing column"
    );
    let value = match col.type_info().name() {
      "BOOLEAN" => Value::Bool(row.get(i)),
      "TEXT" | "VARCHAR" | "CHAR" | "NAME" | "CITEXT" => Value::String(row.get(i)),
      "INTEGER" | "INT4" | "INT8" => {
        let n: i64 = row.get(i);
        json!(n)
      }
      "REAL" => {
        let n: f64 = row.get(i);
        json!(n)
      }
      "BLOB" => {
        let data: Vec<u8> = row.get(i);
        bytes_to_json(bytes::Bytes::from(data), "", None).unwrap_or(Value::Null)
      }
      _ => {
        if let Ok(s) = row.try_get::<String, usize>(i) {
          Value::String(s)
        } else if let Ok(n) = row.try_get::<i64, usize>(i) {
          json!(n)
        } else if let Ok(n) = row.try_get::<f64, usize>(i) {
          json!(n)
        } else if let Ok(b) = row.try_get::<bool, usize>(i) {
          Value::Bool(b)
        } else if let Ok(data) = row.try_get::<Vec<u8>, usize>(i) {
          bytes_to_json(bytes::Bytes::from(data), "", None).unwrap_or(Value::Null)
        } else {
          Value::Null
        }
      }
    };
    val.insert(col.name().to_string(), value);
  });
  Value::Object(val)
}

fn value_to_args(val: Option<&Value>) -> Option<SqliteArguments<'_>> {
  match val {
    Some(Value::Array(a)) => {
      let mut args = SqliteArguments::default();
      for v in a {
        let _ = match v {
          Value::Bool(b) => args.add(*b),
          Value::String(s) => args.add(s.as_str()),
          Value::Number(n) => {
            if n.is_f64() {
              args.add(n.as_f64().unwrap_or_default())
            } else {
              args.add(n.as_i64().unwrap_or_default())
            }
          }
          _ => Ok(()),
        };
      }
      Some(args)
    }
    _ => None,
  }
}
