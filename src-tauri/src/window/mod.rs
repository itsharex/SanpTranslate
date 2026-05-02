use crate::error::AppError;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

pub fn create_settings_window(app: &AppHandle) -> Result<(), AppError> {
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(());
    }

    WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("/settings".into()))
        .title("SnapTranslate - 设置")
        .inner_size(500.0, 600.0)
        .center()
        .resizable(true)
        .build()
        .map_err(|e| AppError::ConfigError(format!("创建设置窗口失败: {}", e)))?;

    Ok(())
}

pub fn create_history_window(app: &AppHandle) -> Result<(), AppError> {
    if let Some(window) = app.get_webview_window("history") {
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(());
    }

    WebviewWindowBuilder::new(app, "history", WebviewUrl::App("/history".into()))
        .title("SnapTranslate - 历史记录")
        .inner_size(600.0, 500.0)
        .center()
        .resizable(true)
        .build()
        .map_err(|e| AppError::ConfigError(format!("创建历史窗口失败: {}", e)))?;

    Ok(())
}

#[allow(dead_code)]
pub fn create_overlay_window(_app: &AppHandle) -> Result<(), AppError> {
    Ok(())
}

#[allow(dead_code)]
pub fn create_pin_window(
    _app: &AppHandle,
    _image_data: &str,
    _x: i32,
    _y: i32,
    _w: u32,
    _h: u32,
) -> Result<String, AppError> {
    Ok(String::new())
}

#[allow(dead_code)]
pub fn close_pin_window(_app: &AppHandle, _window_id: &str) -> Result<(), AppError> {
    Ok(())
}
