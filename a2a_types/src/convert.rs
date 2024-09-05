use serde_json::Value;

use crate::HttpActionResult;

impl From<HttpActionResult> for Value {
  fn from(action: HttpActionResult) -> Self {
    serde_json::to_value(action).unwrap()
  }
}
