use crate::data::{AppPaths, RemindersFile};
use async_cron_scheduler::cron;
use async_cron_scheduler::{Job, JobId, Scheduler};
use chrono::offset::Local;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::str::FromStr;
use std::sync::Mutex;
use tauri::api::dialog;
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

#[cfg(not(target_os = "macos"))]
static mut APP_IDENTIFIER: Option<String> = Option::None;

#[derive(Serialize, Deserialize, Clone, Type)]
pub struct Group {
  pub title: String,
  pub description: String,
  pub enabled: bool,
  pub id: String,
  #[serde(skip)]
  #[specta(type = null)]
  pub job_id: Option<JobId>,
  pub cron: String,
}
impl Group {
  pub fn create_job(&mut self, cron: cron::Schedule, scheduler: &mut Scheduler<Local>) {
    if self.enabled {
      let job = Job::cron_schedule(cron);
      let group = self.clone();
      let job_id = scheduler.insert(job, move |_id| {
        #[cfg(target_os = "macos")]
        let result = mac_notification_sys::send_notification(
          &group.title,
          None,
          &group.description,
          Some(mac_notification_sys::Notification::new().close_button("Done")),
        );

        #[cfg(not(target_os = "macos"))]
        let result = unsafe {
          tauri::api::notification::Notification::new(APP_IDENTIFIER.as_ref().unwrap())
            .title(&group.title)
            .body(&group.description)
            .show()
        };

        match result {
          Ok(_) => println!("Show \"{}\"", group.title),
          Err(e) => eprintln!("Could not show notification: {}", e),
        }
      });
      self.job_id = Some(job_id);
      println!("Create job \"{}\" at {}", self.title, self.cron);
    }
  }
}

pub struct Instance {
  file: RemindersFile,
  scheduler: Option<Scheduler<Local>>,
  app_paths: AppPaths,
}
impl Instance {
  pub fn init(file: RemindersFile, app_paths: AppPaths, app_identifier: &str) -> Self {
    #[cfg(target_os = "macos")]
    mac_notification_sys::set_application(&app_identifier).unwrap();
    #[cfg(not(target_os = "macos"))]
    unsafe {
      APP_IDENTIFIER = Some(app_identifier.to_string());
    }
    Self {
      file,
      scheduler: None,
      app_paths,
    }
  }
  pub fn save(&self) -> Result<(), String> {
    self.file.save(&self.app_paths)
  }
  pub fn add_group(&mut self, mut group: Group) -> Result<(), String> {
    match &mut self.scheduler {
      Some(scheduler) => {
        let schedule = parse_cron(&group.cron)?;
        group.create_job(schedule, scheduler);
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
    new_group.create_job(schedule, scheduler);
    Ok(())
  }
  pub fn update_job(&mut self, job_id: JobId, new_group: &mut Group) -> Result<(), String> {
    let schedule = parse_cron(&new_group.cron)?;
    let scheduler = match &mut self.scheduler {
      Some(scheduler) => scheduler,
      None => return Ok(()),
    };
    scheduler.remove(job_id);
    new_group.create_job(schedule, scheduler);
    println!("Update job \"{}\" at {}", new_group.title, new_group.cron);
    Ok(())
  }
  pub fn start(&mut self) {
    let (mut scheduler, sched_service) = Scheduler::<Local>::launch(tokio::time::sleep);

    let mut errors = Vec::new();
    for group in &mut self.file.groups {
      match parse_cron(&group.cron) {
        Ok(schedule) => group.create_job(schedule, &mut scheduler),
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
#[specta::specta]
pub async fn new_group(mut group: Group, data: State<'_, Data>) -> Result<Vec<Group>, String> {
  let mut data = data.0.lock().unwrap();
  group.id = data.generate_id();
  data.add_group(group)?;
  data.save()?;
  Ok(data.file.groups.clone())
}

#[command]
#[specta::specta]
pub async fn get_groups(data: State<'_, Data>) -> Result<Vec<Group>, String> {
  let data = data.0.lock().unwrap();
  Ok(data.file.groups.clone())
}

#[command]
#[specta::specta]
pub async fn update_group(mut group: Group, data: State<'_, Data>) -> Result<Vec<Group>, String> {
  let mut data = data.0.lock().unwrap();
  let i = match data.file.groups.iter().position(|g| g.id == group.id) {
    Some(i) => i,
    None => throw!("Group not found"),
  };
  match data.file.groups[i].job_id {
    Some(job_id) => data.update_job(job_id, &mut group)?,
    None => data.create_job(&mut group)?,
  }
  data.file.groups[i] = group;
  data.save()?;
  Ok(data.file.groups.clone())
}

#[command]
#[specta::specta]
pub async fn delete_group(index: u32, data: State<'_, Data>) -> Result<Vec<Group>, String> {
  let mut data = data.0.lock().unwrap();
  data.delete_group(index.try_into().unwrap());
  data.save()?;
  Ok(data.file.groups.clone())
}
