//! 会话监控模块
//!
//! 提供会话发现、状态检测、文件监控等功能。
//!
//! # 模块结构
//!
//! - `discovery`: 会话发现，扫描锁文件和日志目录
//! - `status_detector`: 状态检测，解析日志推断会话状态
//! - `watcher`: 文件监控，使用 notify 监听日志变化
//!
//! # 使用示例
//!
//! ```rust
//! use crate::monitor::SessionMonitor;
//!
//! let monitor = SessionMonitor::new().await?;
//! monitor.start().await?;
//!
//! // 获取活跃会话
//! let sessions = monitor.get_active_sessions().await?;
//!
//! // 监听状态变化
//! while let Some(event) = monitor.next_event().await {
//!     println!("收到事件: {:?}", event);
//! }
//! ```

pub mod discovery;
pub mod status_detector;
pub mod watcher;

use crate::error::Result;
use crate::models::{Message, Session, SessionStatus};
use discovery::{DiscoveredSession, SessionDiscovery};
use status_detector::StatusDetector;
use std::collections::HashMap;
use std::mem;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};
use watcher::{WatchEvent, WatchManager};

/// 监控事件
#[derive(Debug, Clone)]
pub enum MonitorEvent {
    /// 发现新会话
    SessionDiscovered { session: Session },
    /// 会话状态变更
    StatusChanged {
        session_id: String,
        old_status: SessionStatus,
        new_status: SessionStatus,
    },
    /// 新消息
    NewMessage {
        session_id: String,
        message: Message,
    },
    /// 会话结束
    SessionEnded { session_id: String },
    /// 错误
    Error { message: String },
}

/// 会话监控器
///
/// 整合发现、状态检测、文件监控，提供统一的监控接口
pub struct SessionMonitor {
    /// 会话发现器
    discovery: SessionDiscovery,
    /// 文件监控管理器
    watch_manager: WatchManager,
    /// 事件接收器
    event_receiver: mpsc::Receiver<MonitorEvent>,
    /// 内部事件发送器
    event_sender: mpsc::Sender<MonitorEvent>,
    /// 会话缓存
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    /// 状态缓存
    status_cache: Arc<RwLock<HashMap<String, SessionStatus>>>,
    /// 是否正在运行
    running: Arc<RwLock<bool>>,
}

impl SessionMonitor {
    /// 创建新的会话监控器
    pub async fn new() -> Result<Self> {
        let discovery = SessionDiscovery::new()?;
        let watch_manager = WatchManager::new().await?;

        let (event_sender, event_receiver) = mpsc::channel(100);

        Ok(Self {
            discovery,
            watch_manager,
            event_receiver,
            event_sender,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            status_cache: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
        })
    }

    /// 启动监控
    ///
    /// 1. 初始化文件监控
    /// 2. 发现现有会话
    /// 3. 启动事件处理循环
    pub async fn start(&mut self) -> Result<()> {
        info!("启动会话监控...");

        // 设置运行标志
        {
            let mut running = self.running.write().await;
            *running = true;
        }

        // 初始化文件监控
        self.watch_manager.initialize().await?;

        // 发现现有会话
        self.discover_existing_sessions().await?;

        // 启动事件处理循环
        self.spawn_event_handler();

        info!("会话监控已启动");
        Ok(())
    }

    /// 停止监控
    pub async fn stop(&mut self) {
        info!("停止会话监控...");

        let mut running = self.running.write().await;
        *running = false;

        // 清理缓存
        self.sessions.write().await.clear();
        self.status_cache.write().await.clear();
    }

