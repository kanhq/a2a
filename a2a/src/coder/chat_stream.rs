use std::{
  collections::HashMap,
  pin::Pin,
  task::{Context, Poll},
};

use anyhow::Result;
use futures::{Stream, TryStreamExt};
use serde_json::{json, Value};
use tokio::io::AsyncBufReadExt;
use tokio_util::io::StreamReader;
use tracing::{debug, error};

use super::{
  mcps::McpClientSet,
  openai::{ChatCompletionChoiceDeltaToolCall, ChatCompletionChunk},
  WriteCode,
};

type ChatStreamReceiver = tokio::sync::mpsc::Receiver<Result<String>>;
type ChatStreamSender = tokio::sync::mpsc::Sender<Result<String>>;

pub struct ChatStream {
  pub rx: ChatStreamReceiver,
}

impl Stream for ChatStream {
  type Item = String;

  fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
    let this = self.get_mut();
    let data = this.rx.poll_recv(cx);
    debug!(?data, "polling chat stream");
    match data {
      Poll::Ready(Some(Ok(msg))) => Poll::Ready(Some(msg)),
      Poll::Ready(Some(Err(_))) => Poll::Ready(None),
      Poll::Ready(None) => Poll::Ready(None),
      Poll::Pending => Poll::Pending,
    }
  }
}

impl ChatStream {
  pub fn new(config: WriteCode) -> Self {
    let (tx, rx) = tokio::sync::mpsc::channel(32);
    tokio::spawn(Self::start_chat_stream(tx, config));
    Self { rx }
  }

  async fn start_chat_stream(tx: ChatStreamSender, config: WriteCode) -> Result<()> {
    let client = reqwest::Client::new();
    let mut messages = vec![
      serde_json::json!({
        "role": "system",
        "content": config.system,
      }),
      serde_json::json!({
        "role": "user",
        "content": config.user,
      }),
    ];

    let mcp_clients = McpClientSet::new(&config.mcp_list).await;

    let tools = mcp_clients.list_tools()?;

    loop {
      let mut tool_calls: HashMap<String, ChatCompletionChoiceDeltaToolCall> = HashMap::new();
      let req = json!({
        "model": config.model,
        "messages": messages,
        "stream": true,
        "stream_options": {
          "include_usage": true,
        },
        "tools": tools
      });

      debug!(req = serde_json::to_string(&req).unwrap(), "llm_request");

      let response = client
        .post(format!("{}/chat/completions", config.base_url))
        .bearer_auth(&config.api_key)
        .json(&req)
        .send()
        .await?;

      if !response.status().is_success() {
        tx.send(Err(anyhow::anyhow!(
          "Failed to call API: {}",
          response.status()
        )))
        .await?;
        break;
      }

      let stream = response
        .bytes_stream()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e));

      let reader = StreamReader::new(stream);
      let mut lines = reader.lines();

      let mut last_tool_call_id = String::new();
      while let Some(line) = lines.next_line().await? {
        if line.is_empty() {
          continue;
        }
        if line.starts_with("data: ") {
          let data = &line[6..];
          if data == "[DONE]" {
            break;
          }
          let chunk = match serde_json::from_str::<ChatCompletionChunk>(data) {
            Ok(chunk) => chunk,
            Err(err) => {
              error!(?err, data, "Failed to parse chunk");
              break;
            }
          };

          for choice in chunk.choices {
            for tool_call in choice.delta.tool_calls.unwrap_or_default() {
              if !tool_call.id.is_empty() {
                last_tool_call_id = tool_call.id.clone();
              }
              if tool_calls.contains_key(&last_tool_call_id) {
                tool_calls
                  .get_mut(&last_tool_call_id)
                  .unwrap()
                  .function
                  .arguments
                  .push_str(&tool_call.function.arguments);
              } else {
                tool_calls.insert(tool_call.id.clone(), tool_call);
              }
            }
          }
        }
        tx.send(Ok(line)).await?;
      }

      if tool_calls.is_empty() {
        // no tool calls need to be made, all messages can be sent
        break;
      }

      // build tool call requests
      let mut tool_calls_messages = Vec::new();
      let mut assistant_tool_calls = Vec::new();
      for (id, tool_call) in tool_calls {
        let args: Value = serde_json::from_str(&tool_call.function.arguments)?;
        let content = mcp_clients
          .call_tool(&tool_call.function.name, Some(args))
          .await?;

        assistant_tool_calls.push(json!({
          "id": id,
          "type": "function",
          "function": {
            "name": tool_call.function.name,
            "arguments": tool_call.function.arguments,
          }
        }));

        tool_calls_messages.push(json!({
          "role": "tool",
          "tool_call_id": id,
          "content": content,
        }));
      }
      messages.push(json!({
        "role": "assistant",
        "tool_calls": assistant_tool_calls,
      }));
      messages.extend(tool_calls_messages);
    }

    Ok(())
  }
}
