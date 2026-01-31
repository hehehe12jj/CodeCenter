// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

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
            tracing::info!("CodeCenter starting...");

            // 同步初始化应用状态（确保在应用就绪前完成）
            let state = tauri::async_runtime::block_on(async {
                AppState::init().await
            });

            match state {
                Ok(state) => {
                    app.manage(state.clone());
                    tracing::info!("AppState initialized successfully");

                    // 启动会话监控器（用于 get_all_sessions 查询）
                    tauri::async_runtime::spawn(async move {
                        // 启动 monitor（用于 get_all_sessions 查询）
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

                        if !monitor_started {
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

