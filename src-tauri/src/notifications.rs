use async_cron_scheduler::{Job, JobId, Scheduler};
use chrono::offset::Local;
use nanoid::nanoid;
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
  pub id: String,
  #[serde(skip)]
  pub job_id: Option<JobId>,
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
  pub fn generate_id(&self) -> String {
    let alphabet: [char; 32] = [
      'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
      's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '2', '3', '4', '5', '6', '7',
    ];
    for _ in 0..100 {
      let id = nanoid!(7, &alphabet);
      let exists = self.groups.iter().any(|g| g.id == id);
      if !exists {
        return id;
      }
    }
    panic!("Error generating ID: Generated IDs already exist")
  }
  pub fn delete_group(&mut self, index: usize) {
    let scheduler = match &mut self.scheduler {
      Some(scheduler) => scheduler,
      None => {
        self.groups.remove(index);
        return;
      }
    };
    match self.groups[index].job_id {
      Some(job_id) => scheduler.remove(job_id),
      None => {}
    };
    self.groups.remove(index);
  }
  pub fn start(&mut self) -> Result<(), String> {
    let (mut scheduler, _sched_service) = Scheduler::<Local>::launch(tokio::time::sleep);

    let mut err = None;
    for group in &mut self.groups {
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
      let job_id = scheduler.insert(job, |id| {
        println!("Job {:?}", id);
      });
      group.job_id = Some(job_id);
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
  group.id = data.generate_id();
  data.add_group(group)?;
  to_json(&data.groups)
}

#[command]
pub async fn delete_group(index: usize, data: State<'_, Data>) -> Result<Value, String> {
  let mut data = data.0.lock().unwrap();
  data.delete_group(index);
  to_json(&data.groups)
}
