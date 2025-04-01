mod sse_server;

use std::sync::Arc;

use a2a_types::Value;
use rmcp::{
  model::{
    CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
  },
  tool, ServerHandler,
};
pub(crate) use sse_server::{McpState, SseServer, SseServerConfig};

use crate::run::execute_js_code;

use super::AppState;

#[derive(Clone)]
pub struct A2AMcp {
  pub(crate) state: Arc<AppState>,
}

#[tool(tool_box)]
impl A2AMcp {
  pub fn new(state: Arc<AppState>) -> Self {
    Self { state }
  }

  #[tool(
    description = r#"run some javascript source, the result will be returned as JSON with two fields: `result` and `logs`

    - `result`: a json object that contains the result of the script execution, use it as what user expected
    - `logs`: a string array with the logs of the script execution, use it as needed
    "#
  )]
  async fn a2a_run(
    &self,
    #[tool(param)]
    #[schemars(description = "the source code to run")]
    script: String,
  ) -> Result<CallToolResult, rmcp::Error> {
    let result: Value = self
      .a2a_run_impl(script)
      .await
      .map_err(|e| rmcp::Error::internal_error(format!("a2a_run failed: {}", e), None))?;
    Ok(CallToolResult::success(vec![Content::json(result)?]))
  }

  async fn a2a_run_impl(&self, script: String) -> anyhow::Result<Value> {
    let params = Value::Null;
    let clean_up = None;
    let result = execute_js_code(&script, &self.state.conf, &params, clean_up).await?;

    Ok(result)
  }
}

#[tool(tool_box)]
impl ServerHandler for A2AMcp {
  fn get_info(&self) -> ServerInfo {
    ServerInfo {
      protocol_version: ProtocolVersion::V_2024_11_05,
      capabilities: ServerCapabilities::builder().enable_tools().build(),
      server_info: Implementation::from_build_env(),
      instructions: Some(
        "This server provides `a2a_run` tools to run a javascript script.".to_string(),
      ),
    }
  }
}
