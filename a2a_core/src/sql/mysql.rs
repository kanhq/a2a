use a2a_tojson::bytes_to_json;
use a2a_types::{SqlAction, SqlActionResult, Value};
use anyhow::Result;
use serde_json::json;
use sqlx::{
  mysql::{MySqlArguments, MySqlRow},
  Arguments, Column, Connection, MySqlConnection, Row, TypeInfo,
};

use super::array_dim;

pub(crate) async fn do_sql_action(action: SqlAction) -> Result<SqlActionResult> {
  let mut conn = MySqlConnection::connect(&action.connection).await?;
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

fn row_to_value(row: MySqlRow) -> Value {
  let mut val = serde_json::Map::new();
  row.columns().iter().enumerate().for_each(|(i, col)| {
    let value = match col.type_info().name() {
      "BOOL" | "BOOLEAN" => Value::Bool(row.get(i)),
      "TEXT" | "VARCHAR" | "CHAR" | "NAME" | "CITEXT" => Value::String(row.get(i)),
      "TINYINT" => {
        let n: i8 = row.get(i);
        json!(n)
      }
      "TINYINT UNSIGNED" => {
        let n: u8 = row.get(i);
        json!(n)
      }
      "SMALLINT" => {
        let n: i16 = row.get(i);
        json!(n)
      }
      "SMALLINT UNSIGNED" => {
        let n: u16 = row.get(i);
        json!(n)
      }
      "INT" => {
        let n: i32 = row.get(i);
        json!(n)
      }
      "INT UNSIGNED" => {
        let n: u32 = row.get(i);
        json!(n)
      }
      "BIGINT" => {
        let n: i64 = row.get(i);
        json!(n)
      }
      "BIGINT UNSIGNED" => {
        let n: u64 = row.get(i);
        json!(n)
      }
      "FLOAT" => {
        let n: f32 = row.get(i);
        json!(n)
      }
      "DOUBLE" => {
        let n: f64 = row.get(i);
        json!(n)
      }
      "VARBINARY" | "BINARY" | "BLOB" => {
        let data: Vec<u8> = row.get(i);
        bytes_to_json(bytes::Bytes::from(data), "", None).unwrap_or(Value::Null)
      }
      "JSON" => {
        let data: serde_json::Value = row.get(i);
        data
      }
      "DATETIME" => {
        let data: time::PrimitiveDateTime = row.get(i);
        json!(data.assume_utc().unix_timestamp_nanos() / 1_000_000)
      }
      "TIMESTAMP" => {
        let data: time::OffsetDateTime = row.get(i);
        json!(data.unix_timestamp_nanos() / 1_000_000)
      }
      "DATE" => {
        let data: time::Date = row.get(i);
        json!(data.to_string())
      }
      "TIME" => {
        let data: time::Time = row.get(i);
        json!(data.to_string())
      }
      _ => Value::Null,
    };
    val.insert(col.name().to_string(), value);
  });
  Value::Object(val)
}

fn value_to_args(val: Option<&Value>) -> Option<MySqlArguments> {
  match val {
    Some(Value::Array(a)) => {
      let mut args = MySqlArguments::default();
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
