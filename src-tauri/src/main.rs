// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Emitter, Manager};

mod commands;
mod error;
mod models;
mod monitor;
mod state;
mod storage;

use state::AppState;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_all_sessions,
            commands::get_session_detail,
            commands::mark_session_completed,
            commands::archive_session,
            commands::unarchive_session,
            commands::send_message,
            commands::open_terminal,
            commands::refresh_status,
            commands::get_config,
            commands::update_config,
        ])
        .setup(|app| {
            tracing::info!("CodeAgent Dashboard starting...");

            // 同步初始化应用状态（确保在应用就绪前完成）
            let state = tauri::async_runtime::block_on(async {
                AppState::init().await
            });

            match state {
                Ok(state) => {
                    app.manage(state.clone());
                    tracing::info!("AppState initialized successfully");

                    // 启动会话监控器
                    let window = app.get_webview_window("main").unwrap_or_else(|| {
                        app.get_webview_window("main").expect("Failed to get main window")
                    });

                    tauri::async_runtime::spawn(async move {
                        // 先启动 monitor（包含会话发现）
                        let monitor_started = {
                            let mut monitor = state.monitor.write().await;
                            tracing::info!("启动会话监控...");
                            match monitor.start().await {
                                Ok(_) => {
                                    tracing::info!("Session monitor started successfully");
                                    true
                                }
                                Err(e) => {
                                    tracing::error!("Failed to start monitor: {}", e);
                                    false
                                }
                            }
                        };

                        if monitor_started {
                            // 然后获取事件接收器（不阻塞其他操作）
                            let mut rx = {
                                let mut monitor_guard = state.monitor.write().await;
                                tracing::info!("获取事件接收器...");
                                let receiver = monitor_guard.take_event_stream();
                                tracing::info!("事件接收器已获取");
                                receiver
                            };

                            // 启动事件转发到前端
                            forward_events_to_frontend(&mut rx, window).await;
                        } else {
                            tracing::error!("无法启动会话监控");
                        }
                    });
                }
                Err(e) => {
                    tracing::error!("Failed to initialize AppState: {}", e);
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("AppState initialization failed: {}", e)
                    )));
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 事件转发到前端
///
/// 监听监控事件并将状态变更推送到前端
async fn forward_events_to_frontend(
    rx: &mut tokio::sync::mpsc::Receiver<monitor::MonitorEvent>,
    window: tauri::WebviewWindow,
) {
    tracing::info!("事件转发循环已启动");

    while let Some(event) = rx.recv().await {
        match event {
            monitor::MonitorEvent::StatusChanged {
                session_id,
                old_status,
                new_status,
            } => {
                tracing::debug!(
                    "转发状态变更事件: {} {:?} -> {:?}",
                    session_id,
                    old_status,
                    new_status
                );

                // 转换为前端事件格式
                if let Err(e) = window.emit("session:status-changed", serde_json::json!({
                    "sessionId": session_id,
                    "oldStatus": serialize_status(&old_status),
                    "newStatus": serialize_status(&new_status)
                })) {
                    tracing::error!("发送状态变更事件失败: {}", e);
                }
            }
            monitor::MonitorEvent::SessionDiscovered { session } => {
                tracing::debug!("转发会话发现事件: {}", session.id);

                if let Err(e) = window.emit("session:discovered", serde_json::json!({
                    "session": session_to_dto(&session)
                })) {
                    tracing::error!("发送会话发现事件失败: {}", e);
                }
            }
            monitor::MonitorEvent::SessionEnded { session_id } => {
                tracing::debug!("转发会话结束事件: {}", session_id);

                if let Err(e) = window.emit("session:ended", serde_json::json!({
                    "sessionId": session_id
                })) {
                    tracing::error!("发送会话结束事件失败: {}", e);
                }
            }
            monitor::MonitorEvent::NewMessage { session_id, message } => {
                tracing::debug!("转发新消息事件: {}", session_id);

                if let Err(e) = window.emit("session:new-message", serde_json::json!({
                    "sessionId": session_id,
                    "message": message_to_dto(&message)
                })) {
                    tracing::error!("发送新消息事件失败: {}", e);
                }
            }
            monitor::MonitorEvent::Error { message } => {
                tracing::error!("监控错误: {}", message);

                if let Err(e) = window.emit("monitor:error", serde_json::json!({
                    "message": message
                })) {
                    tracing::error!("发送错误事件失败: {}", e);
                }
            }
        }
    }

    tracing::info!("事件转发循环已停止");
}

/// 序列化会话状态为字符串
fn serialize_status(status: &models::SessionStatus) -> String {
    match status {
        models::SessionStatus::Running => "running",
        models::SessionStatus::WaitingInput => "waiting_input",
        models::SessionStatus::Completed => "completed",
        models::SessionStatus::Blocked => "blocked",
        models::SessionStatus::Unknown => "unknown",
    }
    .to_string()
}

/// 将会话转换为前端 DTO
fn session_to_dto(session: &models::Session) -> serde_json::Value {
    serde_json::json!({
        "id": session.id,
        "title": session.title,
        "projectName": session.project_name,
        "projectPath": session.project_path,
        "agentType": session.agent_type,
        "status": serialize_status(&session.status),
        "createdAt": session.created_at.timestamp_millis(),
        "lastActiveAt": session.last_active_at.timestamp_millis(),
        "summary": session.summary,
        "isArchived": session.is_archived
    })
}

/// 将消息转换为前端 DTO
fn message_to_dto(message: &models::Message) -> serde_json::Value {
    serde_json::json!({
        "role": match message.role {
            models::MessageRole::User => "user",
            models::MessageRole::Assistant => "assistant",
        },
        "content": message.content,
        "timestamp": message.timestamp.timestamp_millis()
    })
}
