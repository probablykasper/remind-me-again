use async_cron_scheduler::{Job, Scheduler};
use chrono::offset::Local;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Mutex;
use tauri::{command, State};

#[derive(Serialize, Deserialize)]
pub enum Repeat {
  #[serde(rename = "never")]
  Never,
  #[serde(rename = "daily")]
  Daily,
}

#[derive(Serialize, Deserialize)]
pub struct Group {
  pub title: String,
  pub description: String,
  pub enabled: bool,
  pub id: u64,
  pub cron: String,
  pub next_date: Option<u64>,
}
impl Group {
  pub fn create_job(&self) -> Result<Job<Local>, String> {
    match Job::cron(self.cron.as_str()) {
      Ok(job) => Ok(job),
      Err(e) => Err(e.to_string()),
    }
  }
}

pub struct Instance {
  pub scheduler: Option<Scheduler<Local>>,
  pub groups: Vec<Group>,
}
impl Instance {
  pub fn add_group(&mut self, group: Group) -> Result<(), String> {
    match &mut self.scheduler {
      Some(scheduler) => {
        let job = group.create_job()?;
        let _job_id = scheduler.insert(job, |id| {
          println!("Job {:?}", id);
        });
        println!("Scheduled \"{}\" at {}", group.title, group.cron);
        self.groups.push(group);
      }
      None => {
        self.groups.push(group);
        self.start()?;
      }
    };
    Ok(())
  }
  pub fn start(&mut self) -> Result<(), String> {
    let (mut scheduler, _sched_service) = Scheduler::<Local>::launch(tokio::time::sleep);

    let mut err = None;
    for group in &self.groups {
      if !group.enabled {
        continue;
      }
      let job = match Job::cron(group.cron.as_str()) {
        Ok(job) => job,
        Err(e) => {
          err = Some(e);
          continue;
        }
      };
      let _job_id = scheduler.insert(job, |id| {
        println!("Job {:?}", id);
      });
      println!("Scheduled \"{}\" at {}", group.title, group.cron);
    }

    self.scheduler = Some(scheduler);
    match err {
      Some(e) => Err(e.to_string()),
      None => Ok(()),
    }
  }
}

pub struct Data(pub Mutex<Instance>);

pub fn to_json<T: Serialize>(data: &T) -> Result<Value, String> {
  match serde_json::to_value(data) {
    Ok(v) => Ok(v),
    Err(e) => throw!("Error serializing {}", e),
  }
}

#[command]
pub async fn get_groups(data: State<'_, Data>) -> Result<Value, String> {
  let data = data.0.lock().unwrap();
  to_json(&data.groups)
}

#[command]
pub async fn new_group(mut group: Group, data: State<'_, Data>) -> Result<Value, String> {
  let mut data = data.0.lock().unwrap();
  group.id = data
    .groups
    .iter()
    .max_by_key(|g| g.id)
    .map(|g| g.id + 1)
    .unwrap_or(0);
  data.add_group(group)?;
  to_json(&data.groups)
}
