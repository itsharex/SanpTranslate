use crate::config::{AppConfig, ConfigManager};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use image::ImageEncoder;
use tauri::Manager;

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
pub fn close_pin_window(window_id: String, app: tauri::AppHandle) -> Result<(), String> {
    crate::window::close_pin_window(&app, &window_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_pin_image(window_id: String, app: tauri::AppHandle) -> Result<Option<String>, String> {
    crate::window::get_pin_image(&app, &window_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn capture_region_from_cache(
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    app: tauri::AppHandle,
) -> Result<crate::window::CropResult, String> {
    log::info!("[CMD] capture_region_from_cache 被调用，x={}, y={}, w={}, h={}", x, y, width, height);

    let cached_screen = {
        let store = app.state::<std::sync::Mutex<crate::window::CachedScreenStore>>();
        let mut store = store.lock().map_err(|e| {
            log::error!("[CMD] 锁定 CachedScreenStore 失败: {}", e);
            e.to_string()
        })?;
        store.screen.take()
    };

    let mut screen = cached_screen.ok_or_else(|| {
        log::error!("[CMD] 缓存中无全屏截图数据");
        "缓存中无全屏截图数据，请重新截图".to_string()
    })?;

    let cropped = image::imageops::crop(&mut screen.image, x, y, width, height);
    let cropped_image = cropped.to_image();

    let rgba_for_clipboard = cropped_image.as_raw().clone();
    let crop_w = cropped_image.width();
    let crop_h = cropped_image.height();

    let png_bytes = encode_png_fast(&cropped_image).map_err(|e| {
        log::error!("[CMD] PNG 编码失败: {}", e);
        e.to_string()
    })?;

    let base64_data = STANDARD.encode(&png_bytes);

    // 异步写入剪贴板（不阻塞返回）
    let app_clone = app.clone();
    std::thread::spawn(move || {
        match crate::clipboard::write_clipboard_image_raw(&app_clone, rgba_for_clipboard, crop_w, crop_h) {
            Ok(_) => log::info!("[CMD] 异步剪贴板写入成功"),
            Err(e) => log::error!("[CMD] 异步剪贴板写入失败: {}", e),
        }
    });

    let scale_factor = screen.scale_factor;
    let logical_x = (screen.monitor_x + x as i32) as f64 / scale_factor;
    let logical_y = (screen.monitor_y + y as i32) as f64 / scale_factor;
    let logical_w = width as f64 / scale_factor;
    let logical_h = height as f64 / scale_factor;

    // 阴影内边距和控制栏高度，需与前端 PinView.vue 保持一致
    const PIN_PADDING: f64 = 4.0;
    const CONTROL_BAR_H: f64 = 36.0;

    let result = crate::window::CropResult {
        base64_data,
        x: logical_x,
        y: logical_y,
        width: logical_w + PIN_PADDING * 2.0,
        height: logical_h + CONTROL_BAR_H + PIN_PADDING * 2.0,
        crop_width: width,
        crop_height: height,
    };

    log::info!("[CMD] capture_region_from_cache 完成，返回 CropResult");
    Ok(result)
}

/// 使用快速压缩级别编码 PNG，性能远优于默认压缩
fn encode_png_fast(image: &image::RgbaImage) -> Result<Vec<u8>, String> {
    let mut buf = std::io::Cursor::new(Vec::new());
    let encoder = image::codecs::png::PngEncoder::new_with_quality(
        &mut buf,
        image::codecs::png::CompressionType::Fast,
        image::codecs::png::FilterType::Sub,
    );
    encoder
        .write_image(
            image.as_raw(),
            image.width(),
            image.height(),
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|e| format!("PNG 快速编码失败: {}", e))?;
    Ok(buf.into_inner())
}

#[tauri::command]
pub fn get_overlay_image(app: tauri::AppHandle) -> Result<Option<crate::window::OverlayImageData>, String> {
    let store = app.state::<std::sync::Mutex<crate::window::CachedScreenStore>>();
    let mut store = store.lock().map_err(|e| {
        log::error!("[CMD] 锁定 CachedScreenStore 失败: {}", e);
        e.to_string()
    })?;
    Ok(store.overlay_image.take())
}

#[tauri::command]
pub fn store_pin_image(label: String, image_data: String, app: tauri::AppHandle) -> Result<(), String> {
    let store = app.state::<std::sync::Mutex<crate::window::PinImageStore>>();
    let mut store = store.lock().map_err(|e| {
        log::error!("[CMD] 锁定 PinImageStore 失败: {}", e);
        e.to_string()
    })?;
    store.images.insert(label.clone(), image_data);
    log::info!("[CMD] store_pin_image: 图像数据已存储，label={}, store中共{}条记录", label, store.images.len());
    Ok(())
}
