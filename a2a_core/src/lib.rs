use a2a_types::Action;
use anyhow::Result;
use serde_json::Value;
use tracing::debug;

mod email_action;
mod file_action;
mod http_action;
mod sql_action;

pub async fn do_action(action: Action) -> Result<Value> {
  debug!(?action, "do_action");
  match action {
    Action::Http(a) => http_action::do_action(a).await.map(Into::into),
    Action::File(a) => file_action::do_action(a).await.map(Into::into),
    Action::Sql(a) => sql_action::do_action(a).await.map(Into::into),
    Action::EMail(a) => email_action::do_action(a).await.map(Into::into),
  }
}
