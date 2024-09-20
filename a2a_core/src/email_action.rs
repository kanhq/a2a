use std::sync::Arc;

use a2a_types::{EMailAction, EMailActionResult, Value};
use anyhow::{anyhow, Result};
use email::{
  account::config::AccountConfig,
  backend::BackendBuilder,
  envelope::{
    list::{ListEnvelopes, ListEnvelopesOptions},
    Id,
  },
  imap::ImapContextBuilder,
  message::peek::PeekMessages,
};
use mail_parser::MimeHeaders;
use serde_json::json;
use tracing::{debug, info, warn};

pub async fn do_action(action: EMailAction) -> Result<EMailActionResult> {
  let current_folder = action
    .folder
    .as_ref()
    .map(|f| f.as_str())
    .unwrap_or("INBOX");

  match action.method.as_str().to_uppercase().as_str() {
    "READ" => {
      on_read(
        &action.account,
        current_folder,
        action.last_id.unwrap_or_default(),
      )
      .await
    }
    _ => Err(anyhow::anyhow!("Invalid method")),
  }
}

async fn on_read(account: &Value, folder: &str, last_id: u64) -> Result<Value> {
  let account_config = Arc::new(AccountConfig::default());
  let ctx_builder = account
    .get("imap")
    .or(Some(account))
    .ok_or(anyhow!("miss imap config"))
    .and_then(|v| serde_json::from_value(v.clone()).map_err(Into::into))
    .map(|c| Arc::new(c))
    .map(|c| ImapContextBuilder::new(account_config.clone(), c))?;

  let user_email = ctx_builder.imap_config.login.clone();
  if user_email.is_empty() {
    return Err(anyhow!("miss user email"));
  }

  let builder = BackendBuilder::new(account_config.clone(), ctx_builder);
  let backend = builder.build().await?;

  let mut envelopes = Vec::new();
  let mut page = 0;
  info!(user_email, folder, last_id, "email check");
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

    debug!(
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

  let mut mails = Vec::new();

  let files_dir = std::env::temp_dir().join("a2a_email").join(&user_email);

  if let Err(err) = std::fs::create_dir_all(&files_dir) {
    warn!(user_email, %err, "create temp dir");
  }

  info!(user_email, folder, last_id, "email fetch");
  for envelope in envelopes.iter() {
    let id = Id::Single(envelope.id.clone().into());
    debug!(user_email, %id, "fetch message");
    match backend.peek_messages(folder, &id).await {
      Ok(msg) => {
        if let Some(msg) = msg.first() {
          match msg.parsed() {
            Ok(parsed) => {
              let body = parsed
                .text_bodies()
                .map(|b| b.text_contents().unwrap_or(""))
                .collect::<String>();
              let mut files = Vec::new();
              for a in parsed.attachments() {
                if let Some(filename) = a.attachment_name() {
                  let local_attachment_name = files_dir.join(filename);
                  debug!(user_email, %id, filename=?local_attachment_name, "email save attachment");
                  if let Err(err) = std::fs::write(&local_attachment_name, a.contents()) {
                    warn!(user_email, id=%envelope.id, filename=?local_attachment_name, %err, "email save attachment");
                    continue;
                  } else {
                    files.push(local_attachment_name);
                  }
                }
              }
              mails.push(json! {
                {
                  "id": envelope.id,
                  "subject": envelope.subject,
                  "from": envelope.from.addr.clone(),
                  "to": envelope.to.addr.clone(),
                  "date": envelope.date.to_string(),
                  "body": body,
                  "attachments": files,
                }
              });
            }
            Err(err) => {
              warn!(user_email, %id, %err, "email parse message failed");
            }
          }
        } else {
          warn!(user_email, %id, "email empty message failed");
        }
      }
      Err(err) => {
        warn!(user_email, %id, %err, "email peek message failed");
      }
    }
  }

  Ok(Value::Array(mails))
}
