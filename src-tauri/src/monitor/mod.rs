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

use crate::error::{AppError, Result};
use crate::models::{Message, Session, SessionStatus};
use discovery::{DiscoveredSession, SessionDiscovery};
use status_detector::StatusDetector;
use std::collections::HashMap;
use std::mem;
use std::path::{Path, PathBuf};
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

/// 进程存在性检测结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessExistence {
    /// 进程确定存在（持锁中）
    Alive,
    /// 找不到锁文件（可能没创建/已退出）
    NotFound,
    /// 有锁但可加锁（进程已死）
    Dead,
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

    /// 刷新并获取所有活跃会话
    ///
    /// 重新扫描发现新会话，移除已结束的会话，返回最新的会话列表
    pub async fn refresh_and_get_sessions(&mut self) -> Result<Vec<Session>> {
        info!("刷新并获取所有活跃会话...");

        // 重新发现会话
        let discovered = self.discovery.discover_sessions().await?;

        let mut sessions = self.sessions.write().await;
        // 直接跟踪活跃的 PID
        let mut active_pids: std::collections::HashSet<u32> = std::collections::HashSet::new();
        // 跟踪活跃的项目路径（用于 pid=0 的会话）
        let mut active_project_paths: std::collections::HashSet<String> = std::collections::HashSet::new();

        // 基于 project_path 去重
        let mut processed_paths: std::collections::HashSet<String> = std::collections::HashSet::new();

        for disc in &discovered {
            let project_path_key = disc.project_path.to_string_lossy().to_string();

            // 去重：同一项目只处理一次
            if !processed_paths.insert(project_path_key.clone()) {
                debug!("跳过重复项目: {}", disc.project_name);
                continue;
            }

            // 跟踪活跃的项目路径（所有发现的会话都跟踪）
            active_project_paths.insert(project_path_key.clone());

            if disc.pid == 0 {
                // pid=0 的会话：从日志发现的，仍视为有效（日志文件在30分钟内）
                // 补充日志路径信息并添加到活跃会话列表
                debug!("发现 pid=0 的会话: {}", disc.project_name);
                // 生成 session ID 并添加到 sessions
                let session_id = Self::generate_session_id(&disc);
                match Self::convert_discovered_to_session(&disc).await {
                    Ok(session) => {
                        sessions.insert(session_id, session);
                        debug!("添加 pid=0 会话: {}", disc.project_name);
                    }
                    Err(e) => {
                        warn!("转换 pid=0 会话失败: {}", e);
                    }
                }
                continue;
            }

            // 检查进程是否存在
            if !self.discovery.process_exists(disc.pid) {
                debug!("进程 {} 不存在，跳过", disc.pid);
                continue;
            }

            active_pids.insert(disc.pid);

            // 生成固定的 session ID
            let session_id = Self::generate_session_id(&disc);

            // 转换并添加/更新会话
            match Self::convert_discovered_to_session(&disc).await {
                Ok(session) => {
                    sessions.insert(session_id, session);
                    debug!("发现会话: {} (pid={})", disc.project_name, disc.pid);
                }
                Err(e) => {
                    warn!("转换会话失败: {}", e);
                }
            }
        }

        // 收集需要移除的会话
        let mut to_remove: Vec<String> = Vec::new();

        // 先处理非 pid=0 的会话（同步检查）
        for id in sessions.keys() {
            if let Some(pid_str) = id.split('_').nth(1) {
                if let Ok(pid) = pid_str.parse::<u32>() {
                    if pid != 0 && !active_pids.contains(&pid) {
                        to_remove.push(id.clone());
                    }
                }
            }
        }

        // 再处理 pid=0 的会话（异步检查锁文件）
        let pid_zero_sessions: Vec<(String, String)> = sessions
            .iter()
            .filter_map(|(id, session)| {
                if let Some(pid_str) = id.split('_').nth(1) {
                    if let Ok(pid) = pid_str.parse::<u32>() {
                        if pid == 0 {
                            return Some((id.clone(), session.project_path.clone()));
                        }
                    }
                }
                None
            })
            .collect();

        // 异步检查每个 pid=0 会话的锁文件
        for (id, project_path) in pid_zero_sessions {
            // 使用 flock 检查进程是否存在
            let existence = self.check_process_existence(&PathBuf::from(&project_path)).await;

            // 获取日志更新时间
            let idle_mins = self.get_log_idle_minutes(&project_path).unwrap_or(i64::MAX);

            // 使用 if let 链来处理所有情况
            if let ProcessExistence::Alive = existence {
                // 进程存在，保留
                debug!(
                    "保留 pid=0 会话: {} (进程在运行)",
                    id
                );
            } else if let ProcessExistence::Dead = existence {
                // 进程已死，移除
                debug!(
                    "移除 pid=0 会话: {} (进程已退出)",
                    id
                );
                to_remove.push(id);
            } else if idle_mins >= 2 {
                // NotFound 且日志超时，移除
                debug!(
                    "移除 pid=0 会话: {} (日志 {} 分钟无更新)",
                    id, idle_mins
                );
                to_remove.push(id);
            } else {
                // NotFound 且日志 < 2 分钟，保留（可能是新增场景）
                debug!(
                    "保留 pid=0 会话: {} (新增/初始化中，日志 {} 分钟前更新)",
                    id, idle_mins
                );
            }
        }

        // 移除已结束的会话
        for id in &to_remove {
            debug!("移除已结束的会话: {}", id);
            sessions.remove(id);
        }

        let count = sessions.len();
        info!("刷新完成，当前有 {} 个活跃会话", count);

        Ok(sessions.values().cloned().collect())
    }

    /// 生成固定的会话 ID
    ///
    /// 使用项目路径的哈希而不是时间戳，确保同一项目/进程的会话 ID 保持稳定
    fn generate_session_id(disc: &DiscoveredSession) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // 创建基于 PID 和项目路径的唯一标识
        let unique_key = format!("{}-{}", disc.pid, disc.project_path.display());

        // 计算哈希
        let mut hasher = DefaultHasher::new();
        unique_key.hash(&mut hasher);
        let hash = hasher.finish();

        format!("sess_{}_{}", disc.pid, hash)
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
        let session_id = Self::generate_session_id(disc);

        // 检测初始状态
        let status = if let Some(ref log_path) = disc.log_path {
            StatusDetector::detect(log_path).unwrap_or(SessionStatus::Unknown)
        } else {
            SessionStatus::Unknown
        };

        // 提取第一条用户消息用于标题和摘要
        let first_user_message = if let Some(ref log_path) = disc.log_path {
            let msg = StatusDetector::extract_first_user_message(log_path).ok().flatten();
            // 调试日志：确认第一条用户消息是否正确提取
            if let Some(ref m) = msg {
                tracing::debug!("[{}] 第一条用户消息: {}", disc.project_name, m.content);
            } else {
                tracing::debug!("[{}] 未找到第一条用户消息, log_path: {:?}", disc.project_name, log_path);
            }
            msg
        } else {
            tracing::debug!("[{}] 无日志路径", disc.project_name);
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

    /// 使用 flock 检查进程是否存在
    ///
    /// 返回 ProcessExistence 枚举：
    /// - Alive: 进程确定存在（持锁中）
    /// - NotFound: 找不到锁文件（可能没创建/已退出）
    /// - Dead: 有锁但可加锁（进程已死）
    async fn check_process_existence(&self, project_path: &PathBuf) -> ProcessExistence {
        use nix::fcntl::flock;
        use nix::fcntl::FlockArg;
        use std::os::fd::AsRawFd;

        debug!("[check_process_existence] 检查项目路径: {}", project_path.display());

        // 归一化路径比较（转小写）
        let target_path = project_path.to_string_lossy().to_lowercase();

        // 查找 IDE 目录下的锁文件
        let ide_dir = &self.discovery.ide_dir;
        if !ide_dir.exists() {
            debug!("[check_process_existence] IDE 目录不存在");
            return ProcessExistence::NotFound;
        }

        let mut entries = match tokio::fs::read_dir(ide_dir).await {
            Ok(entries) => entries,
            Err(e) => {
                debug!("[check_process_existence] 读取 IDE 目录失败: {}", e);
                return ProcessExistence::NotFound;
            }
        };

        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.extension() != Some("lock".as_ref()) {
                continue;
            }

            // 读取锁文件内容，检查是否包含目标项目
            match tokio::fs::read_to_string(&path).await {
                Ok(content) => {
                    match serde_json::from_str::<serde_json::Value>(&content) {
                        Ok(lock) => {
                            // 归一化路径比较
                            let workspaces = lock.get("workspaceFolders");
                            if let Some(ws_array) = workspaces {
                                if let Some(ws_vec) = ws_array.as_array() {
                                    let matches = ws_vec.iter().any(|w| {
                                        w.as_str().map(|s| {
                                            let lock_path = s.to_lowercase();
                                            // 支持精确匹配和前缀匹配
                                            lock_path == target_path ||
                                                lock_path.starts_with(&target_path)
                                        }).unwrap_or(false)
                                    });
                                    if !matches {
                                        continue;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            debug!("[check_process_existence] 解析锁文件失败: {}", e);
                            continue;
                        }
                    }
                }
                Err(e) => {
                    debug!("[check_process_existence] 读取锁文件失败: {}", e);
                    continue;
                }
            }

            // 找到匹配的锁文件，尝试获取排他锁
            match std::fs::File::open(&path) {
                Ok(file) => {
                    // 尝试获取非阻塞排他锁
                    #[cfg(unix)]
                    {
                        let fd = file.as_raw_fd();
                        match flock(fd, FlockArg::LockExclusive) {
                            Ok(()) => {
                                // 加锁成功，说明原进程已释放锁（进程已死）
                                let _ = flock(fd, FlockArg::Unlock);
                                debug!("[check_process_existence] 锁可获取，进程已死");
                                return ProcessExistence::Dead;
                            }
                            Err(nix::errno::Errno::EWOULDBLOCK) | Err(nix::errno::Errno::EAGAIN) => {
                                // 加锁失败，说明锁正被占用（进程活着）
                                debug!("[check_process_existence] 锁被占用，进程在运行");
                                return ProcessExistence::Alive;
                            }
                            Err(e) => {
                                debug!("[check_process_existence] flock 错误: {}，保守认为进程存活", e);
                                // 其他错误，保守处理认为进程存活
                                return ProcessExistence::Alive;
                            }
                        }
                    }

                    // Windows: 使用 has_active_lock_file 作为后备
                    #[cfg(windows)]
                    {
                        let has_lock = self
                            .discovery
                            .has_active_lock_file(project_path)
                            .await;
                        return if has_lock { ProcessExistence::Alive } else { ProcessExistence::NotFound };
                    }
                }
                Err(e) => {
                    debug!("[check_process_existence] 打开锁文件失败: {}", e);
                    continue;
                }
            }
        }

        debug!("[check_process_existence] 未找到匹配的锁文件");
        ProcessExistence::NotFound
    }

    /// 获取日志文件的空闲时间（分钟）
    /// 返回 None 表示无法获取（日志文件不存在等）
    fn get_log_idle_minutes(&self, project_path: &str) -> Option<i64> {
        let home = dirs::home_dir()?;
        let encoded = project_path.replace('/', "--").replace('\\', "--");
        let log_dir = home.join(".claude").join("projects").join(encoded);

        // 查找最新的 jsonl 文件
        let latest_file = std::fs::read_dir(&log_dir).ok()?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.path().extension().map(|ext| ext == "jsonl").unwrap_or(false)
            })
            .max_by_key(|entry| {
                entry.metadata().ok()?.modified().ok()
            })?;

        let metadata = latest_file.metadata().ok()?;
        let mtime = metadata.modified().ok()?;
        let mtime: chrono::DateTime<chrono::Utc> = mtime.into();

        let now = chrono::Utc::now();
        Some(now.signed_duration_since(mtime).num_minutes())
    }

    /// 立即刷新（区分新增与存量）
    ///
    /// 策略：
    /// - 新发现的会话：只要日志存在就立即记录，状态为 Initializing
    /// - 已缓存的会话：必须有锁才算存活
    pub async fn instant_refresh(&mut self) -> Result<()> {
        let discovered = self.discovery.discover_sessions().await?;
        let mut sessions = self.sessions.write().await;

        // 收集本轮发现的会话 ID
        let mut current_round_ids: std::collections::HashSet<String> =
            std::collections::HashSet::new();

        for disc in discovered {
            let session_id = Self::generate_session_id(&disc);
            current_round_ids.insert(session_id.clone());

            // 检查会话是否已缓存
            let is_known = sessions.contains_key(&session_id);

            // 判定进程是否存活：
            // - pid != 0：直接检查进程是否存在（kill(pid, 0)）
            // - pid = 0：从日志发现，检查日志是否在 5 分钟内有更新
            let physical_alive = if disc.pid == 0 {
                // pid=0：从日志发现，检查日志更新时间
                let idle_mins = self.get_log_idle_minutes(&disc.project_path.to_string_lossy()).unwrap_or(i64::MAX);
                debug!("[instant_refresh] {} pid=0, 日志空闲 {} 分钟", disc.project_name, idle_mins);
                idle_mins < 5  // 5 分钟内有更新认为存活
            } else {
                // pid!=0：直接检查进程是否存在
                self.discovery.process_exists(disc.pid)
            };

            debug!(
                "[instant_refresh] {} (is_known={}, alive={}, pid={})",
                disc.project_name, is_known, physical_alive, disc.pid
            );

            if !is_known {
                // === 场景：新增实例 ===
                // 只要日志存在就立即记录
                let mut new_session = Self::convert_discovered_to_session(&disc).await?;

                // 根据锁状态设置初始状态
                new_session.status = if physical_alive {
                    SessionStatus::Running
                } else {
                    SessionStatus::Initializing
                };

                sessions.insert(session_id.clone(), new_session);
                debug!(
                    "[instant_refresh] 新增会话: {} (状态: {:?})",
                    disc.project_name,
                    if physical_alive {
                        SessionStatus::Running
                    } else {
                        SessionStatus::Initializing
                    }
                );

                // 发送发现事件
                if let Some(session) = sessions.get(&session_id) {
                    let _ = self
                        .event_sender
                        .send(MonitorEvent::SessionDiscovered {
                            session: session.clone(),
                        })
                        .await;
                }
            } else {
                // === 场景：存量实例检查 ===
                if !physical_alive {
                    // 存量会话没锁了，判定为真正退出
                    sessions.remove(&session_id);
                    let _ = self
                        .event_sender
                        .send(MonitorEvent::SessionEnded {
                            session_id: session_id.clone(),
                        })
                        .await;
                    debug!("[instant_refresh] 移除失效会话: {}", session_id);
                } else {
                    // 如果原来是 Initializing，现在有锁了，自动转 Running
                    if let Some(s) = sessions.get_mut(&session_id) {
                        if s.status == SessionStatus::Initializing {
                            s.status = SessionStatus::Running;
                            debug!("[instant_refresh] {} 状态更新: Initializing -> Running", session_id);
                        }
                    }
                }
            }
        }

        // 补偿：如果磁盘上连日志都没了，强制清理
        sessions.retain(|id, _| current_round_ids.contains(id));

        info!("[instant_refresh] 完成，发现 {} 个会话", current_round_ids.len());

        Ok(())
    }
}

