use anyhow::{bail, Result};
use rmcp::{
  model::{CallToolRequestParam, Tool},
  service::RunningService,
  transport::{SseClientTransport, StreamableHttpClientTransport},
  RoleClient, ServiceExt,
};
use serde_json::{json, Value};

struct McpClient {
  service: RunningService<RoleClient, ()>,
  tools: Vec<Tool>,
}

impl McpClient {
  pub async fn new(conn_str: &str) -> Result<Self> {
    if conn_str.starts_with("http") {
      if conn_str.ends_with("/sse") {
        let tr = SseClientTransport::start(conn_str.to_owned()).await?;
        let service = ().serve(tr).await?;
        let tools = service.list_tools(None).await?;
        Ok(Self {
          service,
          tools: tools.tools,
        })
      } else {
        let tr = StreamableHttpClientTransport::from_uri(conn_str);
        let service = ().serve(tr).await?;
        let tools = service.list_tools(None).await?;
        Ok(Self {
          service,
          tools: tools.tools,
        })
      }
    } else {
      bail!("Unsupported connection string: {}", conn_str);
    }
  }

  pub fn list_tools(&self) -> Result<Value> {
    let tools = self
      .tools
      .iter()
      .map(|tool| {
        json!(
          {
            "type": "function",
            "function": {
              "name": tool.name,
              "description": tool.description,
              "parameters": tool.input_schema,
            }
          }
        )
      })
      .collect::<Vec<_>>();
    Ok(Value::Array(tools))
  }

  pub async fn call_tool(&self, tool_name: &str, args: Option<Value>) -> Result<String> {
    let resp = self
      .service
      .call_tool(CallToolRequestParam {
        name: tool_name.to_string().into(),
        arguments: args.and_then(|v| v.as_object().cloned()),
      })
      .await?;
    if resp.is_error.unwrap_or(false) {
      bail!(
        "Tool call failed: {}",
        resp
          .content
          .map(|a| a
            .iter()
            .filter_map(|r| r.as_text())
            .fold(String::new(), |mut f, r| {
              f.push_str(&r.text);
              f
            }))
          .unwrap_or("Unknown error".to_string())
      );
    }
    let result = resp
      .content
      .map(|a| {
        let a =
          a.into_iter()
            .filter_map(|r| r.as_text().cloned())
            .fold(String::new(), |mut a, r| {
              a.push('\n');
              a.push_str(&r.text);
              a
            });
        a
      })
      .unwrap_or_default();
    Ok(result)
  }

  pub fn is_my_tool(&self, tool_name: &str) -> bool {
    self.tools.iter().any(|tool| tool.name == tool_name)
  }
}

pub struct McpClientSet {
  clients: Vec<McpClient>,
}

impl McpClientSet {
  pub async fn new<S: AsRef<str>>(urls: &[S]) -> Self {
    let mut clients = Vec::new();
    for url in urls {
      if let Ok(client) = McpClient::new(url.as_ref()).await {
        clients.push(client);
      }
    }
    Self { clients }
  }

  pub fn list_tools(&self) -> Result<Value> {
    let mut tools = Vec::new();
    for client in &self.clients {
      let client_tools = client.list_tools()?;
      if let Value::Array(arr) = client_tools {
        tools.extend(arr);
      }
    }
    if tools.is_empty() {
      Ok(Value::Null)
    } else {
      Ok(Value::Array(tools))
    }
  }

  pub async fn call_tool(&self, tool_name: &str, args: Option<Value>) -> Result<String> {
    for client in &self.clients {
      if client.is_my_tool(tool_name) {
        return client.call_tool(tool_name, args).await;
      }
    }
    Err(anyhow::anyhow!("No suitable client found"))
  }
}
