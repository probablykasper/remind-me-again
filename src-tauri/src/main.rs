#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use notifications::{Data, Instance};
use std::sync::Mutex;
use std::thread;
use tauri::api::{dialog, shell};
#[cfg(target_os = "macos")]
use tauri::AboutMetadata;
use tauri::{
  command, AppHandle, CustomMenuItem, Manager, Menu, MenuEntry, MenuItem, State, Submenu,
  SystemTray, SystemTrayEvent, Window, WindowBuilder, WindowUrl,
};
use tauri_plugin_window_state::StateFlags;

#[macro_export]
macro_rules! throw {
  ($($arg:tt)*) => {{
    return Err(format!($($arg)*))
  }};
}

#[command]
#[specta::specta]
fn error_popup(msg: String, win: Window) {
  eprintln!("Error: {}", msg);
  thread::spawn(move || {
    dialog::message(Some(&win), "Error", msg);
  });
}

mod data;
mod notifications;

fn main() {
  let ctx = tauri::generate_context!();

  // macOS "App Nap" periodically pauses our app when it's in the background.
  // We need to prevent that so our intervals are not interrupted.
  #[cfg(target_os = "macos")]
  macos_app_nap::prevent();

  let app_paths = data::AppPaths::from_tauri_config(ctx.config());
  let mut error_msg = None;
  let reminders_file = match data::RemindersFile::load(&app_paths) {
    Ok(reminders_file) => reminders_file,
    Err(e) => {
      error_msg = Some(e);
      data::RemindersFile { groups: Vec::new() }
    }
  };

  if !cfg!(debug_assertions) {
    rust_macios::user_notifications::UNUserNotificationCenter::current_notification_center()
      .request_authorization_with_options_completion_handler(
        &[
          rust_macios::user_notifications::UNAuthorizationOptions::Alert,
          rust_macios::user_notifications::UNAuthorizationOptions::Sound,
        ],
        |granted: bool, error: Option<rust_macios::foundation::NSError>| {
          if !granted {
            println!("Notifications permission not granted");
          }
          if let Some(e) = error {
            println!(
              "Notification authorization error: {} {}",
              e.code(),
              e.localized_description()
            );
          }
        },
      );
  }

  let instance = Instance::init(
    reminders_file,
    app_paths,
    &ctx.config().tauri.bundle.identifier,
  );

  #[cfg(debug_assertions)]
  {
    tauri_specta::ts::export(
      specta::collect_types![
        error_popup,
        hide,
        notifications::new_group,
        notifications::get_groups,
        notifications::update_group,
        notifications::delete_group,
      ],
      "../bindings.ts",
    )
    .unwrap();
    println!("Generated TS types");
  }

  let app = tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
      error_popup,
      hide,
      notifications::new_group,
      notifications::get_groups,
      notifications::update_group,
      notifications::delete_group,
    ])
    .manage(Data(Mutex::new(instance)))
    .plugin(
      tauri_plugin_window_state::Builder::default()
        .with_state_flags(StateFlags::SIZE & StateFlags::POSITION)
        .build(),
    )
    .setup(|app| {
      #[cfg(target_os = "macos")]
      app.set_activation_policy(tauri::ActivationPolicy::Accessory);
      let win = create_window(&app.app_handle());
      match error_msg {
        Some(error_msg) => error_popup(error_msg, win),
        None => {}
      }
      Ok(())
    })
    .menu(Menu::with_items([
      #[cfg(target_os = "macos")]
      MenuEntry::Submenu(Submenu::new(
        &ctx.package_info().name,
        Menu::with_items([
          MenuItem::About(ctx.package_info().name.clone(), AboutMetadata::default()).into(),
          MenuItem::Separator.into(),
          MenuItem::Services.into(),
          MenuItem::Separator.into(),
          MenuItem::Hide.into(),
          MenuItem::HideOthers.into(),
          MenuItem::ShowAll.into(),
          MenuItem::Separator.into(),
          MenuItem::Quit.into(),
        ]),
      )),
      MenuEntry::Submenu(Submenu::new(
        "File",
        Menu::with_items([
          CustomMenuItem::new("New Reminder", "New Reminder")
            .accelerator("CmdOrCtrl+N")
            .into(),
          MenuItem::Separator.into(),
          CustomMenuItem::new("Edit Reminder", "Edit Reminder").into(),
          MenuItem::Separator.into(),
          MenuItem::CloseWindow.into(),
        ]),
      )),
      MenuEntry::Submenu(Submenu::new(
        "Edit",
        Menu::with_items([
          MenuItem::Undo.into(),
          MenuItem::Redo.into(),
          MenuItem::Separator.into(),
          MenuItem::Cut.into(),
          MenuItem::Copy.into(),
          MenuItem::Paste.into(),
          #[cfg(not(target_os = "macos"))]
          MenuItem::Separator.into(),
          MenuItem::SelectAll.into(),
        ]),
      )),
      MenuEntry::Submenu(Submenu::new(
        "View",
        Menu::with_items([MenuItem::EnterFullScreen.into()]),
      )),
      MenuEntry::Submenu(Submenu::new(
        "Window",
        Menu::with_items([MenuItem::Minimize.into(), MenuItem::Zoom.into()]),
      )),
      MenuEntry::Submenu(Submenu::new(
        "Help",
        Menu::with_items([CustomMenuItem::new("Learn More", "Learn More").into()]),
      )),
    ]))
    .on_menu_event(|event| {
      let event_name = event.menu_item_id();
      match event_name {
        "Learn More" => {
          let url = "https://github.com/probablykasper/remind-me-again".to_string();
          shell::open(&event.window().shell_scope(), url, None).unwrap();
        }
        _ => {}
      }
    })
    .system_tray(SystemTray::new())
    .on_system_tray_event(|app, event| match event {
      SystemTrayEvent::LeftClick { .. } => {
        let is_visible = app
          .get_window("main")
          .map(|w| w.is_visible().unwrap())
          .unwrap_or(false);
        if is_visible {
          hide(app.app_handle());
        } else {
          show(&app.app_handle());
        }
      }
      _ => {}
    })
    .build(ctx)
    .expect("error while running tauri application");
  {
    let data: State<Data> = app.state();
    let mut x = data.0.lock().unwrap();
    x.start();
  }

  app.run(|app_handle, e| match e {
    tauri::RunEvent::ExitRequested { api, .. } => {
      api.prevent_exit();
    }
    tauri::RunEvent::WindowEvent { event, .. } => match event {
      tauri::WindowEvent::CloseRequested { api, .. } => {
        api.prevent_close();
        hide(app_handle.app_handle());
      }
      _ => {}
    },
    _ => {}
  });
}

