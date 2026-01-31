use crate::models::{Message, Session, SessionDetail, SessionStatus};
use crate::monitor::status_detector::StatusDetector;
use crate::state::AppState;
use std::path::PathBuf;
use tauri::State;

/// 获取所有活跃会话
#[tauri::command]
pub async fn get_all_sessions(state: State<'_, AppState>) -> std::result::Result<Vec<Session>, String> {
    tracing::info!("[get_all_sessions] 命令被调用");

    // 使用 write lock 以支持即时刷新
    tracing::info!("[get_all_sessions] 尝试获取 monitor 锁...");
    let mut monitor = state.monitor.write().await;
    tracing::info!("[get_all_sessions] 获取到 monitor 锁, 调用 instant_refresh...");

    // 调用 instant_refresh 区分新增/存量会话
    if let Err(e) = monitor.instant_refresh().await {
        tracing::error!("[get_all_sessions] instant_refresh 错误: {}", e);
        return Err(e.to_string());
    }

    let sessions = monitor.get_active_sessions().await.map_err(|e| {
        tracing::error!("[get_all_sessions] get_active_sessions 错误: {}", e);
        e.to_string()
    })?;

    tracing::info!("[get_all_sessions] 获取到 {} 个会话", sessions.len());

    // 只过滤掉 Unknown 状态的会话（Initializing 也展示，显示为运行中）
    let filtered_sessions: Vec<Session> = sessions
        .into_iter()
        .filter(|s| s.status != SessionStatus::Unknown)
        .collect();
    tracing::info!("[get_all_sessions] 过滤后剩余 {} 个会话", filtered_sessions.len());

    // 如果没有会话，尝试从 storage 加载
    if filtered_sessions.is_empty() {
        let storage = state.storage();
        let storage_sessions = storage.get_active_sessions().await.map_err(|e| e.to_string())?;
        tracing::info!("[get_all_sessions] 从 storage 获取到 {} 个会话", storage_sessions.len());
        Ok(storage_sessions)
    } else {
        // 打印第一个会话的详细信息用于调试
        if let Some(first) = filtered_sessions.first() {
            tracing::info!("[get_all_sessions] 第一个会话: id={}, title={}, status={:?}, created_at={:?}",
                first.id, first.title, first.status, first.created_at);
        }
        Ok(filtered_sessions)
    }
}

/// 获取会话详情
#[tauri::command]
pub async fn get_session_detail(
    id: String,
    message_limit: Option<usize>,
    state: State<'_, AppState>,
) -> std::result::Result<SessionDetail, String> {
    // 默认消息限制
    let limit = message_limit.unwrap_or(20);

    // 首先尝试从 monitor 获取会话
    let monitor = state.monitor.read().await;

    if let Some(session) = monitor.get_session(&id).await {
        // 获取日志文件路径
        let log_path = find_session_log_path(&session.project_path).await;

        // 提取消息
        let messages: Vec<Message> = if let Some(ref path) = log_path {
            StatusDetector::extract_recent_messages(path, limit)
                .map_err(|e: crate::error::AppError| e.to_string())?
                .into_iter()
                .rev() // 按时间正序排列
                .collect()
        } else {
            Vec::new()
        };

        // 获取进程信息
        let process_info = extract_process_info(&session);

        // 计算统计信息
        let stats = calculate_stats(&session, &messages);

        return Ok(SessionDetail {
            session,
            messages,
            process_info,
            stats,
        });
    }

    // 如果 monitor 中没有，尝试从 storage 加载
    drop(monitor);
    let storage = state.storage();
    storage.load_session_detail(&id).await.map_err(|e| e.to_string())
}

/// 查找会话的日志文件路径
async fn find_session_log_path(project_path: &str) -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let project_path = PathBuf::from(project_path);
    let encoded = project_path.to_string_lossy().replace(['/', '\\'], "--");
    let log_dir = home.join(".claude").join("projects").join(encoded);

    // 查找最新的 jsonl 文件
    std::fs::read_dir(&log_dir)
        .ok()?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .map(|ext| ext == "jsonl")
                .unwrap_or(false)
        })
        .max_by_key(|entry| {
            entry
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        })
        .map(|entry| entry.path())
}

/// 从会话提取进程信息
fn extract_process_info(session: &Session) -> Option<crate::models::ProcessInfo> {
    // 从 session ID 中解析 PID
    // session ID 格式: sess_{pid}_{timestamp}
    if let Some(pid_str) = session.id.split('_').nth(1) {
        if let Ok(pid) = pid_str.parse::<u32>() {
            return Some(crate::models::ProcessInfo {
                pid,
                start_time: session.created_at,
                command_line: format!("claude --project {}", session.project_path),
            });
        }
    }
    None
}

/// 计算会话统计信息
fn calculate_stats(
    session: &Session,
    messages: &[Message],
) -> crate::models::SessionStats {
    let message_count = messages.len() as u32;

    // 估算 token 数量（基于字符数的粗略估算）
    let total_tokens: Option<u32> = Some(
        messages
            .iter()
            .map(|m| m.content.len() / 4) // 平均每个 token 约 4 个字符
            .sum::<usize>() as u32,
    );

    // 计算会话持续时间
    let duration_secs = session
        .last_active_at
        .signed_duration_since(session.created_at)
        .num_seconds() as u64;

    crate::models::SessionStats {
        message_count,
        total_tokens,
        duration_secs,
    }
}

/// 标记会话完成
#[tauri::command]
pub async fn mark_session_completed(id: String, state: State<'_, AppState>) -> std::result::Result<(), String> {
    let storage = state.storage();

    let mut session = storage.get_session(&id).await.map_err(|e| e.to_string())?;
    session.status = SessionStatus::Completed;
    session.last_active_at = chrono::Utc::now();

    storage.update_session(&session).await.map_err(|e| e.to_string())
}

/// 归档会话
#[tauri::command]
pub async fn archive_session(id: String, state: State<'_, AppState>) -> std::result::Result<(), String> {
    let storage = state.storage();

    let mut session = storage.get_session(&id).await.map_err(|e| e.to_string())?;
    session.is_archived = true;
    session.last_active_at = chrono::Utc::now();

    storage.update_session(&session).await.map_err(|e| e.to_string())
}

/// 取消归档
#[tauri::command]
pub async fn unarchive_session(id: String, state: State<'_, AppState>) -> std::result::Result<(), String> {
    let storage = state.storage();

    let mut session = storage.get_session(&id).await.map_err(|e| e.to_string())?;
    session.is_archived = false;
    session.last_active_at = chrono::Utc::now();

    storage.update_session(&session).await.map_err(|e| e.to_string())
}
