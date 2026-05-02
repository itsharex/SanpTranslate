use crate::capture::{CaptureRegion, CaptureService};
use crate::config::{AppConfig, ConfigManager};
use crate::window::PinWindowInfo;

#[tauri::command]
pub fn get_config(app: tauri::AppHandle) -> Result<AppConfig, String> {
    let manager = ConfigManager::new(&app).map_err(|e| e.to_string())?;
    manager.load().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_config(app: tauri::AppHandle, config: AppConfig) -> Result<(), String> {
    let manager = ConfigManager::new(&app).map_err(|e| e.to_string())?;
    manager.save(&config).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn capture_fullscreen(
    monitor_id: Option<String>,
    _app: tauri::AppHandle,
) -> Result<String, String> {
    log::info!("[CMD] capture_fullscreen 被调用，monitor_id={:?}", monitor_id);
    let service = CaptureService::new().map_err(|e| {
        log::error!("[CMD] 创建截图服务失败: {}", e);
        e.to_string()
    })?;
    let result = service
        .capture_fullscreen(monitor_id.as_deref())
        .map_err(|e| {
            log::error!("[CMD] 全屏截图失败: {}", e);
            e.to_string()
        })?;
    log::info!("[CMD] 全屏截图成功，数据长度={}", result.len());
    Ok(result)
}

#[tauri::command]
pub fn capture_region(
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    monitor_id: Option<String>,
    _app: tauri::AppHandle,
) -> Result<String, String> {
    log::info!("[CMD] capture_region 被调用，x={}, y={}, w={}, h={}", x, y, width, height);
    let service = CaptureService::new().map_err(|e| e.to_string())?;
    let region = CaptureRegion {
        x,
        y,
        width,
        height,
        monitor_id,
    };
    service.capture_region(&region).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn write_clipboard_image(image_data: String, app: tauri::AppHandle) -> Result<(), String> {
    log::info!("[CMD] write_clipboard_image 被调用，数据长度={}", image_data.len());
    crate::clipboard::write_clipboard_image(&app, &image_data).map_err(|e| {
        log::error!("[CMD] 写入剪贴板失败: {}", e);
        e.to_string()
    })
}

#[tauri::command]
pub fn read_clipboard_image(app: tauri::AppHandle) -> Result<Option<String>, String> {
    crate::clipboard::read_clipboard_image(&app).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn write_clipboard_text(text: String, app: tauri::AppHandle) -> Result<(), String> {
    crate::clipboard::write_clipboard_text(&app, &text).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_pin_window(
    image_data: String,
    x: i32,
    y: i32,
    w: u32,
    h: u32,
    app: tauri::AppHandle,
) -> Result<PinWindowInfo, String> {
    log::info!("[CMD] create_pin_window 命令被调用，x={}, y={}, w={}, h={}, 数据长度={}",
        x, y, w, h, image_data.len());
    let result = crate::window::prepare_pin_window(&app, &image_data, x, y, w, h).map_err(|e| {
        log::error!("[CMD] prepare_pin_window 失败: {}", e);
        e.to_string()
    })?;
    log::info!("[CMD] prepare_pin_window 成功，返回 label={}", result.label);
    Ok(result)
}

#[tauri::command]
pub fn close_pin_window(window_id: String, app: tauri::AppHandle) -> Result<(), String> {
    crate::window::close_pin_window(&app, &window_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_pin_image(window_id: String, app: tauri::AppHandle) -> Result<Option<String>, String> {
    crate::window::get_pin_image(&app, &window_id).map_err(|e| e.to_string())
}
