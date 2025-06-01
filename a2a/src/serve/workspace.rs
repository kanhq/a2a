use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{app_conf::InitWorkDir, init::init_workdir};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceFile {
  pub path: String,
  pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload", rename_all = "camelCase")]
pub enum WorkspaceOperation {
  Read(WorkspaceFile),
  Write(WorkspaceFile),
  Delete(WorkspaceFile),
  List(String),
  Initialize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload", rename_all = "camelCase")]
pub enum WorkspaceOperationResponse {
  File(WorkspaceFile),
  List(Vec<WorkspaceFile>),
  Initialize,
}

pub async fn handle_workspace_operation<P: AsRef<Path>>(
  root_path: P,
  operation: WorkspaceOperation,
) -> Result<WorkspaceOperationResponse> {
  match operation {
    WorkspaceOperation::Read(mut file) => {
      let path = root_path.as_ref().join(&file.path.trim_start_matches('/'));
      let content = std::fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("Failed to read {}: {}", path.display(), e))?;
      file.content = Some(content);
      Ok(WorkspaceOperationResponse::File(file))
    }
    WorkspaceOperation::Write(mut file) => {
      let path = root_path.as_ref().join(&file.path.trim_start_matches('/'));
      let content = file.content.unwrap_or_default();
      std::fs::write(&path, content)
        .map_err(|e| anyhow::anyhow!("Failed to write {}: {}", path.display(), e))?;
      file.content = None; // Clear content after writing
      Ok(WorkspaceOperationResponse::File(file))
    }
    WorkspaceOperation::Delete(file) => {
      let path = root_path.as_ref().join(&file.path.trim_start_matches('/'));
      std::fs::remove_file(&path)
        .map_err(|e| anyhow::anyhow!("Failed to delete {}: {}", path.display(), e))?;
      Ok(WorkspaceOperationResponse::File(file))
    }
    WorkspaceOperation::List(path) => {
      let dir_path = root_path.as_ref().join(path.trim_start_matches('/'));
      let mut files = walk_dir(&dir_path);
      let base = root_path.as_ref().to_string_lossy().to_string();
      files.iter_mut().for_each(|file| {
        // Normalize the file path to be relative to the root path
        file.path = file.path.trim_start_matches(&base).to_string();
      });
      Ok(WorkspaceOperationResponse::List(files))
    }
    WorkspaceOperation::Initialize => {
      let wd = InitWorkDir {
        work_dir: Some(root_path.as_ref().to_string_lossy().to_string()),
        root_path: root_path.as_ref().to_path_buf(),
      };
      init_workdir(&wd)?;
      Ok(WorkspaceOperationResponse::Initialize)
    }
  }
}

fn walk_dir<P: AsRef<Path>>(root: P) -> Vec<WorkspaceFile> {
  let mut files = Vec::new();
  if let Ok(mut entries) = std::fs::read_dir(root.as_ref()) {
    while let Some(Ok(entry)) = entries.next() {
      let path = entry.path();
      if path.is_file() {
        files.push(WorkspaceFile {
          path: path.to_string_lossy().to_string(),
          content: None,
        });
      } else if path.is_dir() {
        let sub_files = walk_dir(&path);
        files.extend(sub_files);
      }
    }
  }
  files
}
