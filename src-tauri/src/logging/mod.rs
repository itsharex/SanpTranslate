// 日志管理模块
// 职责：
// 1. 生成带时间戳的日志文件名（每次启动独立文件）
// 2. 过期清理 —— 删除超过指定天数的旧日志文件
//
// 文件命名格式：snaptranslate_YYYY-MM-DD_HH-MM-SS.log
// 每次应用启动自动创建新文件，无需会话轮转。

use std::fs;
use std::path::{Path, PathBuf};

use chrono::Local;

/// 日志文件名前缀
const LOG_PREFIX: &str = "snaptranslate_";

/// 日志保留天数
const MAX_LOG_DAYS: i64 = 30;

/// 应用 identifier，与 tauri.conf.json 保持一致
/// 仅 prod 模式下使用（dev 模式日志写入项目根目录/log/）
#[cfg(not(debug_assertions))]
const APP_IDENTIFIER: &str = "snaptranslate";

/// 在 tauri 初始化前执行日志目录初始化
///
/// 返回当前会话的日志文件名（不含 .log 后缀），
/// 供 tauri-plugin-log 的 LogDir.file_name 配置使用。
///
/// 必须在 `tauri::Builder::default()` 之前调用。
///
/// 日志目录策略：
/// - dev 模式：项目根目录下的 `log/` 文件夹，便于开发时直接查看
/// - prod 模式：OS 标准日志目录（Windows: `%LOCALAPPDATA%\snaptranslate\logs` 等）
pub fn init_before_tauri() -> String {
    let file_name = format!(
        "{}{}",
        LOG_PREFIX,
        Local::now().format("%Y-%m-%d_%H-%M-%S")
    );

    if let Some(log_dir) = get_log_dir() {
        init_log_dir(&log_dir);
    } else {
        eprintln!("[LOGGING] 无法确定日志目录路径，跳过日志初始化");
    }

    file_name
}

/// 获取日志目录路径
///
/// dev 模式使用项目根目录下的 `log/`（通过 `CARGO_MANIFEST_DIR` 推导），
/// prod 模式使用 OS 标准日志目录。
pub fn get_log_dir() -> Option<PathBuf> {
    #[cfg(debug_assertions)]
    {
        // dev 模式：项目根目录/log/
        // CARGO_MANIFEST_DIR 指向 src-tauri/，取其父目录即为项目根目录
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        Some(manifest_dir.parent()?.join("log"))
    }

    #[cfg(not(debug_assertions))]
    {
        get_prod_log_dir()
    }
}

/// prod 模式下获取 OS 标准日志目录路径
#[cfg(not(debug_assertions))]
fn get_prod_log_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        let local_app_data = std::env::var("LOCALAPPDATA").ok()?;
        Some(PathBuf::from(local_app_data).join(APP_IDENTIFIER).join("logs"))
    }

    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").ok()?;
        Some(PathBuf::from(home).join("Library").join("Logs").join(APP_IDENTIFIER))
    }

    #[cfg(target_os = "linux")]
    {
        let home = std::env::var("HOME").ok()?;
        Some(PathBuf::from(home).join(".local").join("share").join(APP_IDENTIFIER).join("logs"))
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        None
    }
}

/// 初始化日志目录：确保目录存在 + 过期清理
fn init_log_dir(log_dir: &Path) {
    if let Err(e) = fs::create_dir_all(log_dir) {
        eprintln!("[LOGGING] 创建日志目录失败: {}", e);
        return;
    }

    clean_expired_logs(log_dir);
}

/// 过期清理：删除超过 MAX_LOG_DAYS 天的 snaptranslate_*.log 文件
///
/// 按文件名中的日期判断，而非文件修改时间。
/// 无法解析日期的文件跳过（保守策略，避免误删）。
fn clean_expired_logs(log_dir: &Path) {
    let entries = match fs::read_dir(log_dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    let now = Local::now().naive_local();
    let max_age = chrono::Duration::days(MAX_LOG_DAYS);

    for entry in entries.flatten() {
        let path = entry.path();
        let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        // 仅处理 snaptranslate_*.log 格式的文件
        if !file_name.starts_with(LOG_PREFIX) || !file_name.ends_with(".log") {
            continue;
        }

        // 从文件名解析日期：snaptranslate_YYYY-MM-DD_HH-MM-SS.log
        let Some(date_str) = file_name
            .strip_prefix(LOG_PREFIX)
            .and_then(|s| s.strip_suffix(".log"))
        else {
            continue;
        };

        let Ok(log_time) = chrono::NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d_%H-%M-%S") else {
            continue;
        };

        if now - log_time > max_age {
            if let Err(e) = fs::remove_file(&path) {
                eprintln!("[LOGGING] 删除过期日志失败 {:?}: {}", path, e);
            }
        }
    }
}
