use crate::error::AppError;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::AppHandle;

// 创建系统托盘图标及菜单
pub fn create_tray(app: &AppHandle) -> Result<(), AppError> {
    let capture_item = MenuItem::with_id(
        app,
        "capture",
        "框选截图翻译  Ctrl+Shift+X",
        true,
        None::<&str>,
    )?;
    let pin_clipboard_item = MenuItem::with_id(
        app,
        "pin_clipboard",
        "从剪贴板贴图  Ctrl+Shift+V",
        true,
        None::<&str>,
    )?;
    let separator1 = PredefinedMenuItem::separator(app)?;
    let translate_recent_item = MenuItem::with_id(
        app,
        "translate_recent",
        "翻译最近一张贴图",
        true,
        None::<&str>,
    )?;
    let history_item = MenuItem::with_id(app, "history", "截图与翻译历史", true, None::<&str>)?;
    let separator2 = PredefinedMenuItem::separator(app)?;
    let settings_item = MenuItem::with_id(app, "settings", "设置", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &capture_item,
            &pin_clipboard_item,
            &separator1,
            &translate_recent_item,
            &history_item,
            &separator2,
            &settings_item,
            &quit_item,
        ],
    )?;

    // 加载默认托盘图标
    let icon = tauri::image::Image::from_bytes(include_bytes!("../../icons/icon.png"))?;

    TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "capture" => {
                // S2 阶段实现截图功能
            }
            "pin_clipboard" => {
                // S2 阶段实现剪贴板贴图功能
            }
            "translate_recent" => {
                // S3 阶段实现翻译最近贴图功能
            }
            "history" => {
                if let Err(e) = crate::window::create_history_window(app) {
                    eprintln!("创建历史窗口失败: {}", e);
                }
            }
            "settings" => {
                if let Err(e) = crate::window::create_settings_window(app) {
                    eprintln!("创建设置窗口失败: {}", e);
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    Ok(())
}
