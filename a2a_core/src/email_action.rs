use std::sync::Arc;

use a2a_types::{EMailAction, EMailActionResult, Value};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use email::{
  account::config::AccountConfig,
  backend::{
    self,
    context::{BackendContext, BackendContextBuilder},
    macros::BackendContext,
    Backend, BackendBuilder,
  },
  envelope::list::{ListEnvelopes, ListEnvelopesOptions},
  imap::{config::ImapConfig, ImapClientBuilder, ImapContextBuilder, ImapContextSync},
  message::send::smtp,
  smtp::{config::SmtpConfig, SmtpContextBuilder, SmtpContextSync},
  AnyResult,
};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::save_point::{self, WithSavePoint};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
struct EMailConfig {
  pub smtp: Option<SmtpConfig>,
  pub imap: Option<ImapConfig>,
}

#[derive(BackendContext)]
struct EMailContext {
  smtp: Option<SmtpContextSync>,
  imap: Option<ImapContextSync>,
}

impl AsRef<Option<ImapContextSync>> for EMailContext {
  fn as_ref(&self) -> &Option<ImapContextSync> {
    &self.imap
  }
}

impl AsRef<Option<SmtpContextSync>> for EMailContext {
  fn as_ref(&self) -> &Option<SmtpContextSync> {
    &self.smtp
  }
}

#[derive(Clone)]
struct EMailContextBuilder {
  smtp: Option<SmtpContextBuilder>,
  imap: Option<ImapContextBuilder>,
}

#[async_trait]
impl BackendContextBuilder for EMailContextBuilder {
  type Context = EMailContext;

  async fn build(self) -> AnyResult<Self::Context> {
    let imap = match self.imap {
      Some(imap) => Some(imap.build().await?),
      None => None,
    };
    let smtp = match self.smtp {
      Some(smtp) => Some(smtp.build().await?),
      None => None,
    };
    Ok(EMailContext { smtp, imap })
  }
}

pub async fn do_action(action: EMailAction) -> Result<EMailActionResult> {
  let mut save_point = action.load().await.ok();

  let current_folder = action
    .folder
    .as_ref()
    .map(|f| f.as_str())
    .unwrap_or("INBOX");

  let last_id = save_point
    .as_ref()
    .and_then(|v| v.pointer("/lastId"))
    .and_then(|v| v.as_u64())
    .unwrap_or_default();
  match action.method.as_str().to_uppercase().as_str() {
    "READ" => match on_read(&action.account, current_folder, last_id).await {
      Ok((msg, id)) => Ok(msg),
      Err(e) => Err(e),
    },
    _ => Err(anyhow::anyhow!("Invalid method")),
  }
}

async fn setup_backend(action: &EMailAction) -> Result<Backend<EMailContext>> {
  let imap_config = action
    .account
    .get("imap")
    .and_then(|v| serde_json::from_value(v.clone()).ok())
    .map(|c| Arc::new(c));

  let smtp_config = action
    .account
    .get("smtp")
    .and_then(|v| serde_json::from_value(v.clone()).ok())
    .map(|c| Arc::new(c));

  let account_config = Arc::new(AccountConfig::default());

  println!(
    "imap: {} smtp:{}",
    imap_config.is_some(),
    smtp_config.is_some()
  );

  let ctx_builder = EMailContextBuilder {
    smtp: smtp_config.map(|c| SmtpContextBuilder::new(account_config.clone(), c)),
    imap: imap_config.map(|c| ImapContextBuilder::new(account_config.clone(), c)),
  };

  let backend_builder = BackendBuilder::new(account_config.clone(), ctx_builder);
  match backend_builder.build().await {
    Ok(backend) => Ok(backend),
    Err(err) => Err(anyhow::anyhow!("Failed to build backend: {:?}", err)),
  }
}
async fn on_read(account: &Value, folder: &str, last_id: u64) -> Result<(Value, u64)> {
  let account_config = Arc::new(AccountConfig::default());
  let ctx_builder = account
    .get("imap")
    .ok_or(anyhow!("no imap config found"))
    .and_then(|v| serde_json::from_value(v.clone()).map_err(Into::into))
    .map(|c| Arc::new(c))
    .map(|c| ImapContextBuilder::new(account_config.clone(), c))?;

  let builder = BackendBuilder::new(account_config.clone(), ctx_builder);
  let backend = builder.build().await?;

  let mut envelopes = Vec::new();
  let mut page = 0;
  loop {
    let chunk = backend
      .list_envelopes(
        folder,
        ListEnvelopesOptions {
          page_size: 10,
          page,
          query: None,
        },
      )
      .await?;
    let should_break = chunk.is_empty()
      || envelopes.len() >= 30
      || chunk.iter().any(|e| {
        e.id
          .parse::<u64>()
          .map(|nid| nid < last_id)
          .unwrap_or(false)
      });

    info!(
      should_break,
      page,
      len = chunk.len(),
      msgs = envelopes.len(),
      "chunk"
    );

    envelopes.extend(chunk.into_iter().filter(|e| {
      e.id
        .parse::<u64>()
        .map(|nid| nid > last_id)
        .unwrap_or(false)
    }));
    page += 1;
    if should_break {
      break;
    }
  }

  envelopes.iter().for_each(|e| {
    info!("EMail received {} {} {:?}", e.id, e.subject, e.date);
  });

  Ok((Value::Null, 0))
}
