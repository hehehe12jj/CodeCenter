use crate::error::AppError;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

/// 会话连接信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionConnection {
    pub session_id: String,
    pub project_path: String,
    pub can_send_input: bool,
}

/// 附加到会话（打开对话弹窗）
///
/// 建立与活跃会话的连接，获取进程信息以便后续交互。
/// 注意：直接向 Claude Code 进程发送输入需要 PTY 支持，当前仅返回连接信息。
#[tauri::command]
pub async fn attach_to_session(
    session_id: String,
    state: State<'_, AppState>,
) -> std::result::Result<SessionConnection, String> {
    let monitor = state.monitor.read().await;

    // 获取会话
    let session = monitor
        .get_session(&session_id)
        .await
        .ok_or_else(|| AppError::SessionNotFound(session_id.clone()).to_string())?;

    // 验证会话状态
    match session.status {
        crate::models::SessionStatus::Running => {
            // 运行中，可以连接
        }
        crate::models::SessionStatus::WaitingInput => {
            // 等待输入，可以连接
        }
        crate::models::SessionStatus::Initializing => {
            // 初始化中，可以连接
        }
        crate::models::SessionStatus::Completed => {
            return Err("会话已完成，无法附加".to_string());
        }
        crate::models::SessionStatus::Blocked => {
            return Err("会话被阻塞，无法附加".to_string());
        }
        crate::models::SessionStatus::Unknown => {
            // 未知状态，谨慎处理
        }
    }

    Ok(SessionConnection {
        session_id: session.id.clone(),
        project_path: session.project_path.clone(),
        can_send_input: false, // 暂时不支持直接发送输入
    })
}

/// 发送消息
///
/// 向会话发送消息。
/// 注意：由于 Claude Code 进程的 stdin 不直接可用，此功能暂时未实现。
#[tauri::command]
pub async fn send_message(
    _session_id: String,
    _content: String,
    _state: State<'_, AppState>,
) -> std::result::Result<(), String> {
    // 暂时不实现：直接向 Claude Code 进程发送 stdin 不可行
    // 如需此功能，建议：
    // 1. 通过项目文件进行交互（让 Claude Code 监听文件变化）
    // 2. 使用 VS Code MCP 协议
    // 3. 使用 pty 重新启动 Claude Code 进程
    Err("此功能暂时未实现。可以通过打开终端在项目目录中与 Claude Code 交互。".to_string())
}

/// 脱离会话（关闭对话弹窗）
///
/// 断开会话连接，清理相关状态。
#[tauri::command]
pub async fn detach_from_session(
    session_id: String,
    state: State<'_, AppState>,
) -> std::result::Result<(), String> {
    // 验证会话是否存在
    let monitor = state.monitor.read().await;
    let _session = monitor
        .get_session(&session_id)
        .await
        .ok_or_else(|| AppError::SessionNotFound(session_id.clone()).to_string())?;

    // 当前不需要清理额外状态，未来可以根据需要扩展
    tracing::info!("已脱离会话: {}", session_id);

    Ok(())
}
