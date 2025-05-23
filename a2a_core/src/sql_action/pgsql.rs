use a2a_tojson::bytes_to_json;
use a2a_types::{SqlAction, SqlActionResult, Value};
use anyhow::Result;
use serde_json::json;
use sqlx::{
  postgres::{PgArguments, PgRow},
  Arguments, Column, Connection, PgConnection, Row, TypeInfo,
};

use super::array_dim;

pub(crate) async fn do_sql_action(action: SqlAction) -> Result<SqlActionResult> {
  let mut conn = PgConnection::connect(&action.connection).await?;
  let sql = &mysql_syntax_to_pgsql(&action.query);
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
        let mut result = Vec::new();
        for row in a {
          if let Some(args) = value_to_args(Some(&row)) {
            let rows = sqlx::query_with(sql, args).fetch_all(&mut conn).await?;
            for row in rows {
              result.push(row_to_value(row));
            }
          }
        }
        Ok(SqlActionResult::Array(result))
      } else {
        anyhow::bail!("Unsupported bind parameters");
      }
    }
    _ => anyhow::bail!("Unsupported rows value"),
  }
}

fn row_to_value(row: PgRow) -> Value {
  let mut val = serde_json::Map::new();
  row.columns().iter().enumerate().for_each(|(i, col)| {
    let value = match col.type_info().name() {
      "BOOL" => Value::Bool(row.get(i)),
      "TEXT" | "VARCHAR" | "CHAR" | "NAME" | "CITEXT" => Value::String(row.get(i)),
      "SMALLINT" | "SMALLSERIAL" | "INT2" => {
        let n: i16 = row.get(i);
        json!(n)
      }
      "INT" | "SERIAL" | "INT4" => {
        let n: i32 = row.get(i);
        json!(n)
      }
      "BIGINT" | "BIGSERIAL" | "INT8" => {
        let n: i64 = row.get(i);
        json!(n)
      }
      "REAL" | "FLOAT4" => {
        let n: f32 = row.get(i);
        json!(n)
      }
      "DOUBLE PRECISION" | "FLOAT8" => {
        let n: f64 = row.get(i);
        json!(n)
      }
      "BYTEA" => {
        let data: Vec<u8> = row.get(i);
        bytes_to_json(bytes::Bytes::from(data), "", None).unwrap_or(Value::Null)
      }
      "JSON" | "JSONB" => {
        let data: serde_json::Value = row.get(i);
        data
      }
      "TIMESTAMP" => {
        let data: time::PrimitiveDateTime = row.get(i);
        json!(data.assume_utc().unix_timestamp_nanos() / 1_000_000)
      }
      "TIMESTAMPTZ" => {
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
      "HSTORE" => {
        let data: serde_json::Value = row.get(i);
        data
      }
      _ => Value::Null,
    };
    val.insert(col.name().to_string(), value);
  });
  Value::Object(val)
}

fn value_to_args(val: Option<&Value>) -> Option<PgArguments> {
  match val {
    Some(Value::Array(a)) => {
      let mut args = PgArguments::default();
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

fn mysql_syntax_to_pgsql(sql: &str) -> String {
  // replace ? to $1, $2, ...
  let mut sn = 1;
  let mut result = String::new();
  for c in sql.chars() {
    if c == '?' {
      result.push_str(&format!("${}", sn));
      sn += 1;
    } else {
      result.push(c);
    }
  }
  // replace "AUTO_INCREMENT" to "GENERATED BY DEFAULT AS IDENTITY" ignore case
  result = regex::Regex::new(r#"(?i)AUTO[\s_]?INCREMENT"#)
    .unwrap()
    .replace_all(&result, "GENERATED BY DEFAULT AS IDENTITY")
    .to_string();

  result
}
