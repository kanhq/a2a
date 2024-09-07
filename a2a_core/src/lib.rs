use a2a_types::Action;
use anyhow::Result;
use file::do_file_action;
use http::do_http_action;
use serde_json::Value;
use sql::do_sql_action;
use tracing::debug;

mod file;
mod http;
mod sql;

pub async fn do_action(action: Action) -> Result<Value> {
  debug!(?action, "do_action");
  match action {
    Action::Http(a) => do_http_action(a).await.map(Into::into),
    Action::File(a) => do_file_action(a).await.map(Into::into),
    Action::Sql(a) => do_sql_action(a).await.map(Into::into),
  }
}