    /// 获取所有活跃会话
    pub async fn get_active_sessions(&self) -> Result<Vec<Session>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.values().cloned().collect())
    }

    /// 获取特定会话
    pub async fn get_session(&self, session_id: &str) -> Option<Session> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).cloned()
    }

    /// 获取会话状态
    pub async fn get_session_status(&self, session_id: &str) -> Option<SessionStatus> {
        // 首先检查缓存
        {
            let cache = self.status_cache.read().await;
            if let Some(status) = cache.get(session_id) {
                return Some(*status);
            }
        }

        // 如果没有缓存，尝试检测
        if let Some(_session) = self.get_session(session_id).await {
            if let Some(log_path) = self.get_session_log_path(session_id).await {
                match StatusDetector::detect(&log_path) {
                    Ok(status) => {
                        // 更新缓存
                        let mut cache = self.status_cache.write().await;
                        cache.insert(session_id.to_string(), status);
                        return Some(status);
                    }
                    Err(e) => {
                        warn!("检测会话状态失败 {}: {}", session_id, e);
                    }
                }
            }
        }

        None
    }

    /// 手动刷新指定会话状态
    pub async fn refresh_session(&self, session_id: &str) -> Result<()> {
        let log_path = self
            .get_session_log_path(session_id)
            .await
            .ok_or_else(|| crate::error::AppError::StorageError("未找到日志文件".to_string()))?;

        // 检测新状态
        let new_status = StatusDetector::detect(&log_path)?;

        // 获取旧状态
        let old_status = {
            let cache = self.status_cache.read().await;
            *cache.get(session_id).unwrap_or(&SessionStatus::Unknown)
        };

        // 如果状态变化，更新并发送事件
        if new_status != old_status {
            debug!(
                "会话 {} 状态变化: {:?} -> {:?}",
                session_id, old_status, new_status
            );

            {
                let mut cache = self.status_cache.write().await;
                cache.insert(session_id.to_string(), new_status);
            }

            {
                let mut sessions = self.sessions.write().await;
                if let Some(session) = sessions.get_mut(session_id) {
                    session.status = new_status;
                }
            }

            // 发送状态变更事件
            let _ = self
                .event_sender
                .send(MonitorEvent::StatusChanged {
                    session_id: session_id.to_string(),
                    old_status,
                    new_status,
                })
                .await;
        }

        Ok(())
    }

    /// 手动刷新所有会话
    pub async fn refresh_all(&self) -> Result<()> {
        let session_ids: Vec<String> = {
            let sessions = self.sessions.read().await;
            sessions.keys().cloned().collect()
        };

        for session_id in session_ids {
            if let Err(e) = self.refresh_session(&session_id).await {
                warn!("刷新会话状态失败 {}: {}", session_id, e);
            }
        }

        Ok(())
    }

    /// 获取下一个事件
    pub async fn next_event(&mut self) -> Option<MonitorEvent> {
        self.event_receiver.recv().await
    }

    /// 获取事件接收器的可变引用
    pub fn event_stream(&mut self) -> &mut mpsc::Receiver<MonitorEvent> {
        &mut self.event_receiver
    }

    /// 获取事件接收器的所有权
    /// 用于在需要 move 接收器的场景
    /// 创建一个新的接收器来替换原接收器
    pub fn take_event_stream(&mut self) -> mpsc::Receiver<MonitorEvent> {
        // 创建一个新的 sender 和 receiver 对
        let (new_sender, new_receiver) = mpsc::channel(100);

        // 用新的 sender 替换原有的 sender
        let old_sender = mem::replace(&mut self.event_sender, new_sender);

        // 用新的 receiver 替换原有的 receiver
        let old_receiver = mem::replace(&mut self.event_receiver, new_receiver);

        old_receiver
    }

    /// 发现现有会话
    async fn discover_existing_sessions(&mut self) -> Result<()> {
        info!("发现现有会话...");

        let discovered = self.discovery.discover_sessions().await?;

        for disc in discovered {
            debug!(
                "发现会话: {} (pid={})",
                disc.project_name, disc.pid
            );

            // 创建会话对象
            let session = self.convert_to_session(&disc).await?;
            let session_id = session.id.clone();

            // 添加到缓存
            {
                let mut sessions = self.sessions.write().await;
                sessions.insert(session_id.clone(), session.clone());
            }

            // 开始监控日志文件
            if let Some(ref log_path) = disc.log_path {
                if let Err(e) = self.watch_manager.watch_session(log_path).await {
                    warn!("监控会话日志失败 {}: {}", session_id, e);
                }
            }

            // 发送发现事件
            let _ = self
                .event_sender
                .send(MonitorEvent::SessionDiscovered { session })
                .await;
        }

        let count = self.sessions.read().await.len();
        info!("已发现 {} 个活跃会话", count);

        Ok(())
    }

    /// 启动事件处理循环
    fn spawn_event_handler(&mut self) {
        // 从 self 中提取需要在异步任务中使用的数据
        let event_sender = self.event_sender.clone();
        let sessions = self.sessions.clone();
        let status_cache = self.status_cache.clone();
        let running = self.running.clone();

        // 获取事件流接收器
        let watch_rx = self.watch_manager.take_event_stream();
        let Some(mut watch_rx) = watch_rx else {
            error!("无法获取事件流接收器");
            return;
        };

        tokio::spawn(async move {
            info!("事件处理循环已启动");

            loop {
                // 检查是否仍在运行
                if !*running.read().await {
                    break;
                }

                match watch_rx.recv().await {
                    Some(WatchEvent::SessionDiscovered { session: disc }) => {
                        // 检查是否已存在
                        let exists = {
                            let sessions = sessions.read().await;
                            sessions.values().any(|s| {
                                s.project_path == disc.project_path.to_string_lossy()
                            })
                        };

                        if !exists {
                            if let Ok(session) =
                                Self::convert_discovered_to_session(&disc).await
                            {
                                let session_id = session.id.clone();

                                {
                                    let mut sessions = sessions.write().await;
                                    sessions.insert(session_id.clone(), session.clone());
                                }

                                let _ = event_sender
                                    .send(MonitorEvent::SessionDiscovered { session })
                                    .await;
                            }
                        }
                    }
                    Some(WatchEvent::LogChanged { session_id, path }) => {
                        // 检测状态变化
                        if let Ok(new_status) = StatusDetector::detect(&path) {
                            let old_status = {
                                let cache = status_cache.read().await;
                                *cache.get(&session_id).unwrap_or(&SessionStatus::Unknown)
                            };

                            if new_status != old_status {
                                {
                                    let mut cache = status_cache.write().await;
                                    cache.insert(session_id.clone(), new_status);
                                }

                                {
                                    let mut sessions = sessions.write().await;
                                    if let Some(session) = sessions.get_mut(&session_id) {
                                        session.status = new_status;
                                    }
                                }

                                let _ = event_sender
                                    .send(MonitorEvent::StatusChanged {
                                        session_id,
                                        old_status,
                                        new_status,
                                    })
                                    .await;
                            }
                        }
                    }
                    Some(WatchEvent::SessionEnded { session_id }) => {
                        {
                            let mut sessions = sessions.write().await;
                            sessions.remove(&session_id);
                        }

                        {
                            let mut cache = status_cache.write().await;
                            cache.remove(&session_id);
                        }

                        let _ = event_sender
                            .send(MonitorEvent::SessionEnded { session_id })
                            .await;
                    }
                    Some(WatchEvent::Error { message }) => {
                        error!("监控错误: {}", message);
                        let _ = event_sender
                            .send(MonitorEvent::Error { message })
                            .await;
                    }
                    None => {
                        // 通道关闭
                        break;
                    }
                }
            }

            info!("事件处理循环已停止");
        });
    }

    /// 转换发现的会话为 Session 对象
    async fn convert_to_session(&self, disc: &DiscoveredSession) -> Result<Session> {
        Self::convert_discovered_to_session(disc).await
    }

    /// 静态方法：转换 DiscoveredSession 为 Session
    async fn convert_discovered_to_session(disc: &DiscoveredSession) -> Result<Session> {
        let session_id = format!("sess_{}_{}", disc.pid, chrono::Utc::now().timestamp_millis());

        // 检测初始状态
        let status = if let Some(ref log_path) = disc.log_path {
            StatusDetector::detect(log_path).unwrap_or(SessionStatus::Unknown)
        } else {
            SessionStatus::Unknown
        };

        // 提取第一条用户消息用于标题和摘要
        let first_user_message = if let Some(ref log_path) = disc.log_path {
            StatusDetector::extract_first_user_message(log_path).ok().flatten()
        } else {
            None
        };

        // 标题使用前30字符，摘要使用前50字符
        let title_prompt = first_user_message.as_ref().map(|m| {
            let content = m.content.chars().take(30).collect::<String>();
            if m.content.len() > 30 {
                format!("{}...", content)
            } else {
                content
            }
        }).unwrap_or_default();

        let summary_text = first_user_message.as_ref().map(|m| {
            let content = m.content.chars().take(50).collect::<String>();
            if m.content.len() > 50 {
                format!("{}...", content)
            } else {
                content
            }
        }).unwrap_or_default();

        let now = chrono::Utc::now();
        let created_at = disc.start_time.unwrap_or(now);

        Ok(Session {
            id: session_id,
            title: format!("{} | {}", disc.project_name, title_prompt),
            project_name: disc.project_name.clone(),
            project_path: disc.project_path.to_string_lossy().to_string(),
            agent_type: "claude".to_string(),
            status,
            created_at,
            last_active_at: now,
            summary: if summary_text.is_empty() { None } else { Some(summary_text) },
            is_archived: false,
        })
    }

    /// 获取会话的日志路径
    async fn get_session_log_path(&self, session_id: &str) -> Option<PathBuf> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).and_then(|session| {
            // 从项目路径构造日志路径
            let home = dirs::home_dir()?;
            let project_path = &session.project_path;
            let encoded = project_path.replace('/', "--").replace('\\', "--");
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
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_monitor_creation() {
        // 这个测试需要 Claude Code 环境，仅在本地运行
        if let Ok(monitor) = SessionMonitor::new().await {
            // 成功创建
            assert!(!(*monitor.running.read().await));
        }
    }
}
