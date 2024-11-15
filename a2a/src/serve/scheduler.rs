use std::{
  collections::HashMap,
  fs,
  path::{Path, PathBuf},
};

use a2a_types::Value;
use anyhow::Result;
use chrono::{DateTime, Local};
use croner::Cron;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::{sync::mpsc, time::Duration};
use tracing::{debug, info, trace, warn};

use crate::run::execute_js_file;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScheduledTask {
  pub name: String,
  #[serde(
    serialize_with = "serialize_cron",
    deserialize_with = "deserialize_cron"
  )]
  pub crons: Vec<Cron>,
  pub command: String,
  pub args: Option<Vec<String>>,
  pub env: Option<HashMap<String, String>>,
  pub cwd: Option<String>,

  #[serde(skip)]
  pub conf: Value,
  #[serde(skip)]
  pub params: Value,
  #[serde(skip)]
  pub is_a2a: bool,
}

fn serialize_cron<S>(crons: &Vec<Cron>, s: S) -> Result<S::Ok, S::Error>
where
  S: serde::Serializer,
{
  let crons: Vec<String> = crons.iter().map(|c| c.pattern.to_string()).collect();
  s.collect_seq(crons)
}

fn deserialize_cron<'de, D>(d: D) -> Result<Vec<Cron>, D::Error>
where
  D: serde::Deserializer<'de>,
{
  let crons: Vec<String> = serde::Deserialize::deserialize(d)?;
  let crons: Vec<Cron> = crons
    .iter()
    .filter_map(|c| Cron::new(&c).with_seconds_optional().parse().ok())
    .collect();
  Ok(crons)
}

impl ScheduledTask {
  pub fn next(&self, now: Option<DateTime<Local>>) -> Option<Duration> {
    let now = now.unwrap_or(chrono::Local::now());
    self
      .crons
      .iter()
      .filter_map(|c| match c.find_next_occurrence(&now, false) {
        Ok(next) => Some(next),
        Err(err) => {
          debug!(task = self.name, err=?err, "task next failed");
          None
        }
      })
      .min()
      .and_then(|next| match next.signed_duration_since(now).to_std() {
        Ok(duration) => Some(duration),
        Err(err) => {
          debug!(
            task = self.name,
            now = ?now,
            next = ?next,
            err = ?err,
            "task next failed"
          );
          None
        }
      })
  }

  pub async fn run(&self) -> Result<()> {
    if self.is_a2a {
      self.run_a2a().await
    } else {
      self.run_shell().await
    }
  }

  pub fn build_params(&mut self) {
    let mut params = serde_json::Map::new();
    if let Some(args) = &self.args {
      args
        .iter()
        .filter_map(|arg| arg.split_once('='))
        .for_each(|(k, v)| {
          params.insert(k.to_string(), json!(v));
        });
    }
    if let Some(env) = &self.env {
      env.iter().for_each(|(k, v)| {
        params.insert(k.to_string(), json!(v));
      });
    }
    self.params = Value::Object(params);
  }

  async fn run_a2a(&self) -> Result<()> {
    let result = execute_js_file(&self.command, &self.conf, &self.params, None).await?;
    debug!(task = self.name, result= ?result, "task success");
    Ok(())
  }

