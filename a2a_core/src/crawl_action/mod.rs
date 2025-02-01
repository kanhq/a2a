use std::{collections::HashMap, ffi::OsString, path::PathBuf, sync::Arc, time::Duration};

use a2a_types::{CrawlAction, CrawlActionResult, LlmAction, Value};
use anyhow::Result;
use headless_chrome::{Browser, LaunchOptions, Tab};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::task::JoinSet;
use tracing::{debug, info, trace, warn};

use crate::llm_action;

mod url;

use url::{CrawlUrl, UrlPattern};

pub async fn do_action(action: CrawlAction) -> Result<CrawlActionResult> {
  let llm = action.llm.map(|v| Arc::new(v));

  let launch_options = action
    .browser
    .and_then(|browser| serde_json::from_value::<SerializableLaunchOptions>(browser).ok())
    .unwrap_or_default();

  let launch_options: LaunchOptions = (&launch_options).into();

  let browser = Browser::new(launch_options)?;
  let parallel = action.parallel.unwrap_or(1);
  let prompts = build_prompts_dict(action.fields.as_ref()).map(|v| Arc::new(v));
  let urls = action
    .urls
    .into_iter()
    .map(CrawlUrl::from)
    .collect::<Vec<_>>();

  let mut tasks = JoinSet::new();
  let llm = llm;

  urls.chunks(parallel).for_each(|urls| {
    match browser.new_tab() {
      Ok(tab) => {
        let llm = llm.clone();
        let urls = urls.to_vec();
        let prompts = prompts.clone();
        tasks.spawn(async move {
          let mut results = HashMap::new();
          for url in urls.iter() {
            debug!(?url, "start");
            let prompt = get_matched_prompt(prompts.as_ref().map(|v| v.as_slice()), url.as_ref());
            match do_crawl(&url, tab.clone(), llm.clone(), prompt).await {
              Ok(result) => {
                results.insert(url.url().to_string(), result);
              }
              Err(err) => {
                warn!(?url, ?err, "error");
              }
            }
            debug!(?url, "done");
          }
          results
        });
      }
      Err(err) => {
        warn!(?err, "new tab error");
      }
    };
  });

  let mut all = serde_json::Map::new();
  tasks.join_all().await.into_iter().for_each(|r| {
    all.extend(r.into_iter().map(|(k, v)| (k.to_string(), json!(v))));
  });

  Ok(all.into())
}

async fn do_crawl(
  url: &CrawlUrl,
  tab: Arc<Tab>,
  llm: Option<Arc<Value>>,
  prompt: Option<String>,
) -> Result<Value> {
  let pure_html = include_str!("pure_html.js");
  let crawl_system = include_str!("crawl_system.md");

  tab.navigate_to(url.url())?;

  //tab.wait_until_navigated()?;

  debug!(?url, "navigated");

  if !url.wait().is_empty() {
    tab.wait_for_element(url.wait())?;
  } else {
    tab.wait_until_navigated()?;
  }
  debug!(?url, "wait done");

  // let full = tab.get_content()?;
  // debug!(?url, "get content done");
  // debug!("{}", full);

  let args = vec![json!(url.selector())];
  let body = tab.find_element(url.selector())?;

  debug!(?url, "element found");

  let result = body.call_js_fn(&pure_html, args, true)?.value;

  debug!(?url, "html got");
  trace!(?url, ?result, "crawl result");

  let html = result.as_ref().and_then(|v| v.as_str());

  match html.zip(llm).zip(prompt) {
    Some(((html, llm), prompt)) => {
      let extract_action = LlmAction {
        override_result_mimetype: Some("application/json".to_string()),
        connection: Some(llm.as_ref().clone()),
        sys_prompt: Some(format!("{}\n <data>{}</data>", crawl_system, html)),
        user_prompt: Some(prompt),
        user_image: None,
      };
      info!(?url, "llm start");
      llm_action::do_action(extract_action).await
    }
    None => html
      .map(|h| Value::String(h.to_string()))
      .ok_or(anyhow::anyhow!("No html")),
  }
}

