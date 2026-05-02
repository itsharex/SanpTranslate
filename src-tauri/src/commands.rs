// Tauri Command 定义模块

use crate::config::{AppConfig, ConfigManager};

/// 获取应用配置
#[tauri::command]
pub fn get_config(app: tauri::AppHandle) -> Result<AppConfig, String> {
    let manager = ConfigManager::new(&app).map_err(|e| e.to_string())?;
    manager.load().map_err(|e| e.to_string())
}

/// 保存应用配置
#[tauri::command]
pub fn save_config(app: tauri::AppHandle, config: AppConfig) -> Result<(), String> {
    let manager = ConfigManager::new(&app).map_err(|e| e.to_string())?;
    manager.save(&config).map_err(|e| e.to_string())
}
