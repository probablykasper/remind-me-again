use crate::data::{to_json, AppPaths, RemindersFile};
use async_cron_scheduler::{Job, JobId, Scheduler};
use chrono::offset::Local;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::str::FromStr;
use std::sync::Mutex;
use tauri::api::dialog;
use tauri::api::notification::Notification;
use tauri::{command, State};

#[derive(Serialize, Deserialize)]
pub enum Repeat {
  #[serde(rename = "never")]
  Never,
  #[serde(rename = "daily")]
  Daily,
}

pub fn parse_cron(s: &str) -> Result<cron::Schedule, String> {
  match cron::Schedule::from_str(s) {
    Ok(schedule) => Ok(schedule),
    Err(e) => throw!("Invalid schedule: {}", e),
  }
}

#[derive(Serialize, Deserialize, Clone)]
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
  pub fn create_job(&mut self, cron: cron::Schedule, scheduler: &mut Scheduler<Local>, a: String) {
    if self.enabled {
      let job = Job::cron_schedule(cron);
      let group = self.clone();
      let job_id = scheduler.insert(job, move |_id| {
        let result = Notification::new(&a)
          .title(&group.title)
          .body(&group.description)
          .show();
        match result {
          Ok(_) => println!("Show \"{}\"", group.title),
          Err(e) => eprintln!("Could not show notification: {}", e),
        }
      });
      self.job_id = Some(job_id);
      println!("Created job \"{}\" at {}", self.title, self.cron);
    }
  }
}

pub struct Instance {
  pub file: RemindersFile,
  pub scheduler: Option<Scheduler<Local>>,
  pub app_paths: AppPaths,
  pub bundle_identifier: String,
}
impl Instance {
  pub fn save(&self) -> Result<(), String> {
    self.file.save(&self.app_paths)
  }
  pub fn add_group(&mut self, mut group: Group) -> Result<(), String> {
    match &mut self.scheduler {
      Some(scheduler) => {
        let schedule = parse_cron(&group.cron)?;
        group.create_job(schedule, scheduler, self.bundle_identifier.clone());
        self.file.groups.push(group);
      }
      None => {
        self.file.groups.push(group);
        self.start();
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
      let exists = self.file.groups.iter().any(|g| g.id == id);
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
        self.file.groups.remove(index);
        return;
      }
    };
    match self.file.groups[index].job_id {
      Some(job_id) => scheduler.remove(job_id),
      None => {}
    };
    self.file.groups.remove(index);
  }
  pub fn create_job(&mut self, new_group: &mut Group) -> Result<(), String> {
    let schedule = parse_cron(&new_group.cron)?;
    let scheduler = match &mut self.scheduler {
      Some(scheduler) => scheduler,
      None => return Ok(()),
    };
    new_group.create_job(schedule, scheduler, self.bundle_identifier.clone());
    Ok(())
  }
  pub fn replace_job(&mut self, job_id: JobId, new_group: &mut Group) -> Result<(), String> {
    let schedule = parse_cron(&new_group.cron)?;
    let scheduler = match &mut self.scheduler {
      Some(scheduler) => scheduler,
      None => return Ok(()),
    };
    scheduler.remove(job_id);
    new_group.create_job(schedule, scheduler, self.bundle_identifier.clone());
    Ok(())
  }
  pub fn start(&mut self) {
    let bundle_identifier = self.bundle_identifier.clone();

    let (mut scheduler, sched_service) = Scheduler::<Local>::launch(tokio::time::sleep);

    let mut errors = Vec::new();
    for group in &mut self.file.groups {
      match parse_cron(&group.cron) {
        Ok(schedule) => group.create_job(schedule, &mut scheduler, bundle_identifier.clone()),
        Err(e) => errors.push(e),
      };
    }
    match errors.get(0) {
      Some(e) => {
        dialog::MessageDialogBuilder::new("Error", e).show(|_| {});
      }
      None => {}
    }

    self.scheduler = Some(scheduler);
    tauri::async_runtime::spawn(sched_service);
  }
}

pub struct Data(pub Mutex<Instance>);

#[command]
pub async fn new_group(mut group: Group, data: State<'_, Data>) -> Result<Value, String> {
  let mut data = data.0.lock().unwrap();
  group.id = data.generate_id();
  data.add_group(group)?;
  data.save()?;
  to_json(&data.file.groups)
}

#[command]
pub async fn get_groups(data: State<'_, Data>) -> Result<Value, String> {
  let data = data.0.lock().unwrap();
  to_json(&data.file.groups)
}

#[command]
pub async fn update_group(mut group: Group, data: State<'_, Data>) -> Result<Value, String> {
  let mut data = data.0.lock().unwrap();
  let i = match data.file.groups.iter().position(|g| g.id == group.id) {
    Some(i) => i,
    None => throw!("Group not found"),
  };
  match data.file.groups[i].job_id {
    Some(job_id) => data.replace_job(job_id, &mut group)?,
    None => data.create_job(&mut group)?,
  }
  data.file.groups[i] = group;
  data.save()?;
  to_json(&data.file.groups)
}

#[command]
pub async fn delete_group(index: usize, data: State<'_, Data>) -> Result<Value, String> {
  let mut data = data.0.lock().unwrap();
  data.delete_group(index);
  data.save()?;
  to_json(&data.file.groups)
}
