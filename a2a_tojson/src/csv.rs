use crate::json2excel::{is_plain_2d, json_to_csv};
use crate::utils::json_typed;
use crate::{Result, Value};

pub(crate) fn to_json(input: String, options: Option<&Value>) -> Result<Value> {
  let has_header = options
    .and_then(|o| o.get("has_header"))
    .and_then(|v| v.as_bool())
    .unwrap_or(true);
  let delimiter = options
    .and_then(|o| o.get("delimiter"))
    .and_then(|v| v.as_str())
    .and_then(|s| s.as_bytes().first())
    .and_then(|b| Some(*b))
    .unwrap_or(b',');
  let as_object = options
    .and_then(|o| o.get("as_object"))
    .and_then(|v| v.as_bool())
    .unwrap_or(true);

  // no header and as_object is not supported
  let as_object = has_header && as_object;

  let rdr = csv::ReaderBuilder::new()
    .has_headers(has_header)
    .delimiter(delimiter)
    .from_reader(input.as_bytes());

  if as_object {
    to_json_object(rdr)
  } else {
    to_json_array(rdr)
  }
}

fn to_json_array(mut rdr: csv::Reader<&[u8]>) -> Result<Value> {
  let mut records = Vec::new();
  if rdr.has_headers() {
    records.push(Value::Array(
      rdr
        .headers()?
        .iter()
        .map(|s| json_typed(s.to_string()))
        .collect(),
    ));
  }
  for result in rdr.records() {
    let record = result?;
    records.push(Value::Array(
      record.iter().map(|s| json_typed(s.to_string())).collect(),
    ));
  }
  Ok(Value::Array(records))
}

fn to_json_object(mut rdr: csv::Reader<&[u8]>) -> Result<Value> {
  let mut records = Vec::new();
  let headers = rdr.headers()?.clone();
  for result in rdr.records() {
    let record = result?;
    let mut obj = serde_json::Map::new();
    for (i, field) in record.iter().enumerate() {
      obj.insert(headers[i].to_string(), json_typed(field.to_string()));
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

  json_to_csv(input).map(|s| s.into())
}
