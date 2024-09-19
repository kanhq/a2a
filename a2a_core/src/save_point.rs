use a2a_types::{EMailAction, FileAction, HttpAction, SqlAction};
use anyhow::Result;
use serde_json::Value;

use crate::file_action;
pub(crate) trait WithSavePoint {
  fn get_save_point(&self) -> Option<String>;
  fn get_save_point_connection(&self) -> Option<Value>;

  async fn load(&self) -> Result<Value> {
    let path = self
      .get_save_point()
      .ok_or(anyhow::anyhow!("No save point"))?;

    let file_action = FileAction {
      method: "READ".to_string(),
      path,
      connection: self.get_save_point_connection().clone(),
      ..Default::default()
    };

    file_action::do_action(file_action).await
  }

  async fn save(&self, value: &Value) -> Result<()> {
    let path = self
      .get_save_point()
      .clone()
      .ok_or(anyhow::anyhow!("No save point"))?;

    let file_action = FileAction {
      method: "WRITE".to_string(),
      path,
      body: Some(serde_json::to_vec_pretty(value)?.into()),
      connection: self.get_save_point_connection().clone(),
      ..Default::default()
    };

    file_action::do_action(file_action).await?;

    Ok(())
  }
}

impl WithSavePoint for SqlAction {
  fn get_save_point(&self) -> Option<String> {
    self.save_point.clone()
  }

  fn get_save_point_connection(&self) -> Option<Value> {
    self.save_point_connection.clone()
  }
}

impl WithSavePoint for FileAction {
  fn get_save_point(&self) -> Option<String> {
    self.save_point.clone()
  }

  fn get_save_point_connection(&self) -> Option<Value> {
    self.save_point_connection.clone()
  }
}

impl WithSavePoint for HttpAction {
  fn get_save_point(&self) -> Option<String> {
    self.save_point.clone()
  }

  fn get_save_point_connection(&self) -> Option<Value> {
    self.save_point_connection.clone()
  }
}

impl WithSavePoint for EMailAction {
  fn get_save_point(&self) -> Option<String> {
    self.save_point.clone()
  }

  fn get_save_point_connection(&self) -> Option<Value> {
    self.save_point_connection.clone()
  }
}