  async fn run_shell(&self) -> Result<()> {
    let mut cmd = tokio::process::Command::new(&self.command);
    if let Some(args) = &self.args {
      cmd.args(args);
    }
    if let Some(env) = &self.env {
      cmd.envs(env);
    }
    if let Some(cwd) = &self.cwd {
      cmd.current_dir(cwd);
    }
    match cmd.output().await {
      Err(err) => Err(anyhow::anyhow!(err)),
      Ok(output) => {
        if output.status.success() {
          trace!(
            task = self.name.as_str(),
            "task success: {}",
            String::from_utf8_lossy(&output.stdout)
          );
          Ok(())
        } else {
          Err(anyhow::anyhow!(String::from_utf8_lossy(&output.stderr)
            .trim()
            .to_string()))
        }
      }
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", content = "payload", rename_all = "camelCase")]
pub enum ScheduleEvent {
  Pause(String),
  Resume(String),
  Stop(String),
  Reload(String),
  ReloadAll,
  StopAll,
}

pub type ScheduleAdminSender = mpsc::Sender<ScheduleEvent>;
type ScheduleAdminReceiver = mpsc::Receiver<ScheduleEvent>;

pub(crate) async fn start(
  api_root: PathBuf,
  schedule_root: PathBuf,
  conf: Value,
) -> Result<ScheduleAdminSender> {
  let (sender, recv) = mpsc::channel(1024);
  let mut states = IndexMap::new();

  // let mut schedules = load_schedulers(&schedule_root).await?;
  // schedules.iter_mut().for_each(|task| {
  //   task.conf = conf.clone();
  //   task.build_params();
  //   let a2a_file = api_root.join(&task.command);
  //   if a2a_file.exists() {
  //     task.is_a2a = true;
  //     task.command = a2a_file.to_str().unwrap_or_default().to_string();
  //   }
  // });
  let schedules = load_schedulers(&api_root, &schedule_root, &conf).await?;

  //debug!("schedules: {:?}", schedules);

  for task in schedules.into_iter() {
    let (task_sender, task_recv) = mpsc::channel(8);
    states.insert(task.name.clone(), task_sender);
    debug!(task = task.name, "task started");
    tokio::spawn(run_cron(task, task_recv));
  }
  states.sort_unstable_keys();

  tokio::spawn(run_manager(
    states,
    api_root.clone(),
    schedule_root.clone(),
    conf.clone(),
    recv,
  ));

  Ok(sender)
}

fn relative_path<P1: AsRef<Path>, P2: AsRef<Path>>(root: P1, path: P2) -> String {
  let root = root.as_ref();
  let path = path.as_ref();
  if root.eq(path) {
    return root
      .with_extension("")
      .file_name()
      .unwrap_or_default()
      .to_string_lossy()
      .to_string();
  }
  let path = if path.starts_with(root) {
    path.strip_prefix(root).unwrap_or(path)
  } else {
    path
  };
  // remove leading / and extension
  path
    .strip_prefix("/")
    .unwrap_or(path)
    .with_extension("")
    .to_string_lossy()
    .to_string()
}

fn load_schedulers_file<P1: AsRef<Path>, P2: AsRef<Path>>(
  api_root: P1,
  scheduler_root: P1,
  conf: &Value,
  file_name: P2,
) -> Result<Vec<ScheduledTask>> {
  let scheduler_root = scheduler_root.as_ref();
  let file_name = file_name.as_ref();
  let api_root = api_root.as_ref();

  let content = fs::read_to_string(file_name)?;
  let mut schedules = serde_json::from_str::<Vec<ScheduledTask>>(&content)?;
  schedules.iter_mut().for_each(|task| {
    task.conf = conf.clone();
    task.build_params();
    let a2a_file = api_root.join(&task.command);
    if a2a_file.exists() {
      task.is_a2a = true;
      task.command = a2a_file.to_str().unwrap_or_default().to_string();
    }
    task.name = format!("{}/{}", relative_path(scheduler_root, file_name), task.name);
  });
  Ok(schedules)
}

async fn load_schedulers(
  api_root: &PathBuf,
  scheduler_root: &PathBuf,
  conf: &Value,
) -> Result<Vec<ScheduledTask>> {
  let schedules = if scheduler_root.is_file() {
    load_schedulers_file(api_root, scheduler_root, conf, scheduler_root)?
  } else {
    let mut schedules = Vec::new();
    let pattern = scheduler_root.join("**/*.json");

    if let Some(walker) = pattern.to_str().and_then(|p| globwalk::glob(p).ok()) {
      for entry in walker {
        if let Ok(entry) = entry {
          let file_name = entry.path();
          match load_schedulers_file(api_root, scheduler_root, conf, file_name) {
            Ok(a) => {
              schedules.extend(a);
            }
            Err(err) => warn!(?file_name, ?err, "load scheduler failed"),
          }
        }
      }
    }
    schedules
  };

  Ok(schedules)
}

async fn run_cron(task: ScheduledTask, mut event_recv: ScheduleAdminReceiver) {
  let (timer_sender, mut timer_recv) = mpsc::channel(2);
  timer_sender.send(()).await.ok();
  let mut paused = false;
  loop {
    tokio::select! {
      _ = timer_recv.recv() => {
        if !paused {
          debug!(task=task.name, "task run");
          match task.run().await {
            Ok(_) => {}
            Err(err) => {
              debug!(task=task.name, err = ?err, "task error");
            }
          }
        }
        let next_duration = task.next(None).unwrap_or(Duration::from_secs(1));
        let timer_sender = timer_sender.clone();
        tokio::spawn(async move {
          let _ = tokio::time::sleep(next_duration).await;
          let _ = timer_sender.send(()).await;
        });
      }
      v = event_recv.recv() => {
        info!(task=task.name, event=?v, "task event");
        match v {
          None => break,
          Some(ScheduleEvent::Pause(_)) => paused = true,
          Some(ScheduleEvent::Resume(_)) => paused = false,
          Some(ScheduleEvent::Stop(_)) => {
            break;
          },
          _ => {}
        };
      }
    };
  }
  debug!(task = task.name, "task stopped");
}

async fn invoke_matched(
  states: &mut IndexMap<String, ScheduleAdminSender>,
  name: &String,
  event: ScheduleEvent,
  remove_matched: bool,
) {
  let mut matched = Vec::new();
  if name.ends_with('*') {
    // prefix match
    let prefix = name.trim_end_matches('*').to_string();
    let first = match states.binary_search_keys(&prefix) {
      Ok(i) => i,
      Err(i) => i,
    };
    for (name, sender) in states.iter().skip(first) {
      if name.starts_with(&prefix) {
        sender.send(event.clone()).await.ok();
        if remove_matched {
          matched.push(name.clone());
        }
      } else {
        break;
      }
    }
  } else {
    if let Some(sender) = states.get(name) {
      sender.send(event).await.ok();
      if remove_matched {
        matched.push(name.clone());
      }
    }
  }
  if remove_matched {
    for name in matched.into_iter() {
      states.swap_remove(&name);
    }
    states.sort_unstable_keys();
  }
}

async fn run_manager(
  mut states: IndexMap<String, ScheduleAdminSender>,
  api_root: PathBuf,
  scheduler_root: PathBuf,
  conf: Value,
  mut recv: ScheduleAdminReceiver,
) {
  loop {
    match recv.recv().await {
      None => {
        debug!("manager stopped");
        break;
      }
      Some(ScheduleEvent::Pause(name)) => {
        invoke_matched(
          &mut states,
          &name,
          ScheduleEvent::Pause(name.clone()),
          false,
        )
        .await;
      }
      Some(ScheduleEvent::Resume(name)) => {
        invoke_matched(
          &mut states,
          &name,
          ScheduleEvent::Resume(name.clone()),
          false,
        )
        .await;
      }
      Some(ScheduleEvent::Stop(name)) => {
        let current_task_name = format!("{}/*", name);
        invoke_matched(
          &mut states,
          &current_task_name,
          ScheduleEvent::Stop(name.clone()),
          true,
        )
        .await;
      }
      Some(ScheduleEvent::Reload(name)) => {
        let current_task_name = format!("{}/*", name);
        invoke_matched(
          &mut states,
          &current_task_name,
          ScheduleEvent::Stop(name.clone()),
          true,
        )
        .await;
        let schedule_file = scheduler_root.join(&name).with_extension("json");
        let schedules = load_schedulers_file(&api_root, &scheduler_root, &conf, &schedule_file)
          .unwrap_or_default();
        for task in schedules.into_iter() {
          let (task_sender, task_recv) = mpsc::channel(8);
          states.insert(task.name.clone(), task_sender);
          debug!(task = task.name, "task started");
          tokio::spawn(run_cron(task, task_recv));
        }
        states.sort_unstable_keys();
      }
      Some(ScheduleEvent::ReloadAll) => {}
      Some(ScheduleEvent::StopAll) => {
        for sender in states.values() {
          sender.send(ScheduleEvent::StopAll).await.ok();
        }
        break;
      }
    }
  }
}

pub async fn test_scheduler(schedule: &crate::app_conf::Scheduler) -> Result<()> {
  let schedules = fs::read_to_string(&schedule.config)?;
  let mut schedules = serde_json::from_str::<Vec<ScheduledTask>>(&schedules)?;
  schedules.iter_mut().for_each(|task| {
    task.build_params();
  });

  let mut start = schedule
    .start
    .as_ref()
    .and_then(|start| start.parse().ok())
    .unwrap_or(chrono::Local::now());
  let num = schedule.num.unwrap_or(10);

  if let Some(task) = schedules.iter().find(|task| task.name == schedule.task) {
    for _ in 0..num {
      if let Some(next) = task.next(Some(start)) {
        start += next;
        info!("task {} next {:?}", task.name, start);
      } else {
        break;
      }
    }
  } else {
    return Err(anyhow::anyhow!("task {} not found", schedule.task));
  }

  Ok(())
}
