// 自动更新模块：处理应用启动时的静默更新检查和安装

use tauri_plugin_updater::UpdaterExt;

/// 检查并安装更新（静默模式，用于自动更新）
/// 仅在有新版本时下载安装，安装完成后自动重启应用
/// 仅在 release 模式下被调用，dev 模式下会产生 dead_code 警告，故允许
#[allow(dead_code)]
pub async fn check_and_install_update(app: &tauri::AppHandle) -> Result<(), String> {
    log::info!("[UPDATE] 开始检查更新...");

    let updater = app.updater().map_err(|e| format!("获取更新器失败: {}", e))?;

    let update = match updater.check().await {
        Ok(Some(update)) => {
            let current_version = app.config().version.as_deref().unwrap_or("unknown");
            log::info!(
                "[UPDATE] 发现新版本: {} (当前: {})",
                update.version,
                current_version
            );
            update
        }
        Ok(None) => {
            log::info!("[UPDATE] 当前已是最新版本");
            return Ok(());
        }
        Err(e) => {
            log::warn!("[UPDATE] 检查更新失败: {}", e);
            return Err(format!("检查更新失败: {}", e));
        }
    };

    // 下载并安装更新
    log::info!("[UPDATE] 开始下载更新...");
    update
        .download_and_install(
            |chunk_length, content_length| {
                log::debug!(
                    "[UPDATE] 下载进度: {} / {:?} 字节",
                    chunk_length,
                    content_length
                );
            },
            || {
                log::info!("[UPDATE] 下载完成，开始安装...");
            },
        )
        .await
        .map_err(|e| format!("下载安装更新失败: {}", e))?;

    log::info!("[UPDATE] 更新安装完成，即将重启应用...");

    // 安装完成后重启应用（restart 是 noreturn 函数）
    app.restart();
}