// Considerations for hide/show handling:
// - window.lose() causes memory leak: https://github.com/tauri-apps/wry/issues/590
// - Due to the accessory activation policy being enabled when the window hides, the underlying app's windows all get focusd. But the accessory policy is needed to control the dock.

fn show(app: &AppHandle) {
  let window = match app.get_window("main") {
    Some(window) => window,
    None => create_window(&app.app_handle()),
  };
  #[cfg(target_os = "macos")]
  {
    app.show().unwrap();
    set_is_accessory_policy(false);
  }
  window.show().unwrap();
  window.unminimize().unwrap();
  window.set_focus().unwrap();
}
#[command]
#[specta::specta]
fn hide(app: AppHandle) {
  let window = app.get_window("main").unwrap();
  window.unminimize().unwrap();
  window.hide().unwrap();
  #[cfg(target_os = "macos")]
  {
    app.hide().unwrap();
    set_is_accessory_policy(true);
  }
}

fn create_window(app: &AppHandle) -> Window {
  let win = WindowBuilder::new(app, "main", WindowUrl::default())
    .title("Remind Me Again")
    .inner_size(400.0, 550.0)
    .min_inner_size(400.0, 200.0)
    .visible(false)
    .skip_taskbar(true);

  #[cfg(target_os = "macos")]
  let win = win
    .transparent(true)
    .theme(Some(tauri::Theme::Dark))
    .hidden_title(true)
    .title_bar_style(tauri::TitleBarStyle::Transparent);

  let win = win.build().expect("Unable to create window");

  #[cfg(target_os = "macos")]
  {
    use cocoa::appkit::NSWindow;
    let nsw = win.ns_window().unwrap() as cocoa::base::id;
    unsafe {
      // set window background color
      let bg_color = cocoa::appkit::NSColor::colorWithRed_green_blue_alpha_(
        cocoa::base::nil,
        // also used in App.svelte
        8.0 / 255.0,
        9.0 / 255.0,
        13.0 / 255.0,
        1.0,
      );
      nsw.setBackgroundColor_(bg_color);
    }
  }
  win
}

#[cfg(target_os = "macos")]
fn set_is_accessory_policy(accessory: bool) {
  use cocoa::appkit::NSApplication;
  use cocoa::appkit::NSApplicationActivationPolicy::{
    NSApplicationActivationPolicyAccessory, NSApplicationActivationPolicyRegular,
  };
  use objc::*;

  let cls = objc::runtime::Class::get("NSApplication").unwrap();
  let app: cocoa::base::id = unsafe { msg_send![cls, sharedApplication] };
  unsafe {
    if accessory {
      app.setActivationPolicy_(NSApplicationActivationPolicyAccessory);
    } else {
      app.setActivationPolicy_(NSApplicationActivationPolicyRegular);
    }
  }
}