/// 使用 flock 检查锁文件是否被占用
async fn check_physical_alive(lock_path: &PathBuf) -> bool {
    use nix::fcntl::flock;
    use nix::fcntl::FlockArg;
    use std::os::fd::AsRawFd;

    debug!("[check_physical_alive] 检查锁: {}", lock_path.display());

    if !lock_path.exists() {
        debug!("[check_physical_alive] 锁文件不存在");
        return false;
    }

    match std::fs::File::open(lock_path) {
        Ok(file) => {
            let fd = file.as_raw_fd();
            match flock(fd, FlockArg::LockExclusive) {
                Ok(()) => {
                    // 加锁成功，锁未被占用
                    let _ = flock(fd, FlockArg::Unlock);
                    debug!("[check_physical_alive] 锁可获取，进程未运行");
                    false
                }
                Err(nix::errno::Errno::EWOULDBLOCK) | Err(nix::errno::Errno::EAGAIN) => {
                    // 加锁失败，锁正被占用
                    debug!("[check_physical_alive] 锁被占用，进程运行中");
                    true
                }
                Err(e) => {
                    debug!("[check_physical_alive] flock 错误: {}，保守返回 true", e);
                    true
                }
            }
        }
        Err(_) => {
            debug!("[check_physical_alive] 无法打开锁文件");
            false
        }
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