fn build_prompts_dict(fields: Option<&Value>) -> Option<Vec<(UrlPattern, String)>> {
  match fields {
    Some(Value::Object(prompts)) => {
      let mut result = Vec::new();
      for (k, v) in prompts.iter() {
        let p = UrlPattern::from(k.as_str());
        if let Some(prompt) = fields_to_type(v) {
          result.push((p, prompt));
        }
      }
      Some(result)
    }
    Some(Value::String(prompts)) => {
      let p = UrlPattern::from("*");
      Some(vec![(p, prompts.clone())])
    }
    Some(Value::Array(_)) => {
      let p = UrlPattern::from("*");
      fields_to_type(fields.unwrap()).map(|prompt| vec![(p, prompt)])
    }
    _ => None,
  }
}

fn get_matched_prompt<'a>(
  prompts: Option<&'a [(UrlPattern, String)]>,
  url: &str,
) -> Option<String> {
  prompts.and_then(|prompts| {
    prompts.iter().find_map(|(p, prompt)| {
      if p.is_match(url) {
        Some(prompt.clone())
      } else {
        None
      }
    })
  })
}

fn fields_to_type(fields: &Value) -> Option<String> {
  let fields = match fields {
    Value::Array(a) => a
      .iter()
      .filter_map(|v| v.as_str())
      .map(|f| format!("  {}: string", f))
      .collect::<Vec<_>>()
      .join("\n"),
    Value::String(prompt) => return Some(prompt.clone()),
    _ => return None,
  };

  Some(format!(
    r#"```typescript
type Data = {{
{}
}}[]
```"#,
    fields
  ))
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
struct SerializableLaunchOptions {
  pub headless: bool,
  pub devtools: bool,
  pub sandbox: bool,
  pub enable_gpu: bool,
  pub enable_logging: bool,
  pub window_size: Option<(u32, u32)>,
  pub port: Option<u16>,
  pub ignore_certificate_errors: bool,
  pub path: Option<PathBuf>,
  pub user_data_dir: Option<PathBuf>,
  pub extensions: Vec<OsString>,
  pub args: Vec<OsString>,
  pub ignore_default_args: Vec<OsString>,
  pub disable_default_args: bool,
  pub idle_browser_timeout: u64,
  pub process_envs: Option<HashMap<String, String>>,
  pub proxy_server: Option<String>,
}

impl Default for SerializableLaunchOptions {
  fn default() -> Self {
    Self {
      headless: true,
      devtools: false,
      sandbox: false,
      enable_gpu: false,
      enable_logging: false,
      idle_browser_timeout: 180,
      window_size: None,
      path: None,
      user_data_dir: None,
      port: None,
      ignore_certificate_errors: true,
      extensions: Vec::new(),
      process_envs: None,
      args: Vec::new(),
      ignore_default_args: Vec::new(),
      disable_default_args: false,
      proxy_server: None,
    }
  }
}

impl<'a> Into<LaunchOptions<'a>> for &'a SerializableLaunchOptions {
  fn into(self) -> LaunchOptions<'a> {
    LaunchOptions {
      headless: self.headless,
      devtools: self.devtools,
      sandbox: self.sandbox,
      enable_gpu: self.enable_gpu,
      enable_logging: self.enable_logging,
      idle_browser_timeout: Duration::from_secs(self.idle_browser_timeout),
      window_size: self.window_size,
      path: self.path.clone(),
      user_data_dir: self.user_data_dir.clone(),
      port: self.port,
      ignore_certificate_errors: self.ignore_certificate_errors,
      extensions: self.extensions.iter().map(|s| s.as_os_str()).collect(),
      args: self.args.iter().map(|s| s.as_os_str()).collect(),
      ignore_default_args: self
        .ignore_default_args
        .iter()
        .map(|s| s.as_os_str())
        .collect(),
      disable_default_args: self.disable_default_args,
      process_envs: self.process_envs.clone(),
      proxy_server: self.proxy_server.as_deref(),
    }
  }
}
