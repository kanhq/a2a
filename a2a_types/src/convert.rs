use serde_json::Value;

use crate::{HttpActionResult, WebSearchResult};

impl From<HttpActionResult> for Value {
  fn from(action: HttpActionResult) -> Self {
    serde_json::to_value(action).unwrap()
  }
}

impl From<WebSearchResult> for Value {
  fn from(action: WebSearchResult) -> Self {
    serde_json::to_value(action).unwrap()
  }
}
