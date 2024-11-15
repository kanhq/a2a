use a2a_types::{NotifyAction, NotifyActionResult, Value};
use anyhow::Result;
use serde_json::json;
use tracing::debug;

pub async fn do_action(action: NotifyAction) -> Result<NotifyActionResult> {
  if action.url.is_empty() {
    return Err(anyhow::anyhow!("url is empty"));
  }
  let provider = Provider::from(action.url.as_str());
  provider.do_notify(action).await
}

#[derive(Debug, Clone)]
enum Provider {
  Common,
  DingTalk,
  Feishu,
  WxWork,
  Slack,
  Telegram,
  Teams,
}

impl From<&str> for Provider {
  fn from(s: &str) -> Self {
    static PROVIDERS: [Provider; 7] = [
      Provider::Common,
      Provider::DingTalk,
      Provider::Feishu,
      Provider::WxWork,
      Provider::Slack,
      Provider::Telegram,
      Provider::Teams,
    ];
    PROVIDERS
      .iter()
      .find(|p| p.keyword().iter().any(|k| s.contains(k)))
      .cloned()
      .unwrap_or(Provider::Common)
  }
}

impl Provider {
  fn keyword(&self) -> &[&str] {
    match self {
      Self::Common => &[],
      Self::DingTalk => &["dingtalk"],
      Self::Feishu => &["feishu"],
      Self::WxWork => &["qyapi.weixin"],
      Self::Slack => &["slack"],
      Self::Telegram => &["telegram"],
      Self::Teams => &["teams"],
    }
  }

  async fn do_notify(&self, action: NotifyAction) -> Result<NotifyActionResult> {
    let url = action.url.clone();
    let body = self.build_body(action);

    let client = reqwest::Client::builder().build()?;
    debug!(?url, ?body, provider=?self, "do_notify");
    let resp = client.post(url).json(&body).send().await?;
    Ok(resp.json().await?)
  }

  fn build_body(&self, action: NotifyAction) -> Value {
    if action.message.is_string() {
      self.build_text_body(action)
    } else {
      action.message
    }
  }

  fn build_text_body(&self, action: NotifyAction) -> Value {
    match self {
      Self::DingTalk => json!({
        "msgtype": "markdown",
        "title": action.title.unwrap_or_default(),
        "text": {
          "content": action.message.as_str().unwrap(),
        },
      }),
      Self::Feishu => json!({
        "msg_type": "interactive",
        "card": {
          "elements": [
            {
              "tag": "div",
              "text": {
                "content": action.message.as_str().unwrap(),
                "tag": "lark_md"
              }
            }
          ],
          "header": {
            "title": {
              "content": action.title.unwrap_or_default(),
              "tag": "plain_text"
            }
          }
        },
      }),
      Self::WxWork => json!({
        "msgtype": "markdown",
        "markdown": {
          "content": action.message.as_str().unwrap(),
        },
      }),
      Self::Slack => json!({
        "text": action.message.as_str().unwrap(),
      }),
      Self::Telegram => json!({
        "text": action.message.as_str().unwrap(),
      }),
      Self::Teams => json!({
        "text": action.message.as_str().unwrap(),
      }),
      _ => json!({
        "text": action.message.as_str().unwrap(),
        "title": action.title.unwrap_or_default(),
      }),
    }
  }
}
