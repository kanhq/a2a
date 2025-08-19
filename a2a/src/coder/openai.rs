use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletionChunk {
  pub id: String,
  pub object: String,
  pub created: u64,
  pub model: String,
  pub choices: Vec<ChatCompletionChoice>,
  pub usage: ChatCompletionUsage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletionChoice {
  pub delta: ChatCompletionChoiceDelta,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub finish_reason: Option<String>,
  index: i64,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub logprobs: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletionChoiceDelta {
  pub role: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub content: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub refusal: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tool_calls: Option<Vec<ChatCompletionChoiceDeltaToolCall>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletionChoiceDeltaToolCall {
  pub index: i64,
  pub id: String,
  #[serde(rename = "type")]
  pub type_: String,
  pub function: FunctionCall,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionCall {
  pub name: String,
  pub arguments: String,
}
