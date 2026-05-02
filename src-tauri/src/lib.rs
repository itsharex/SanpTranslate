mod capture;
mod clipboard;
mod commands;
mod config;
mod error;
mod history;
mod hotkey;
mod ocr;
mod translate;
mod tray;
mod window;

use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .target(tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout))
                .target(tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Webview))
                .target(tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir { file_name: None }))
                .level(log::LevelFilter::Debug)
                .build(),
        )
        .manage(Mutex::new(window::PinImageStore::default()))
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::save_config,
            commands::capture_fullscreen,
            commands::capture_region,
            commands::write_clipboard_image,
            commands::read_clipboard_image,
            commands::write_clipboard_text,
            commands::create_pin_window,
            commands::close_pin_window,
            commands::get_pin_image
        ])
        .setup(|app| {
            let config_manager = config::ConfigManager::new(app.handle())?;
            let app_config = config_manager.load()?;

            tray::create_tray(app.handle(), &app_config.shortcuts)?;

            #[cfg(desktop)]
            hotkey::register_hotkeys(app.handle(), &app_config.shortcuts)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
