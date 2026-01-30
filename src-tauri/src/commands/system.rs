use crate::state::AppState;
use std::path::PathBuf;
use std::process::Command;
use tauri::State;

/// 打开系统终端
///
/// 使用 AppleScript 打开 macOS Terminal.app 并切换到指定项目目录。
#[tauri::command]
pub async fn open_terminal(
    project_path: String,
    _state: State<'_, AppState>,
) -> std::result::Result<(), String> {
    let path = PathBuf::from(&project_path);

    // 验证路径是否存在
    if !path.exists() {
        return Err(format!("项目路径不存在: {}", project_path));
    }

    if !path.is_dir() {
        return Err(format!("项目路径不是目录: {}", project_path));
    }

    // 使用 AppleScript 打开 Terminal 并执行命令
    // 先 cd 到项目目录，然后显示提示符
    let path_str = project_path.replace('"', "\\\"");
    let script = format!(
        r#"osascript -e 'tell app "Terminal" to do script "cd \"{}\" && clear'" 2>&1"#,
        path_str
    );

    let output = Command::new("sh")
        .args(["-c", &script])
        .output()
        .map_err(|e| format!("执行命令失败: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("打开终端失败: {}", stderr));
    }

    tracing::info!("已打开终端并切换到: {}", project_path);
    Ok(())
}

/// 手动刷新状态
#[tauri::command]
pub async fn refresh_status(state: State<'_, AppState>) -> std::result::Result<(), String> {
    // TODO: 触发状态刷新
    // 刷新所有会话状态
    let monitor = state.monitor.read().await;
    monitor.refresh_all().await.map_err(|e| e.to_string())
}

/// 获取应用配置
#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> std::result::Result<String, String> {
    let config = state.config.read().await;
    serde_json::to_string(&*config).map_err(|e| e.to_string())
}

/// 更新配置
#[tauri::command]
pub async fn update_config(
    config_json: String,
    state: State<'_, AppState>,
) -> std::result::Result<(), String> {
    let new_config: crate::models::AppConfig = serde_json::from_str(&config_json)
        .map_err(|e| format!("配置解析失败: {}", e))?;

    // 更新内存中的配置
    {
        let mut config = state.config.write().await;
        *config = new_config;
    }

    // 保存到文件
    state.save_config().await.map_err(|e| e.to_string())
}
