use std::{collections::HashMap, fs, path::PathBuf, process::Stdio};

use a2a_types::Value;
use anyhow::Result;
use croner::Cron;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::{sync::mpsc, time::Duration};
use tracing::{debug, info};

use crate::run::execute_js;

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
  pub fn next(&self) -> Option<Duration> {
    let now = chrono::Utc::now();
    self
      .crons
      .iter()
      .filter_map(|c| match c.find_next_occurrence(&now, false) {
        Ok(next) => Some(next),
        Err(err) => {
          debug!("task {} next {:?}", self.name, err);
          None
        }
      })
      .min()
      .and_then(|next| match next.signed_duration_since(now).to_std() {
        Ok(duration) => Some(duration),
        Err(err) => {
          debug!(
            "task {} next {:?} now {} next {}",
            self.name, err, now, next
          );
          None
        }
      })
  }

  pub async fn run(&self) -> Result<()> {
    if self.command.ends_with("a2a.js") {
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
    let _ = execute_js(&self.command, &self.conf, &self.params, None).await?;
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
    cmd.stdout(Stdio::null());
    let status = cmd.status().await?;
    if status.success() {
      Ok(())
    } else {
      Err(anyhow::anyhow!("task {} failed", self.name))
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

pub(crate) async fn start(schedule_file: PathBuf, conf: Value) -> Result<ScheduleAdminSender> {
  let (sender, recv) = mpsc::channel(1024);
  let mut states = IndexMap::new();
  let schedules = fs::read_to_string(schedule_file)?;
  let mut schedules = serde_json::from_str::<Vec<ScheduledTask>>(&schedules)?;
  schedules.iter_mut().for_each(|task| {
    task.conf = conf.clone();
    task.build_params();
  });

  //debug!("schedules: {:?}", schedules);

  for task in schedules.into_iter() {
    let (task_sender, task_recv) = mpsc::channel(8);
    states.insert(task.name.clone(), task_sender);
    debug!("task {} started", task.name);
    tokio::spawn(run_cron(task, task_recv));
  }
  states.sort_unstable_keys();

  tokio::spawn(run_manager(states, recv));

  Ok(sender)
}

async fn run_cron(task: ScheduledTask, mut event_recv: ScheduleAdminReceiver) {
  let (timer_sender, mut timer_recv) = mpsc::channel(2);
  timer_sender.send(()).await.ok();
  let mut paused = false;
  loop {
    tokio::select! {
      _ = timer_recv.recv() => {
        if !paused {
          debug!("task {} run", task.name);
          match task.run().await {
            Ok(_) => {}
            Err(err) => {
              debug!("task {} error: {:?}", task.name, err);
            }
          }
        }
        let next_duration = task.next().unwrap_or(Duration::from_secs(1));
        let timer_sender = timer_sender.clone();
        tokio::spawn(async move {
          let _ = tokio::time::sleep(next_duration).await;
          let _ = timer_sender.send(()).await;
        });
      }
      v = event_recv.recv() => {
        info!("task {} event {:?}", task.name, v);
        match v {
          None => break,
          Some(ScheduleEvent::Pause(_)) => paused = true,
          Some(ScheduleEvent::Resume(_)) => paused = false,
          _ => {}
        };
      }
    };
  }
}

async fn invoke_matched(
  states: &IndexMap<String, ScheduleAdminSender>,
  name: &String,
  event: ScheduleEvent,
) {
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
      } else {
        break;
      }
    }
  } else {
    if let Some(sender) = states.get(name) {
      sender.send(event).await.ok();
    }
  }
}

async fn run_manager(
  states: IndexMap<String, ScheduleAdminSender>,
  mut recv: ScheduleAdminReceiver,
) {
  loop {
    match recv.recv().await {
      None => {
        debug!("manager stopped");
        break;
      }
      Some(ScheduleEvent::Pause(name)) => {
        invoke_matched(&states, &name, ScheduleEvent::Pause(name.clone())).await;
      }
      Some(ScheduleEvent::Resume(name)) => {
        invoke_matched(&states, &name, ScheduleEvent::Resume(name.clone())).await;
      }
      Some(ScheduleEvent::Stop(name)) => {
        if let Some(sender) = states.get(&name) {
          sender.send(ScheduleEvent::Stop(name)).await.ok();
        }
      }
      Some(ScheduleEvent::Reload(name)) => {
        if let Some(sender) = states.get(&name) {
          sender.send(ScheduleEvent::Reload(name)).await.ok();
        }
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
