use anyhow::Result;

pub mod excel;
pub mod html;
pub(crate) mod table;

pub use excel::json_to_excel;
pub use html::json_to_html;
use serde_json::Value;

pub fn is_plain_2d(value: &Value) -> bool {
  // Check if the value is an array of array
  if let Some(array) = value.as_array() {
    array.iter().all(|v| v.is_array())
  } else {
    false
  }
}

/// Convert JSON to CSV
pub fn json_to_csv(value: &Value) -> Result<String> {
  let (cells, headers) = table::json_to_cells(&value);

  let mut result = headers.join("\t");
  result.push('\n');
  cells.iter().for_each(|row| {
    let line = row
      .iter()
      .map(|v| {
        if v.is_null() {
          "".to_string()
        } else {
          v.to_string()
        }
      })
      .collect::<Vec<String>>()
      .join("\t");
    result.push_str(&line);
    result.push('\n');
  });

  Ok(result)
}
