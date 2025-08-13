// mod sse_server;

use std::sync::Arc;

use a2a_types::Value;
use rmcp::{
  handler::server::tool::{Parameters, ToolRouter},
  model::{
    CallToolResult, Content, GetPromptRequestParam, GetPromptResult, Implementation,
    ListPromptsResult, PaginatedRequestParam, Prompt, PromptMessage, PromptMessageContent,
    PromptMessageRole, ProtocolVersion, ServerCapabilities, ServerInfo,
  },
  schemars::JsonSchema,
  service::RequestContext,
  tool, tool_handler, tool_router, RoleServer, ServerHandler,
};
use serde::Deserialize;
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

#[tool_handler]
impl ServerHandler for A2AMcp {
  fn get_info(&self) -> ServerInfo {
    ServerInfo {
      protocol_version: ProtocolVersion::V_2025_03_26,
      capabilities: ServerCapabilities::builder()
        .enable_tools()
        .enable_prompts()
        .build(),
      server_info: Implementation::from_build_env(),
      instructions: Some(
        "This server provides `a2a_run` tools to run a javascript script.".to_string(),
      ),
    }
  }

  async fn list_prompts(
    &self,
    _request: Option<PaginatedRequestParam>,
    _: RequestContext<RoleServer>,
  ) -> Result<ListPromptsResult, rmcp::ErrorData> {
    Ok(ListPromptsResult {
      next_cursor: None,
      prompts: vec![
        Prompt::new("a2a", Some("Let llm know how to use a2a_run tool"), None),
        Prompt::new(
          "config",
          Some("Convert the natural language description of the configuration to a JSON object"),
          None,
        ),
      ],
    })
  }

  async fn get_prompt(
    &self,
    GetPromptRequestParam { name, arguments: _ }: GetPromptRequestParam,
    _: RequestContext<RoleServer>,
  ) -> Result<GetPromptResult, rmcp::ErrorData> {
    match name.as_str() {
      "a2a" => {
        let prompt = format!("You should write javascript code to complete the user input, then call tool `a2a_run` to execute the script and process the results, then answer the user based on the result of the script. \n {}", default_system_prompt().to_string());
        Ok(GetPromptResult {
          description: None,
          messages: vec![PromptMessage {
            role: PromptMessageRole::User,
            content: PromptMessageContent::text(prompt),
          }],
        })
      }
      _ => Err(rmcp::ErrorData::invalid_params("prompt not found", None)),
    }
  }
}
