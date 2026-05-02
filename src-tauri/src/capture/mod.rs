use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use image::DynamicImage;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use xcap::Monitor;

use crate::error::AppError;

/// 图像格式
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageFormat {
    Png,
    Jpeg,
}

/// 显示器信息
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorInfo {
    pub id: String,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub is_primary: bool,
}

/// 截图区域
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureRegion {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub monitor_id: Option<String>,
}

/// 截图结果
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CapturedImage {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: ImageFormat,
}

/// 截图服务
pub struct CaptureService {
    monitors: Vec<Monitor>,
}

impl CaptureService {
    /// 创建截图服务实例，获取显示器列表
    pub fn new() -> Result<Self, AppError> {
        let monitors = Monitor::all()
            .map_err(|e| AppError::CaptureError(format!("获取显示器列表失败: {}", e)))?;
        Ok(Self { monitors })
    }

    /// 返回所有显示器信息
    #[allow(dead_code)]
    pub fn list_monitors(&self) -> Vec<MonitorInfo> {
        self.monitors
            .iter()
            .filter_map(|m| {
                Some(MonitorInfo {
                    id: m.id().ok()?.to_string(),
                    name: m.friendly_name().unwrap_or_else(|_| "未知显示器".to_string()),
                    x: m.x().ok()?,
                    y: m.y().ok()?,
                    width: m.width().ok()?,
                    height: m.height().ok()?,
                    is_primary: m.is_primary().unwrap_or(false),
                })
            })
            .collect()
    }

    /// 截取全屏截图，返回 Base64 编码的 PNG
    pub fn capture_fullscreen(&self, monitor_id: Option<&str>) -> Result<String, AppError> {
        let monitor = self.find_monitor(monitor_id)?;
        let image = monitor
            .capture_image()
            .map_err(|e| AppError::CaptureError(format!("截取全屏失败: {}", e)))?;

        let dynamic_image = DynamicImage::ImageRgba8(image);
        encode_to_base64_png(&dynamic_image)
    }

    /// 截取区域截图，返回 Base64 编码的 PNG
    pub fn capture_region(&self, region: &CaptureRegion) -> Result<String, AppError> {
        let monitor = self.find_monitor(region.monitor_id.as_deref())?;
        let full_image = monitor
            .capture_image()
            .map_err(|e| AppError::CaptureError(format!("截取全屏失败: {}", e)))?;

        let mut dynamic_image = DynamicImage::ImageRgba8(full_image);

        // 将绝对坐标转换为显示器相对坐标
        let monitor_x = monitor.x().map_err(|e| AppError::CaptureError(e.to_string()))?;
        let monitor_y = monitor.y().map_err(|e| AppError::CaptureError(e.to_string()))?;

        let relative_x = (region.x - monitor_x).max(0) as u32;
        let relative_y = (region.y - monitor_y).max(0) as u32;

        let cropped = dynamic_image.crop(relative_x, relative_y, region.width, region.height);
        encode_to_base64_png(&cropped)
    }

    /// 根据 ID 查找显示器，若未指定则返回主显示器
    fn find_monitor(&self, monitor_id: Option<&str>) -> Result<&Monitor, AppError> {
        match monitor_id {
            Some(id) => self
                .monitors
                .iter()
                .find(|m| m.id().map(|i| i.to_string() == id).unwrap_or(false))
                .ok_or_else(|| AppError::CaptureError(format!("未找到ID为 {} 的显示器", id))),
            None => self
                .monitors
                .iter()
                .find(|m| m.is_primary().unwrap_or(false))
                .or_else(|| self.monitors.first())
                .ok_or_else(|| AppError::CaptureError("未找到可用显示器".to_string())),
        }
    }
}

/// 将 DynamicImage 编码为 Base64 PNG 字符串
fn encode_to_base64_png(image: &DynamicImage) -> Result<String, AppError> {
    let mut buf = Cursor::new(Vec::new());
    image
        .write_to(&mut buf, image::ImageFormat::Png)
        .map_err(|e| AppError::CaptureError(format!("PNG编码失败: {}", e)))?;
    Ok(STANDARD.encode(buf.get_ref()))
}
