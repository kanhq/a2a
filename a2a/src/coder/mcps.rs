use anyhow::{bail, Result};
use rmcp::{
  model::{
    CallToolRequestParams, ClientCapabilities, ClientInfo, Implementation,
    InitializeRequestParams, Tool,
  },
  service::RunningService,
  transport::StreamableHttpClientTransport,
  RoleClient, ServiceExt,
};
use serde_json::{json, Value};
use tracing::debug;

struct McpClient {
  service: RunningService<RoleClient, InitializeRequestParams>,
  tools: Vec<Tool>,
}

impl McpClient {
  pub async fn new(conn_str: &str) -> Result<Self> {
    if conn_str.starts_with("http") {
      let tr = StreamableHttpClientTransport::from_uri(conn_str);
      let client_info = ClientInfo::new(
        ClientCapabilities::default(),
        Implementation::new("a2a-client", "0.1.0"),
      );
      let client = client_info.serve(tr).await?;

      let server_info = client.peer_info();
      debug!("server info: {server_info:?}");

      let tools = client.list_tools(None).await?;
      Ok(Self {
        service: client,
        tools: tools.tools,
      })
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
    let mut params = CallToolRequestParams::new(tool_name.to_string());
    if let Some(args) = args.as_ref().and_then(|a| a.as_object()) {
      params = params.with_arguments(args.clone());
    }

    let resp = self.service.call_tool(params).await?;
    if resp.is_error.unwrap_or(false) {
      bail!(
        "Tool call failed: {}",
        resp
          .content
          .iter()
          .filter_map(|r| r.as_text())
          .fold(String::new(), |mut f, r| {
            f.push_str(&r.text);
            f
          })
      );
    }
    let result = resp
      .content
      .iter()
      .flat_map(|r| r.as_text())
      .fold(String::new(), |mut a, r| {
        a.push('\n');
        a.push_str(&r.text);
        a
      });
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
