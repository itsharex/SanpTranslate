use serde::Serialize;

#[allow(clippy::enum_variant_names)]
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
pub enum AppError {
    CaptureError(String),
    OcrError(String),
    TranslateError(String),
    ConfigError(String),
    DatabaseError(String),
    ClipboardError(String),
    NetworkError(String),
    TrayError(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::CaptureError(msg) => write!(f, "截图错误: {}", msg),
            AppError::OcrError(msg) => write!(f, "OCR错误: {}", msg),
            AppError::TranslateError(msg) => write!(f, "翻译错误: {}", msg),
            AppError::ConfigError(msg) => write!(f, "配置错误: {}", msg),
            AppError::DatabaseError(msg) => write!(f, "数据库错误: {}", msg),
            AppError::ClipboardError(msg) => write!(f, "剪贴板错误: {}", msg),
            AppError::NetworkError(msg) => write!(f, "网络错误: {}", msg),
            AppError::TrayError(msg) => write!(f, "托盘错误: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::ConfigError(e.to_string())
    }
}

impl From<toml::de::Error> for AppError {
    fn from(e: toml::de::Error) -> Self {
        AppError::ConfigError(e.to_string())
    }
}

impl From<toml::ser::Error> for AppError {
    fn from(e: toml::ser::Error) -> Self {
        AppError::ConfigError(e.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        if e.is_timeout() {
            AppError::NetworkError("请求超时".to_string())
        } else if e.is_connect() {
            AppError::NetworkError("无法连接到服务器".to_string())
        } else {
            AppError::NetworkError(e.to_string())
        }
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        AppError::DatabaseError(e.to_string())
    }
}

impl From<tauri::Error> for AppError {
    fn from(e: tauri::Error) -> Self {
        AppError::TrayError(e.to_string())
    }
}
