use a2a_types::Action;
use anyhow::Result;
use crawl_action::web_search_action;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{info, trace};
use utils::uuid_v7;

mod crawl_action;
mod email_action;
mod enc_action;
mod file_action;
mod http_action;
mod llm_action;
mod notify_action;
mod shell_action;
mod sql_action;
pub mod utils;

pub async fn do_action(action: Action) -> Result<Value> {
  let id = uuid_v7();
  if tracing::enabled!(tracing::Level::TRACE) {
    trace!(id, ?action, "do_action start");
  } else {
    info!(id, kind = action.get_kind(), "do_action start");
  }
  let r = match action {
    Action::Http(a) => http_action::do_action(a).await.map(Into::into),
    Action::File(a) => file_action::do_action(a).await.map(Into::into),
    Action::Sql(a) => sql_action::do_action(a).await.map(Into::into),
    Action::EMail(a) => email_action::do_action(a).await.map(Into::into),
    Action::Shell(a) => shell_action::do_action(a).await.map(Into::into),
    Action::Llm(a) => llm_action::do_action(a).await.map(Into::into),
    Action::Notify(a) => notify_action::do_action(a).await.map(Into::into),
    Action::Enc(a) => enc_action::do_action(a).map(Into::into),
    Action::Crawl(a) => crawl_action::do_action(a).await.map(Into::into),
    Action::WebSearch(a) => web_search_action::do_action(a).await.map(Into::into),
  };

  if tracing::enabled!(tracing::Level::TRACE) {
    let result = a2a_types::ActionResultWrapper { result: &r };
    trace!(id, ?result, "do_action result");
  } else {
    match r {
      Ok(_) => info!(id, "do_action success"),
      Err(ref err) => info!(id, ?err, "do_action error"),
    }
  }
  r
}

#[derive(Debug, Serialize, Default, Deserialize)]
pub(crate) struct EmailAccount {
  pub imap: email::imap::config::ImapConfig,
  pub smtp: email::smtp::config::SmtpConfig,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub(crate) struct ActionConnection {
  pub file_s3: opendal::services::S3Config,
  pub file_blob: opendal::services::AzblobConfig,
  pub file_oss: opendal::services::OssConfig,
  pub file_gcs: opendal::services::GcsConfig,
  pub file_ftp: opendal::services::FtpConfig,

  pub sql_mysql: String,
  pub sql_postgres: String,
  pub sql_sqlite: String,

  pub email_account: EmailAccount,

  pub llm: llm_action::LlmConnection,

  pub browser: crawl_action::SerializableLaunchOptions,
}

/// get the structure of the default connection
pub fn default_connection() -> Value {
  let mut connection = ActionConnection::default();
  connection.sql_mysql = "mysql://root:password@localhost:3306/db".to_string();
  connection.sql_postgres = "postgres://root:password@localhost:5432/db".to_string();
  connection.sql_sqlite = "sqlite://db.sqlite".to_string();

  let connection = serde_json::to_value(connection).unwrap();
  connection
}
