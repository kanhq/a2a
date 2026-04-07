// mod sse_server;

use std::sync::Arc;

use a2a_types::Value;
use rmcp::{
  handler::server::{tool::ToolRouter, wrapper::Parameters},
  model::{
    CallToolResult, Content, GetPromptRequestParam, GetPromptResult, Implementation,
    ListPromptsResult, PaginatedRequestParam, PaginatedRequestParams, Prompt, PromptMessage,
    PromptMessageContent, PromptMessageRole, ProtocolVersion, ServerCapabilities, ServerInfo,
  },
  prompt, prompt_router,
  schemars::JsonSchema,
  service::RequestContext,
  tool, tool_handler, tool_router, RoleServer, ServerHandler,
};
use serde::Deserialize;
use tracing::debug;
//pub(crate) use sse_server::{McpState, SseServer, SseServerConfig};

use crate::{coder::default_system_prompt, run::execute_js_code};
use futures::Future;

use super::AppState;

#[derive(Clone)]
pub struct A2AMcp {
  tool_router: ToolRouter<Self>,
  pub(crate) state: Arc<AppState>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct RunParams {
  #[schemars(description = "the script to run")]
  script: String,
}

#[tool_router]
impl A2AMcp {
  pub fn new(state: Arc<AppState>) -> Self {
    Self {
      state,
      tool_router: Self::tool_router(),
    }
  }

  #[tool(
    description = r#"run some javascript source, the result will be returned as JSON with two fields: `result` and `logs`

    - `result`: a json object that contains the result of the script execution, use it as what user expected
    - `logs`: a string array with the logs of the script execution, use it as needed
    "#
  )]
  async fn a2a_run(
    &self,
    Parameters(RunParams { script }): Parameters<RunParams>,
  ) -> Result<CallToolResult, rmcp::ErrorData> {
    debug!(script, "a2a_mcp_run");
    let result: Value = self
      .a2a_run_impl(script)
      .await
      .map_err(|e| rmcp::ErrorData::internal_error(format!("a2a_run failed: {}", e), None))?;
    Ok(CallToolResult::success(vec![Content::json(result)?]))
  }

  async fn a2a_run_impl(&self, script: String) -> anyhow::Result<Value> {
    let params = Value::Null;
    let clean_up = None;
    let conf = self
      .state
      .conf
      .read()
      .map_err(|e| anyhow::anyhow!("Failed to read configuration: {}", e))?
      .clone();
    let result = execute_js_code(&script, &conf, &params, clean_up).await?;

    Ok(result)
  }
}

#[prompt_router]
impl A2AMcp {
  #[prompt(name = "a2a", description = "Let llm know how to use a2a_run tool")]
  async fn greeting(&self) -> Vec<PromptMessage> {
    let prompt = format!(
      r#"You should write JavaScript code to meet user needs,
then call tool `a2a_run` to execute the script and process the results, 
then reply the user based on the result of the script. 

If other tools are also provided here, 
please select and use the appropriate tool based on the user's input and the tool's description to obtain more reference information when you write the script.

Basic and mandatory coding standards are as follows

{}"#,
      default_system_prompt().to_string()
    );
    vec![PromptMessage::new_text(PromptMessageRole::User, prompt)]
  }

  #[prompt(
    name = "config",
    description = "Convert the natural language description of the configuration to a JSON object"
  )]
  async fn config(&self) -> Vec<PromptMessage> {
    vec![PromptMessage::new_text(
      PromptMessageRole::User,
      "Hello! How can you help me today?",
    )]
  }
}

#[tool_handler]
impl ServerHandler for A2AMcp {
  fn get_info(&self) -> ServerInfo {
    ServerInfo::new(
      ServerCapabilities::builder()
        .enable_tools()
        .enable_prompts()
        .build(),
    )
    .with_protocol_version(ProtocolVersion::V_2025_06_18)
    .with_server_info(Implementation::new("a2a", "0.1.19"))
  }
}
