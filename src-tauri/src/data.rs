use crate::notifications::Group;
use atomicwrites::{AtomicFile, OverwriteBehavior};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::io::Write;
use std::path::PathBuf;
use tauri::Config;

#[derive(Clone)]
pub struct AppPaths {
  pub app_dir: PathBuf,
  pub reminders_file: PathBuf,
}
impl AppPaths {
  pub fn from_tauri_config(config: &Config) -> Self {
    let app_dir = match env::var("DEVELOPMENT").is_ok() {
      true => env::current_dir().unwrap().join("appdata"),
      false => tauri::api::path::app_dir(config).unwrap(),
    };
    AppPaths {
      app_dir: app_dir.clone(),
      reminders_file: app_dir.join("Settings.json"),
    }
  }
}

pub fn to_json<T: Serialize>(data: &T) -> Result<Value, String> {
  match serde_json::to_value(data) {
    Ok(v) => Ok(v),
    Err(e) => throw!("Error serializing {}", e),
  }
}

pub fn ensure_parent_exists(file_path: &PathBuf) -> Result<(), String> {
  if let Some(parent) = file_path.parent() {
    if let Err(e) = std::fs::create_dir_all(parent) {
      throw!("Error creating parent folder: {}", e.to_string());
    }
  }
  Ok(())
}

pub fn write_atomically(file_path: &PathBuf, buf: &[u8]) -> Result<(), String> {
  ensure_parent_exists(&file_path)?;
  let af = AtomicFile::new(&file_path, OverwriteBehavior::AllowOverwrite);
  match af.write(|f| f.write_all(&buf)) {
    Ok(_) => Ok(()),
    Err(e) => Err(e.to_string()),
  }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RemindersFile {
  pub groups: Vec<Group>,
}
impl RemindersFile {
  pub fn load(paths: &AppPaths) -> Result<Self, String> {
    let reminders = match std::fs::read_to_string(&paths.reminders_file) {
      Ok(reminders_str) => {
        let reminders: RemindersFile = match serde_json::from_str(&reminders_str) {
          Ok(reminders) => reminders,
          Err(e) => throw!("Could not parse reminders file: {}", e),
        };
        reminders
      }
      Err(e) => match e.kind() {
        std::io::ErrorKind::NotFound => RemindersFile { groups: Vec::new() },
        _ => throw!("{}", e.to_string()),
      },
    };
    Ok(reminders)
  }

  pub fn save(&self, paths: &AppPaths) -> Result<(), String> {
    let mut json = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"\t");
    let mut ser = serde_json::Serializer::with_formatter(&mut json, formatter);
    match self.serialize(&mut ser) {
      Ok(_) => {}
      Err(e) => throw!("Error saving content: {}", e.to_string()),
    }
    match write_atomically(&paths.reminders_file, &json) {
      Ok(_) => {}
      Err(e) => throw!("Error saving: {}", e.to_string()),
    }
    Ok(())
  }
}
