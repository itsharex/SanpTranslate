use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use image::DynamicImage;
use std::io::Cursor;
use tauri_plugin_clipboard_manager::ClipboardExt;

use crate::error::AppError;

/// 将 Base64 编码的图像数据写入系统剪贴板
/// 算法: Base64 解码 -> 解码为 DynamicImage -> 编码为 PNG bytes -> 构造 tauri Image -> 写入剪贴板
pub fn write_clipboard_image(app: &tauri::AppHandle, image_data: &str) -> Result<(), AppError> {
    // Base64 解码
    let bytes = BASE64
        .decode(image_data)
        .map_err(|e| AppError::ClipboardError(format!("Base64 解码失败: {}", e)))?;

    // 解码为 DynamicImage（支持任意图片格式）
    let img = image::load_from_memory(&bytes)
        .map_err(|e| AppError::ClipboardError(format!("图像解码失败: {}", e)))?;

    // 编码为 PNG bytes
    let mut png_buf = Cursor::new(Vec::new());
    img.write_to(&mut png_buf, image::ImageFormat::Png)
        .map_err(|e| AppError::ClipboardError(format!("PNG 编码失败: {}", e)))?;
    let png_bytes = png_buf.into_inner();

    // 从 PNG bytes 构造 tauri Image
    let tauri_image = tauri::image::Image::from_bytes(&png_bytes)
        .map_err(|e| AppError::ClipboardError(format!("构造 tauri Image 失败: {}", e)))?;

    // 写入剪贴板
    let clipboard = app.clipboard();
    clipboard
        .write_image(&tauri_image)
        .map_err(|e| AppError::ClipboardError(format!("写入剪贴板图像失败: {}", e)))?;

    Ok(())
}

/// 从系统剪贴板读取图像，返回 Base64 编码数据
/// 算法: 从剪贴板读取 tauri Image -> RGBA 数据转换为 PNG -> Base64 编码
/// 如果剪贴板中没有图像，返回 Ok(None)
pub fn read_clipboard_image(app: &tauri::AppHandle) -> Result<Option<String>, AppError> {
    let clipboard = app.clipboard();

    // 尝试读取图像，无图像时返回 Ok(None)
    let tauri_image = match clipboard.read_image() {
        Ok(img) => img,
        Err(_) => return Ok(None),
    };

    // 从 tauri::image::Image 提取 RGBA 数据
    let rgba_data = tauri_image.rgba();
    let width = tauri_image.width();
    let height = tauri_image.height();

    // 将 RGBA 数据构造为 RgbaImage，再转为 DynamicImage
    let rgba_image = image::RgbaImage::from_raw(width, height, rgba_data.to_vec())
        .ok_or_else(|| AppError::ClipboardError("RGBA 数据无法构造图像".to_string()))?;
    let dynamic_image = DynamicImage::ImageRgba8(rgba_image);

    // 编码为 PNG bytes
    let mut png_buf = Cursor::new(Vec::new());
    dynamic_image
        .write_to(&mut png_buf, image::ImageFormat::Png)
        .map_err(|e| AppError::ClipboardError(format!("PNG 编码失败: {}", e)))?;
    let png_bytes = png_buf.into_inner();

    // Base64 编码
    let base64_str = BASE64.encode(&png_bytes);

    Ok(Some(base64_str))
}

/// 将文本写入系统剪贴板
pub fn write_clipboard_text(app: &tauri::AppHandle, text: &str) -> Result<(), AppError> {
    let clipboard = app.clipboard();
    clipboard
        .write_text(text)
        .map_err(|e| AppError::ClipboardError(format!("写入剪贴板文本失败: {}", e)))?;

    Ok(())
}
