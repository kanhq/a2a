use anyhow::Result;
use rust_xlsxwriter::{Format, FormatAlign, Workbook, XlsxError};
use serde_json::Value;

/// Convert JSON to Excel
pub fn json_to_excel(value: &Value) -> Result<Vec<u8>> {
  let (cells, headers) = super::table::json_to_cells(&value);
  let pretty_headers = super::table::plain_headers_to_nested(&headers);

  let mut workbook = Workbook::new();

  write_pretty_sheet(&mut workbook, &pretty_headers, &cells)?;
  // write_plain_sheet(&mut workbook, &headers, &cells)?;
  // write_json_sheet(&mut workbook, &text)?;

  workbook
    .save_to_buffer()
    .map_err(|e| XlsxError::from(e).into())
}

fn write_pretty_sheet(
  workbook: &mut Workbook,
  headers: &Vec<Vec<Option<String>>>,
  cells: &Vec<Vec<Value>>,
) -> Result<()> {
  let sheet = workbook.add_worksheet();
  sheet.set_name("Sheet1")?;

  let header_format = Format::new()
    .set_bold()
    .set_align(FormatAlign::Center)
    .set_align(FormatAlign::VerticalCenter)
    .set_font_size(9)
    .set_font_name("Microsoft YaHei");

  let row_format = Format::new()
    .set_font_name("Microsoft YaHei")
    .set_font_size(9);

  for r in 0..headers.len() {
    for c in 0..headers[r].len() {
      if headers[r][c].is_none() {
        continue;
      }
      let row_span = if r == headers.len() - 1 {
        1
      } else {
        (r + 1..headers.len())
          .take_while(|&i| headers[i][c].is_none())
          .count()
          + 1
      };
      let col_span = if r == headers.len() - 1 {
        1
      } else {
        (c + 1..headers[r].len())
          .take_while(|&i| headers[r][i].is_none())
          .count()
          + 1
      };

      if row_span > 1 {
        sheet.merge_range(
          r as u32,
          c as u16,
          r as u32 + row_span as u32 - 1,
          c as u16,
          headers[r][c].as_ref().map_or("", |v| v.as_str()),
          &header_format,
        )?;
      }
      if col_span > 1 {
        sheet.merge_range(
          r as u32,
          c as u16,
          r as u32,
          c as u16 + col_span as u16 - 1,
          headers[r][c].as_ref().map_or("", |v| v.as_str()),
          &header_format,
        )?;
      }
      if row_span == 1 && col_span == 1 {
        sheet.write_with_format(
          r as u32,
          c as u16,
          headers[r][c].as_ref().map_or("", |v| v.as_str()),
          &header_format,
        )?;
      }
    }
  }

  for r in 0..cells.len() {
    for c in 0..cells[r].len() {
      match &cells[r][c] {
        Value::Number(n) => {
          let n = n.as_f64().unwrap_or_default();
          sheet.write_number_with_format((r + headers.len()) as u32, c as u16, n, &row_format)?;
        }
        Value::String(s) => {
          sheet.write_with_format((r + headers.len()) as u32, c as u16, s, &row_format)?;
        }
        Value::Bool(b) => {
          sheet.write_boolean_with_format((r + headers.len()) as u32, c as u16, *b, &row_format)?;
        }
        _ => {
          continue;
        }
      }
    }
  }

  sheet.autofit();

  Ok(())
}

#[allow(dead_code)]
fn write_plain_sheet(
  workbook: &mut Workbook,
  headers: &Vec<String>,
  cells: &Vec<Vec<Value>>,
) -> Result<()> {
  let sheet = workbook.add_worksheet();
  sheet.set_name("PlainData")?;

  let header_format = Format::new()
    .set_bold()
    .set_align(FormatAlign::Center)
    .set_align(FormatAlign::VerticalCenter)
    .set_font_size(9)
    .set_font_name("Microsoft YaHei");

  let row_format = Format::new()
    .set_font_name("Microsoft YaHei")
    .set_font_size(9);

  for (c, header) in headers.iter().enumerate() {
    sheet.write_with_format(0, c as u16, header, &header_format)?;
  }

  for r in 0..cells.len() {
    for c in 0..cells[r].len() {
      match &cells[r][c] {
        Value::Number(n) => {
          let n = n.as_f64().unwrap_or_default();
          sheet.write_number_with_format((r + 1) as u32, c as u16, n, &row_format)?;
        }
        Value::String(s) => {
          sheet.write_with_format((r + 1) as u32, c as u16, s, &row_format)?;
        }
        Value::Bool(b) => {
          sheet.write_boolean_with_format((r + 1) as u32, c as u16, *b, &row_format)?;
        }
        _ => {
          continue;
        }
      }
    }
  }

  sheet.autofit();

  Ok(())
}

#[allow(dead_code)]
fn write_json_sheet(workbook: &mut Workbook, text: &str) -> Result<()> {
  let sheet = workbook.add_worksheet();
  sheet.set_name("JsonData")?;

  let format = Format::new()
    .set_font_name("Consolas")
    .set_font_size(9)
    .set_text_wrap();

  sheet.write_with_format(0, 0, text, &format)?;
  sheet.set_row_height(0, 800)?;
  sheet.set_column_width(0, 600)?;

  sheet.autofit();

  Ok(())
}
