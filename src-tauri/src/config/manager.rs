// 配置管理器实现

use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

/// 翻译模式
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum TranslateMode {
    #[default]
    Ocr,
    Multimodal,
}

/// 快捷键配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutConfig {
    /// 截图翻译快捷键
    pub capture: String,
    /// 固定到剪贴板快捷键
    pub pin_clipboard: String,
}

impl Default for ShortcutConfig {
    fn default() -> Self {
        ShortcutConfig {
            capture: "Ctrl+Shift+X".to_string(),
            pin_clipboard: "Ctrl+Shift+V".to_string(),
        }
    }
}

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// API 基础地址
    pub api_base_url: String,
    /// 文本模型名称
    pub text_model: String,
    /// 视觉模型名称（可选，多模态模式需要）
    pub vision_model: Option<String>,
    /// 目标语言
    pub target_language: String,
    /// 默认翻译模式
    pub default_mode: TranslateMode,
    /// 快捷键配置
    pub shortcuts: ShortcutConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            api_base_url: String::new(),
            text_model: String::new(),
            vision_model: None,
            target_language: "zh-CN".to_string(),
            default_mode: TranslateMode::Ocr,
            shortcuts: ShortcutConfig::default(),
        }
    }
}

/// 配置管理器，负责配置文件的读写
pub struct ConfigManager {
    /// 配置文件所在目录
    config_dir: PathBuf,
    /// 配置文件完整路径
    config_path: PathBuf,
}

impl ConfigManager {
    /// 创建配置管理器实例
    pub fn new(app: &tauri::AppHandle) -> Result<Self, AppError> {
        let config_dir = app
            .path()
            .app_config_dir()
            .map_err(|e| AppError::ConfigError(format!("无法获取配置目录: {}", e)))?;

        let config_path = config_dir.join("config.toml");

        Ok(ConfigManager {
            config_dir,
            config_path,
        })
    }

    /// 加载配置，若配置文件不存在则创建默认配置
    pub fn load(&self) -> Result<AppConfig, AppError> {
        if !self.config_path.exists() {
            let default_config = AppConfig::default();
            self.save(&default_config)?;
            return Ok(default_config);
        }

        let content = fs::read_to_string(&self.config_path)?;
        let config: AppConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// 保存配置，使用原子写入避免写入过程中损坏
    pub fn save(&self, config: &AppConfig) -> Result<(), AppError> {
        fs::create_dir_all(&self.config_dir)?;

        let content = toml::to_string_pretty(config)?;

        // 原子写入：先写临时文件，再重命名
        let temp_path = self.config_path.with_extension("toml.tmp");
        fs::write(&temp_path, &content)?;
        fs::rename(&temp_path, &self.config_path)?;

        Ok(())
    }

    /// 获取配置文件目录路径
    #[allow(dead_code)]
    pub fn get_config_dir(&self) -> &PathBuf {
        &self.config_dir
    }
}
