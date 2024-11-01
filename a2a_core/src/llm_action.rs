use a2a_types::{LlmAction, LlmActionResult, Value};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
struct LlmConnection {
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

  let body = response.text().await?;
  debug!(?body, "llm_response");
  let body = serde_json::from_str::<SimpleLlmResponse>(&body)?;
  let body = if let Some(mut choices) = body.choices {
    choices.pop().map(|c| c.message.content)
  } else {
    if let Some(err) = body.error {
      if let Some(message) = err.get("message") {
        return Err(anyhow::anyhow!("{}", message));
      } else {
        return Err(anyhow::anyhow!("failed"));
      }
    } else {
      return Err(anyhow::anyhow!("No response"));
    }
  };
  if is_json {
    let body = extract_json(body)?;
    serde_json::from_str(&body).map_err(|e| e.into())
  } else {
    let body = body.unwrap_or_default();
    Ok(serde_json::Value::String(body))
  }
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct SimpleLlmRequest {
  model: String,
  temperature: Option<f32>,
  messages: Vec<Message>,
  response_format: Option<ResponseFormat>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SimpleLlmResponse {
  error: Option<Value>,
  choices: Option<Vec<Choice>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Choice {
  message: MessageS,
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
  };

  reqwest::Client::new()
    .request(reqwest::Method::POST, &connection.url)
    .header("x-portkey-provider", &connection.provider)
    .header("content-type", "application/json")
    .bearer_auth(&connection.key)
    .json(&body)
    .build()
    .map_err(|e| e.into())
}

fn extract_json(body: Option<String>) -> Result<String> {
  match body {
    Some(body) => match body.find('{') {
      Some(start) => {
        let end = body.rfind('}').ok_or(anyhow::anyhow!("Invalid JSON"))?;
        Ok(body[start..=end].to_string())
      }
      None => Ok(body),
    },
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
