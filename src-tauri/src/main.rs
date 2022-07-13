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

#[macro_export]
macro_rules! throw {
  ($($arg:tt)*) => {{
    return Err(format!($($arg)*))
  }};
}

#[command]
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
  let instance = Instance {
    file: reminders_file,
    app_paths,
    scheduler: None,
    bundle_identifier: ctx.config().tauri.bundle.identifier.clone(),
  };

  let app = tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
      error_popup,
      notifications::new_group,
      notifications::get_groups,
      notifications::update_group,
      notifications::delete_group,
    ])
    .manage(Data(Mutex::new(instance)))
    .plugin(tauri_plugin_window_state::Builder::default().build())
    .setup(|app| {
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
        let window = match app.get_window("main") {
          Some(window) => match window.is_visible().expect("winvis") {
            true => {
              // hide the window instead of closing due to processes not closing memory leak: https://github.com/tauri-apps/wry/issues/590
              window.hide().expect("winhide");
              // window.close().expect("winclose");
              set_is_accessory_policy(true);
              return;
            }
            false => window,
          },
          None => create_window(&app.app_handle()),
        };
        set_is_accessory_policy(false);
        std::thread::sleep(std::time::Duration::from_millis(5));
        window.set_focus().unwrap();
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
        let window = app_handle.get_window("main").expect("getwin");
        // hide the window instead of closing due to processes not closing memory leak: https://github.com/tauri-apps/wry/issues/590
        window.hide().expect("winhide");
        // window.close().expect("winclose");
        set_is_accessory_policy(true);
      }
      _ => {}
    },
    _ => {}
  });
}

fn create_window(app: &AppHandle) -> Window {
  let win = WindowBuilder::new(app, "main", WindowUrl::default())
    .title("Remind Me Again")
    .inner_size(400.0, 550.0)
    .min_inner_size(400.0, 200.0)
    .skip_taskbar(true)
    .visible(false) // tauri_plugin_window_state reveals window
    .transparent(true)
    .build()
    .expect("Unable to create window");

  #[cfg(target_os = "macos")]
  {
    use cocoa::appkit::NSWindow;
    let nsw = win.ns_window().unwrap() as cocoa::base::id;
    unsafe {
      // manual implementation for now (PR https://github.com/tauri-apps/tauri/pull/3965)
      {
        nsw.setTitlebarAppearsTransparent_(cocoa::base::YES);

        // tauri enables fullsizecontentview by default, so disable it
        let mut style_mask = nsw.styleMask();
        style_mask.set(
          cocoa::appkit::NSWindowStyleMask::NSFullSizeContentViewWindowMask,
          false,
        );
        nsw.setStyleMask_(style_mask);
      }

      nsw.setTitleVisibility_(cocoa::appkit::NSWindowTitleVisibility::NSWindowTitleHidden);

      // set window to always be dark mode
      use cocoa::appkit::NSAppearanceNameVibrantDark;
      use objc::*;
      let appearance: cocoa::base::id = msg_send![
        class!(NSAppearance),
        appearanceNamed: NSAppearanceNameVibrantDark
      ];
      let () = msg_send![nsw, setAppearance: appearance];

      // set window background color
      let bg_color = cocoa::appkit::NSColor::colorWithRed_green_blue_alpha_(
        cocoa::base::nil,
        // 34.0 / 255.0 * 0.5,
        // 38.0 / 255.0 * 0.5,
        // 45.5 / 255.0 * 0.5,
        // 1.0,
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

#[allow(unused_variables)]
fn set_is_accessory_policy(accessory: bool) {
  #[cfg(target_os = "macos")]
  {
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
}
