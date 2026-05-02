use crate::error::AppError;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use uuid::Uuid;

pub struct PinImageStore {
    pub images: HashMap<String, String>,
}

impl Default for PinImageStore {
    fn default() -> Self {
        Self {
            images: HashMap::new(),
        }
    }
}

#[derive(serde::Serialize)]
pub struct PinWindowInfo {
    pub label: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

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
        .map_err(|e| AppError::ConfigError(format!("创建历史记录窗口失败: {}", e)))?;

    Ok(())
}

pub fn create_overlay_window(app: &AppHandle, image_data: &str) -> Result<(), AppError> {
    log::info!("[OVERLAY] 开始创建 overlay 窗口，图片数据长度={}", image_data.len());

    if let Some(existing) = app.get_webview_window("overlay") {
        log::warn!("[OVERLAY] 已存在 overlay 窗口，先销毁");
        match existing.destroy() {
            Ok(_) => log::info!("[OVERLAY] 旧 overlay 窗口销毁成功"),
            Err(e) => log::error!("[OVERLAY] 旧 overlay 窗口销毁失败: {}", e),
        }
    }

    let monitor = app.primary_monitor()
        .ok()
        .flatten()
        .ok_or_else(|| AppError::ConfigError("获取主显示器信息失败".to_string()))?;

    let scale_factor = monitor.scale_factor();
    let monitor_x = monitor.position().x as f64 / scale_factor;
    let monitor_y = monitor.position().y as f64 / scale_factor;
    let monitor_w = monitor.size().width as f64 / scale_factor;
    let monitor_h = monitor.size().height as f64 / scale_factor;

    log::info!("[OVERLAY] 显示器信息: scale_factor={}, pos=({},{}) size=({},{})",
        scale_factor, monitor_x, monitor_y, monitor_w, monitor_h);

    let window = WebviewWindowBuilder::new(app, "overlay", WebviewUrl::App("/overlay".into()))
        .title("SnapTranslate - 截图蒙版")
        .decorations(false)
        .always_on_top(true)
        .transparent(true)
        .shadow(false)
        .focusable(true)
        .resizable(false)
        .position(monitor_x, monitor_y)
        .inner_size(monitor_w, monitor_h)
        .build()
        .map_err(|e| {
            log::error!("[OVERLAY] 创建窗口失败: {}", e);
            AppError::ConfigError(format!("创建蒙版窗口失败: {}", e))
        })?;

    log::info!("[OVERLAY] 窗口创建成功，label={}", window.label());

    let app_handle = app.clone();
    let image_data_owned = image_data.to_string();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(200));
        log::info!("[OVERLAY] 延迟200ms后发送 overlay-image 事件，数据长度={}", image_data_owned.len());
        if let Err(e) = app_handle.emit("overlay-image", &image_data_owned) {
            log::error!("[OVERLAY] 发送截图数据失败: {}", e);
        } else {
            log::info!("[OVERLAY] overlay-image 事件发送成功");
        }
    });

    Ok(())
}

pub fn prepare_pin_window(
    app: &AppHandle,
    image_data: &str,
    x: i32,
    y: i32,
    w: u32,
    h: u32,
) -> Result<PinWindowInfo, AppError> {
    // 阴影内边距，需与前端 PinView.vue 中的 PIN_PADDING 保持一致
    const PIN_PADDING: f64 = 4.0;
    const CONTROL_BAR_H: f64 = 36.0;

    let label = format!("pin-{}", Uuid::new_v4());
    log::info!("[PIN] 准备贴图窗口 label={}, x={}, y={}, w={}, h={}, 图片数据长度={}",
        label, x, y, w, h, image_data.len());

    {
        let store = app.state::<Mutex<PinImageStore>>();
        let mut store = store.lock().map_err(|e| {
            log::error!("[PIN] 锁定 PinImageStore 失败: {}", e);
            AppError::ConfigError(format!("锁定状态失败: {}", e))
        })?;
        store.images.insert(label.clone(), image_data.to_string());
        log::info!("[PIN] 图片数据已存入 store，当前 store 中有 {} 条记录", store.images.len());
    }

    let scale_factor = app.primary_monitor()
        .ok()
        .flatten()
        .map(|m| m.scale_factor())
        .unwrap_or(1.0);

    let logical_x = x as f64 / scale_factor;
    let logical_y = y as f64 / scale_factor;
    let logical_w = w as f64 / scale_factor;
    let logical_h = h as f64 / scale_factor;

    log::info!("[PIN] DPI转换: scale_factor={}, logical=({},{},{},{})",
        scale_factor, logical_x, logical_y, logical_w, logical_h);

    let info = PinWindowInfo {
        label,
        x: logical_x,
        y: logical_y,
        width: logical_w + PIN_PADDING * 2.0,
        height: logical_h + CONTROL_BAR_H + PIN_PADDING * 2.0,
    };

    log::info!("[PIN] 准备完成，返回窗口信息: label={}, pos=({},{}) size=({},{})",
        info.label, info.x, info.y, info.width, info.height);

    Ok(info)
}

pub fn create_pin_window_on_main_thread(
    app: &AppHandle,
    image_data: &str,
    x: i32,
    y: i32,
    w: u32,
    h: u32,
) -> Result<String, AppError> {
    let info = prepare_pin_window(app, image_data, x, y, w, h)?;

    let _window = WebviewWindowBuilder::new(app, &info.label, WebviewUrl::App("/pin".into()))
        .title("SnapTranslate - 贴图")
        .decorations(false)
        .always_on_top(true)
        .transparent(true)
        .shadow(false)
        .skip_taskbar(true)
        .resizable(false)
        .position(info.x, info.y)
        .inner_size(info.width, info.height)
        .build()
        .map_err(|e| {
            log::error!("[PIN] 创建贴图窗口失败: {}", e);
            AppError::ConfigError(format!("创建贴图窗口失败: {}", e))
        })?;

    log::info!("[PIN] 贴图窗口创建成功 label={}", info.label);
    Ok(info.label)
}

pub fn get_pin_image(app: &AppHandle, window_id: &str) -> Result<Option<String>, AppError> {
    log::info!("[PIN] get_pin_image 被调用，window_id={}", window_id);

    let store = app.state::<Mutex<PinImageStore>>();
    let mut store = store.lock().map_err(|e| {
        log::error!("[PIN] 锁定 PinImageStore 失败: {}", e);
        AppError::ConfigError(format!("锁定状态失败: {}", e))
    })?;

    let result = store.images.remove(window_id);
    match &result {
        Some(data) => log::info!("[PIN] 找到图片数据，长度={}", data.len()),
        None => log::error!("[PIN] 未找到图片数据！store 中现有的 key: {:?}", store.images.keys().collect::<Vec<_>>()),
    }

    Ok(result)
}

pub fn close_pin_window(app: &AppHandle, window_id: &str) -> Result<(), AppError> {
    log::info!("[PIN] close_pin_window 被调用，window_id={}", window_id);

    if let Some(window) = app.get_webview_window(window_id) {
        window.destroy().map_err(|e| {
            log::error!("[PIN] 关闭贴图窗口 {} 失败: {}", window_id, e);
            AppError::ConfigError(format!("关闭贴图窗口 {} 失败: {}", window_id, e))
        })?;
        log::info!("[PIN] 贴图窗口 {} 已销毁", window_id);
    } else {
        log::warn!("[PIN] 未找到贴图窗口 {}", window_id);
    }
    Ok(())
}
