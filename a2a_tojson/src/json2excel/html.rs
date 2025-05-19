use anyhow::Result;
use serde_json::Value;
use std::fmt;

/// Convert JSON to html table, with the given table class
/// th will be used for the headers
/// colspan and rowspan in th will be calculated
pub fn json_to_html(value: &Value, table_class: &str) -> Result<String> {
  let (cells, headers) = super::table::json_to_cells(&value);

  let mut w = String::new();

  let headers = super::table::plain_headers_to_nested(&headers);

  // headers.iter().for_each(|row| {
  //   row.iter().for_each(|v| {
  //     print!("{:?}\t", v);
  //   });
  //   println!("");
  // });

  fmt::write(&mut w, format_args!("<table class=\"{}\">\n", table_class))?;

  for r in 0..headers.len() {
    fmt::write(&mut w, format_args!("  <tr>\n"))?;
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

      fmt::write(&mut w, format_args!("    <th",))?;
      if row_span > 1 {
        fmt::write(&mut w, format_args!(" rowspan=\"{}\"", row_span))?;
      }
      if col_span > 1 {
        fmt::write(&mut w, format_args!(" colspan=\"{}\"", col_span))?;
      }
      fmt::write(
        &mut w,
        format_args!(
          ">{}</th>\n",
          headers[r][c].as_ref().map(|v| v.as_str()).unwrap_or("")
        ),
      )?;
    }
    fmt::write(&mut w, format_args!("  </tr>\n"))?;
  }

  for r in 0..cells.len() {
    fmt::write(&mut w, format_args!("  <tr>\n"))?;
    for c in 0..cells[r].len() {
      let text = if cells[r][c].is_null() {
        "".to_string()
      } else {
        cells[r][c].to_string()
      };
      fmt::write(
        &mut w,
        format_args!("    <td>{}</td>\n", text.trim_matches('"')),
      )?;
    }
    fmt::write(&mut w, format_args!("  </tr>\n"))?;
  }

  w.push_str("</table>\n");
  Ok(w)
}
