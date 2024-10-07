use std::{fs, path::PathBuf};

use anyhow::Result;
use croner::Cron;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use tokio::{sync::mpsc, time::Duration};
use tracing::{debug, info};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScheduledTask {
  pub name: String,
  #[serde(
    serialize_with = "serialize_cron",
    deserialize_with = "deserialize_cron"
  )]
  pub crons: Vec<Cron>,
  pub task: String,
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

pub(crate) async fn start(schedule_file: PathBuf) -> Result<ScheduleAdminSender> {
  let (sender, recv) = mpsc::channel(1024);
  let mut states = IndexMap::new();
  let schedules = fs::read_to_string(schedule_file)?;
  let schedules = serde_json::from_str::<Vec<ScheduledTask>>(&schedules)?;

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
        let next_duration = task.next().unwrap_or(Duration::from_secs(1));
        let timer_sender = timer_sender.clone();
        let name = task.name.clone();
        tokio::spawn(async move {
          if !paused {
            debug!(name, "run task")
          }else{
            debug!(name, "task paused")
          }
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
