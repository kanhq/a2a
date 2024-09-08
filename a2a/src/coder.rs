use std::{
  path::{Path, PathBuf},
  str::FromStr,
};

use crate::{
  app_conf::{Coder, Runner},
  run,
};

use a2a_types::Value;
use anyhow::Result;
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::{io::AsyncBufReadExt, task::JoinSet};
use tracing::{debug, info, warn};

const DEFAULT_SYSTEM_PROMPT: &'static str = include_str!("./code.md");

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct WriteCode {
  // ## input
  pub system: String,
  pub user: String,
  pub output: Option<String>,
  pub provider: String,
  pub model: String,
  pub base_url: String,
  pub api_key: String,

  // # stats
  pub retry: usize,
  pub first_token_time: u64,
  pub total_time: u64,
  pub prompt_tokens: usize,
  pub response_tokens: usize,
  pub run_result: Option<Value>,
}

pub(crate) async fn execute(arg: &Coder) -> Result<()> {
  let system = arg
    .system
    .as_ref()
    .map(|s| text_or_file(&s))
    .map(|v| v.0)
    .unwrap_or(DEFAULT_SYSTEM_PROMPT.to_string());

  let (user, is_file) = text_or_file(&arg.user);

  let default_output_name = if is_file {
    arg
      .user
      .rsplit_once('.')
      .map(|(name, _)| format!("{}.{{provider}}.{{model}}.js", name))
  } else {
    Some("a2a.{{provider}}.{{model}}.js".to_string())
  };

  let mut set = JoinSet::new();

  let _r = arg
    .models
    .iter()
    .map(|(provider, model)| {
      let output = arg.file.as_ref().or(default_output_name.as_ref()).map(|f| {
        f.replace("{provider}", &provider)
          .replace("{model}", &model)
      });

      WriteCode {
        system: system.clone(),
        user: user.clone(),
        output: output.clone(),
        provider: provider.clone(),
        model: model.clone(),
        base_url: arg.base_url.clone(),
        api_key: arg.api_key.clone(),

        ..Default::default()
      }
    })
    .map(|code| set.spawn(write_code(code)))
    .collect::<Vec<_>>();

  let mut results = vec![];
  while let Some(done) = set.join_next().await {
    match done {
      Ok(Ok(code)) => results.push(code),
      Ok(Err(err)) => warn!(?err, "write code error"),
      Err(err) => warn!(?err, "write code join error"),
    }
  }

  if arg.run.unwrap_or(false) {
    let conf_dir = arg
      .conf_dir
      .as_ref()
      .and_then(|d| PathBuf::from_str(d.as_str()).ok())
      .ok_or(anyhow::anyhow!("invalid conf dir"))?;
    for r in results.iter_mut() {
      if let Some(output) = r.output.as_ref() {
        let runner = Runner {
          file: output.clone(),
          conf_dir: conf_dir.clone(),
          clean: arg.clean.clone(),
        };
        r.run_result = run::execute(&runner).await.ok();
      }
    }
  }

  println!(
    "{:<20}\t{:<20}\t{:<10}\t{:<10}\t{:<10}\t{:<10}\t{:<10}\t{:<12}\t{:<10}",
    "provider",
    "model",
    "first_time",
    "total_time",
    "input",
    "output",
    "com_speed",
    "infer_speed",
    "run_success",
  );
  results.iter().for_each(|r| {
    println!(
      "{:<20}\t{:<20}\t{:<10}\t{:<10}\t{:<10}\t{:<10}\t{:<10.2}\t{:<12.2}\t{:<10}",
      r.provider,
      r.model,
      r.first_token_time,
      r.total_time,
      r.prompt_tokens,
      r.response_tokens,
      ((r.prompt_tokens * 1000) as f64) / (r.first_token_time as f64),
      ((r.response_tokens * 1000) as f64) / ((r.total_time - r.first_token_time) as f64),
      r.run_result.as_ref().map(|_v| 1).unwrap_or(0),
    );
  });

  Ok(())
}

fn text_or_file(text: &str) -> (String, bool) {
  let path = Path::new(text);
  if path.is_file() {
    (
      std::fs::read_to_string(path).unwrap_or(text.to_string()),
      true,
    )
  } else {
    (text.to_string(), false)
  }
}

#[derive(Debug, Serialize, Deserialize)]
struct Usage {
  pub completion_tokens: usize,
  pub prompt_tokens: usize,
  pub total_tokens: usize,
}

async fn write_code(mut code: WriteCode) -> Result<WriteCode> {
  info!(
    provider = code.provider.as_str(),
    model = code.model.as_str(),
    "writing code start"
  );

  let client = reqwest::Client::new();

  let llm_body = json!({
    "model": code.model,
    "messages": [
      {
        "role": "system",
        "content": code.system
      },
      {
        "role": "user",
        "content": code.user
      }
    ],
    "stream": true,
    "stream_options": {
      "include_usage": true,
    },
    "max_tokens": 2048,
  });
  let url = format!("{}/chat/completions", code.base_url);
  let request = client
    .post(&url)
    .bearer_auth(&code.api_key)
    .header("x-portkey-provider", &code.provider)
    .json(&llm_body)
    .build()?;

  let start_time = time::OffsetDateTime::now_utc();

  let response = client.execute(request).await?;
  // check if the response is successful
  response.error_for_status_ref()?;

  let stream = response
    .bytes_stream()
    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e));

  let mut read = tokio_util::io::StreamReader::new(stream).lines();

  let mut first_token = true;
  let mut llm_response: String = String::new();
  let mut step = 0;
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
      match serde_json::from_str::<Value>(chunk) {
        Ok(data) => {
          //debug!(chunk, "stream chunk");
          if let Some(usage) = data
            .pointer("/usage")
            .and_then(|u| serde_json::from_value::<Usage>(u.clone()).ok())
          {
            code.prompt_tokens = usage.prompt_tokens;
            code.response_tokens = usage.completion_tokens;
          }

          let content = data
            .pointer("/choices/0/delta/content")
            .and_then(|c| c.as_str())
            .unwrap_or_default();
          if first_token {
            code.first_token_time =
              (time::OffsetDateTime::now_utc() - start_time).whole_milliseconds() as u64;
            first_token = false;
          }
          llm_response.push_str(content);
          let lines = llm_response.lines().count();
          if lines / 10 > step {
            debug!(
              lines,
              provider = code.provider.as_str(),
              model = code.model.as_str(),
              "stream lines"
            );
          }
          step = lines / 10;
        }
        Err(err) => {
          warn!(?err, chunk, "parse stream error");
          break;
        }
      }
    }
  }

  let end_time = time::OffsetDateTime::now_utc();
  code.total_time = (end_time - start_time).whole_milliseconds() as u64;

  if let Some(output) = code.output.as_ref() {
    debug!(output, "llm response");
    std::fs::write(output, extract_code(&llm_response))?;
  }

  info!(
    provider = code.provider.as_str(),
    model = code.model.as_str(),
    "writing code done"
  );

  Ok(code)
}

fn extract_code(resp: &str) -> String {
  let mut code = String::new();
  let mut in_code = false;
  for line in resp.lines() {
    if line.trim().starts_with("```") {
      in_code = !in_code;
    } else if in_code {
      code.push_str(line);
      code.push('\n');
    }
  }
  code
}
