use a2a_types::Action;
use anyhow::Result;
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
