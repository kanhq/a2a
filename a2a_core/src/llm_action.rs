use a2a_types::{LlmAction, LlmActionResult, Value};
use anyhow::Result;
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncBufReadExt;
use tracing::{debug, trace, warn};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub(crate) struct LlmConnection {
  pub url: String,
  pub key: String,
  pub provider: String,
  pub model: String,
  pub temperature: Option<f32>,
}

pub async fn do_action(action: LlmAction) -> Result<LlmActionResult> {
  let connection: LlmConnection = action
    .connection
    .clone()
    .and_then(|c| serde_json::from_value(c).ok())
    .ok_or(anyhow::anyhow!("Invalid connection"))?;

  let is_json = action
    .override_result_mimetype
    .as_ref()
    .map(|m| m == "application/json")
    .unwrap_or(false);

  let client = reqwest::Client::builder();

  let req = to_request(&action, &connection, is_json)?;

  let response = client.build()?.execute(req).await?;

  debug!(?response, "llm response");
  let mut body = String::default();
  if !response.status().is_success() {
    return Err(anyhow::anyhow!("Request failed: {}", response.status()));
  }
  response.error_for_status_ref()?;
  let stream = response
    .bytes_stream()
    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e));
  let mut read = tokio_util::io::StreamReader::new(stream).lines();

  while let Some(chunk) = read.next_line().await? {
    let chunk = chunk.trim();
    if chunk.starts_with("data:") {
      let chunk = chunk.strip_prefix("data:").unwrap().trim();
      if chunk.is_empty() {
        continue;
      }
      if chunk.eq("[DONE]") {
        break;
      }
      match serde_json::from_str::<SimpleLlmResponse>(chunk) {
        Ok(data) => {
          let c = data
            .choices
            .as_ref()
            .and_then(|c| c.get(0))
            .and_then(|c| c.delta.as_ref())
            .map(|c| c.content.as_str());
          if let Some(c) = c {
            trace!(?c, "stream chunk");
            body.push_str(c);
          }
        }
        Err(err) => {
          warn!(?err, chunk, "Failed to parse response");
        }
      }
    }
  }

  if is_json {
    let body = extract_json(Some(body))?;
    serde_json::from_str(&body).map_err(|e| e.into())
  } else {
    Ok(serde_json::Value::String(body))
  }
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct SimpleLlmRequest {
  model: String,
  temperature: Option<f32>,
  messages: Vec<Message>,
  response_format: Option<ResponseFormat>,
  stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct SimpleLlmResponse {
  error: Option<Value>,
  choices: Option<Vec<Choice>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Choice {
  message: Option<MessageS>,
  delta: Option<MessageS>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum Message {
  S(MessageS),
  M(MessageM),
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct MessageM {
  role: String,
  content: Vec<MessageContent>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct MessageS {
  role: String,
  content: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
enum MessageContent {
  Text { text: String },
  ImageUrl { url: String, detail: Option<String> },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
enum ResponseFormat {
  Text,
  Json,
  JsonSchema { json_schema: serde_json::Value },
}

fn to_request(
  action: &LlmAction,
  connection: &LlmConnection,
  is_json: bool,
) -> Result<reqwest::Request> {
  let response_format = if is_json {
    Some(ResponseFormat::Json)
  } else {
    None
  };

  let mut messages = vec![];

  if let Some(prompt) = action.sys_prompt.as_ref() {
    messages.push(Message::S(MessageS::with_role("system").with_text(prompt)));
  }
  if let Some(image) = action.user_image.as_ref() {
    let mut user_message = MessageM::with_role("user").with_image_url(image, None);
    if let Some(prompt) = action.user_prompt.as_ref() {
      user_message = user_message.with_text(prompt);
    }
    messages.push(Message::M(user_message));
  } else {
    if let Some(prompt) = action.user_prompt.as_ref() {
      messages.push(Message::S(MessageS::with_role("user").with_text(prompt)));
    }
  }

  let body = SimpleLlmRequest {
    model: connection.model.clone(),
    temperature: connection.temperature,
    messages,
    response_format,
    stream: true,
  };

  reqwest::Client::new()
    .request(reqwest::Method::POST, &connection.url)
    .header("x-portkey-provider", &connection.provider)
    .header("x-portkey-model", &connection.model)
    .header("content-type", "application/json")
    .bearer_auth(&connection.key)
    .json(&body)
    .build()
    .map_err(|e| e.into())
}

fn extract_json(body: Option<String>) -> Result<String> {
  match body {
    Some(body) => {
      let first = body.find(|c| c == '{' || c == '[');
      let last = body.rfind(|c| c == '}' || c == ']');
      match (first, last) {
        (Some(start), Some(end)) => Ok(body[start..=end].to_string()),
        _ => Ok(body),
      }
    }
    None => Ok("{}".to_string()),
  }
}

impl MessageM {
  fn with_role(role: &str) -> MessageM {
    MessageM {
      role: role.to_string(),
      content: vec![],
    }
  }

  fn with_text(mut self, text: &str) -> Self {
    self.content.push(MessageContent::Text {
      text: text.to_string(),
    });
    self
  }

  fn with_image_url(mut self, url: &str, detail: Option<&str>) -> Self {
    self.content.push(MessageContent::ImageUrl {
      url: url.to_string(),
      detail: detail.map(|s| s.to_string()),
    });
    self
  }
}

impl MessageS {
  fn with_role(role: &str) -> MessageS {
    MessageS {
      role: role.to_string(),
      content: "".to_string(),
    }
  }
  fn with_text(mut self, text: &str) -> Self {
    self.content = text.to_string();
    self
  }
}
