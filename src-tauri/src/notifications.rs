use chrono;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Mutex;
use tauri::{command, State};
use tokio_cron_scheduler::{Job, JobScheduler};

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
  pub repeat: Repeat,
  pub next_date: Option<u64>,
}

pub struct Instance {
  pub job_scheduler: JobScheduler,
  pub groups: Vec<Group>,
}
impl Instance {
  pub async fn init(groups: Vec<Group>) -> Instance {
    let sched = JobScheduler::new().expect("new sched");

    sched
      .add(
        Job::new("0 * * * * *", |_uuid, _l| {
          println!("I run every 5 seconds {}", chrono::offset::Local::now());
        })
        .expect("new job"),
      )
      .expect("add job");

    sched.start().expect("start sched");

    Instance {
      job_scheduler: sched,
      groups: groups,
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
  data.groups.push(group);
  to_json(&data.groups)
}
