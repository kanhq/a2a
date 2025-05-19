use std::vec;

use serde_json::Value;

/// calculate the columns and rows of the JSON object
/// columns is the number of JSON fields count, including nested objects expanded
/// rows is the max length of the nested Arrays
pub fn json_to_2d(value: &Value) -> (usize, usize) {
  match value {
    Value::Object(map) => {
      let mut columns = map.len();
      let mut rows = 1;
      for (_, v) in map {
        let (c, r) = json_to_2d(v);
        if c > 0 {
          columns += c - 1;
        } else {
          columns += c;
        }
        rows = rows.max(r);
      }
      (columns, rows)
    }
    Value::Array(array) => {
      let mut columns = 0;
      let mut rows = array.len();
      for v in array {
        let (c, r) = json_to_2d(v);
        columns = columns.max(c);
        rows = rows.max(r);
      }
      (columns, rows)
    }
    _ => (0, 0),
  }
}

/// convert JSON to cells and headers
pub fn json_to_cells(value: &Value) -> (Vec<Vec<Value>>, Vec<String>) {
  let (columns, rows) = json_to_2d(value);

  let mut cells = vec![vec![Value::Null; columns]; rows];
  let mut headers = vec![];

  let parent = String::default();

  fill_cells(&mut cells, &mut headers, true, value, 0, 0, parent);

  (cells, headers)
}

fn fill_cells(
  cells: &mut Vec<Vec<Value>>,
  headers: &mut Vec<String>,
  change_header: bool,
  value: &Value,
  row: usize,
  col: usize,
  parent: String,
) {
  match value {
    Value::Object(map) => {
      let mut i = 0;
      for (k, v) in map {
        let path = if parent.is_empty() {
          k.clone()
        } else {
          format!("{}.{}", parent, k)
        };
        if change_header
          && (!v.is_object()
            || (v.is_object() && v.as_object().map(|m| m.len()).unwrap_or_default() == 0))
          && (!v.is_array()
            || (v.is_array() && v.as_array().map(|a| a.len()).unwrap_or_default() == 0))
        {
          headers.push(path.clone());
        }
        fill_cells(cells, headers, true, v, row, col + i, path);
        if v.is_object() {
          i += v.as_object().map(|m| m.len()).unwrap_or(1);
        } else if v.is_array() {
          i += v
            .as_array()
            .and_then(|v| v.first())
            .and_then(|v| v.as_object().map(|m| m.len()))
            .unwrap_or(1);
        } else {
          i += 1;
        }
      }
    }
    Value::Array(array) => {
      for (i, v) in array.iter().enumerate() {
        fill_cells(cells, headers, i == 0, v, row + i, col, parent.clone());
      }
    }
    _ => {
      cells[row][col] = value.clone();
    }
  }
}

pub fn plain_headers_to_nested(headers: &[String]) -> Vec<Vec<Option<String>>> {
  let splitted = headers
    .iter()
    .map(|h| h.split('.').collect::<Vec<&str>>())
    .collect::<Vec<Vec<&str>>>();

  let depth = splitted.iter().map(|v| v.len()).max().unwrap_or(1);

  let mut result = vec![vec![None; headers.len()]; depth];

  for (i, h) in splitted.iter().enumerate() {
    for (j, v) in h.iter().enumerate() {
      result[j][i] = Some(v.to_string());
    }
  }

  let last_id = result.len() - 1;
  result.iter_mut().enumerate().for_each(|(i, v)| {
    if i != last_id {
      let mut j = 0;
      for k in 1..v.len() {
        if v[k].eq(&v[j]) {
          v[k] = None;
        } else {
          j = k;
        }
      }
    }
  });

  result
}
