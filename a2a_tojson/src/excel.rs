use a2a_types::Value;
use anyhow::Result;
use calamine::{Data, Range, Reader};

use crate::json2excel::{is_plain_2d, json_to_excel};

pub(crate) fn to_json(input: String, options: Option<&Value>) -> Result<Value> {
  // first row is header
  let has_header = options
    .and_then(|o| o.get("has_header"))
    .and_then(|v| v.as_bool())
    .unwrap_or(true);

  // user provided headers
  let headers = options
    .and_then(|o| o.get("headers"))
    .and_then(|v| v.as_array())
    .map(|a| a.iter().map(|v| v.to_string()).collect::<Vec<String>>());

  let sheet = options
    .and_then(|o| o.get("sheet"))
    .and_then(|v| v.as_str())
    .unwrap_or("Sheet1");

  let mut workbook =
    calamine::open_workbook_auto(input.clone()).map_err(|err| anyhow::anyhow!(err))?;

  let range = workbook
    .worksheet_range(sheet)
    .map_err(|err| anyhow::anyhow!(err))?;

  if has_header || headers.is_some() {
    to_json_object(range, has_header, headers)
  } else {
    to_json_map(&range)
  }
}

fn excel_col_name(col: usize) -> String {
  let mut name = String::new();
  let mut n = col;
  while n > 0 {
    n -= 1;
    name.insert(0, ((n % 26) as u8 + b'A') as char);
    n /= 26;
  }
  name
}

fn excel_value(value: &Data) -> Value {
  match value {
    Data::Bool(b) => Value::Bool(*b),
    Data::Error(e) => Value::String(format!("#ERROR: {}", e)),
    Data::Float(f) => Value::Number(serde_json::Number::from_f64(*f).unwrap()),
    Data::Int(i) => Value::Number(serde_json::Number::from_f64(*i as f64).unwrap()),
    Data::String(s) => Value::String(s.to_string()),
    Data::Empty => Value::Null,
    _ => Value::String(value.to_string()),
  }
}

fn to_json_map(range: &Range<Data>) -> Result<Value> {
  let mut map = serde_json::Map::new();
  range.rows().enumerate().for_each(|(i, r)| {
    r.iter().enumerate().for_each(|(j, c)| {
      map.insert(format!("{}{}", excel_col_name(j), i + 1), excel_value(c));
    });
  });
  Ok(Value::Object(map))
}

fn to_json_object(
  range: Range<Data>,
  has_header: bool,
  headers: Option<Vec<String>>,
) -> Result<Value> {
  let mut records = Vec::new();
  let headers = headers.unwrap_or_else(|| {
    range
      .rows()
      .next()
      .map(|r| r.iter().map(|c| c.to_string()).collect())
      .unwrap_or_default()
  });

  for row in range.rows().skip(if has_header { 1 } else { 0 }) {
    let mut obj = serde_json::Map::new();
    for (i, field) in row.iter().enumerate() {
      obj.insert(headers[i].to_string(), excel_value(field));
    }
    records.push(Value::Object(obj));
  }

  Ok(Value::Array(records))
}

pub(crate) fn to_mimetype_bytes(input: &Value) -> Result<bytes::Bytes> {
  if input.is_string() {
    return Ok(bytes::Bytes::from(input.as_str().unwrap().to_string()));
  }

  if is_plain_2d(input) {
    let mut wtr = csv::Writer::from_writer(vec![]);
    for item in input.as_array().unwrap() {
      if let Value::Array(arr) = item {
        wtr.write_record(arr.iter().map(|v| v.to_string()))?;
      }
    }
    return Ok(wtr.into_inner()?.into());
  }

  json_to_excel(input).map(|s| s.into())
}
