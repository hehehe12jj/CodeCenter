//! 文件监控模块
//!
//! 使用 notify crate 监控 Claude Code 日志文件的变化。

use crate::error::Result;
use crate::monitor::discovery::{DiscoveredSession, SessionDiscovery};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};

/// 监控事件
#[derive(Debug, Clone)]
pub enum WatchEvent {
    /// 发现新会话
    SessionDiscovered { session: DiscoveredSession },
    /// 会话状态可能变更
    LogChanged { session_id: String, path: PathBuf },
    /// 会话结束（锁文件被删除）
    SessionEnded { session_id: String },
    /// 监控错误
    Error { message: String },
}

/// 文件监控器
pub struct LogWatcher {
    /// notify 监控器实例
    watcher: RecommendedWatcher,
    /// 当前监控的路径集合
    watched_paths: Arc<RwLock<HashSet<PathBuf>>>,
    /// 事件发送通道
    event_sender: mpsc::Sender<WatchEvent>,
    /// 会话发现器
    discovery: SessionDiscovery,
}

impl LogWatcher {
    /// 创建新的文件监控器
    pub fn new(event_sender: mpsc::Sender<WatchEvent>) -> Result<Self> {
        let watched_paths = Arc::new(RwLock::new(HashSet::new()));
        let discovery = SessionDiscovery::new()?;

        // 创建 notify 监控器
        let watcher = Self::create_watcher(event_sender.clone(), watched_paths.clone())?;

        Ok(Self {
            watcher,
            watched_paths,
            event_sender,
            discovery,
        })
    }

    /// 创建底层 notify 监控器
    fn create_watcher(
        event_sender: mpsc::Sender<WatchEvent>,
        watched_paths: Arc<RwLock<HashSet<PathBuf>>>,
    ) -> Result<RecommendedWatcher> {
        let watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            match res {
                Ok(event) => {
                    Self::handle_notify_event(event, &event_sender, &watched_paths);
                }
                Err(e) => {
                    error!("文件监控错误: {}", e);
                    let _ = event_sender.try_send(WatchEvent::Error {
                        message: e.to_string(),
                    });
                }
            }
        })?;

        Ok(watcher)
    }

    /// 处理 notify 事件
    fn handle_notify_event(
        event: Event,
        sender: &mpsc::Sender<WatchEvent>,
        watched_paths: &Arc<RwLock<HashSet<PathBuf>>>,
    ) {
        debug!("收到文件事件: {:?} - {:?}", event.kind, event.paths);

        match event.kind {
            EventKind::Create(_) => {
                // 新文件创建 - 可能是新会话的锁文件或日志
                for path in &event.paths {
                    if Self::is_lock_file(path) {
                        // 新锁文件 - 发现新会话，使用 try_send 避免阻塞
                        let path = path.clone();
                        let sender = sender.clone();
                        // 使用当前运行时句柄来 spawn 任务
                        if let Ok(handle) = tokio::runtime::Handle::try_current() {
                            handle.spawn(async move {
                                if let Some(session) =
                                    Self::parse_lock_file_for_discovery(&path).await
                                {
                                    let _ = sender
                                        .send(WatchEvent::SessionDiscovered { session })
                                        .await;
                                }
                            });
                        }
                    } else if Self::is_log_file(path) {
                        // 新日志文件 - 发送变更事件
                        if let Some(session_id) = Self::extract_session_id_from_log(path) {
                            let _ = sender.try_send(WatchEvent::LogChanged {
                                session_id,
                                path: path.clone(),
                            });
                        }
                    }
                }
            }
            EventKind::Modify(_) => {
                // 文件修改 - 日志更新
                for path in &event.paths {
                    if Self::is_log_file(path) {
                        if let Some(session_id) = Self::extract_session_id_from_log(path) {
                            let _ = sender.try_send(WatchEvent::LogChanged {
                                session_id,
                                path: path.clone(),
                            });
                        }
                    }
                }
            }
            EventKind::Remove(_) => {
                // 文件删除 - 会话结束
                for path in &event.paths {
                    if Self::is_lock_file(path) {
                        if let Some(session_id) = Self::extract_session_id_from_lock(path) {
                            // 从监控集合中移除
                            let watched_paths = watched_paths.clone();
                            let path = path.clone();
                            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                                handle.spawn(async move {
                                    let mut paths = watched_paths.write().await;
                                    paths.remove(&path);
                                });
                            }

                            let _ = sender.try_send(WatchEvent::SessionEnded { session_id });
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// 初始化监控
    ///
    /// 1. 监控 IDE 目录（发现新锁文件）
    /// 2. 监控已存在的会话日志
    pub async fn initialize(&mut self) -> Result<()> {
        info!("初始化文件监控...");

        // 监控锁文件目录
        let ide_dir = self.discovery.ide_dir.clone();
        if ide_dir.exists() {
            self.watcher.watch(&ide_dir, RecursiveMode::NonRecursive)?;
            info!("开始监控锁文件目录: {:?}", ide_dir);
        }

        // 发现已存在的会话并开始监控
        match self.discovery.discover_sessions().await {
            Ok(sessions) => {
                for session in sessions {
                    if let Some(log_path) = &session.log_path {
                        if let Err(e) = self.watch_log(log_path).await {
                            warn!("监控日志文件失败 {:?}: {}", log_path, e);
                        }
                    }
                }
            }
            Err(e) => {
                warn!("发现会话失败: {}", e);
            }
        }

        Ok(())
    }

    /// 开始监控指定日志文件
    pub async fn watch_log(&mut self,
        path: &Path,
    ) -> Result<()> {
        // 检查是否已在监控
        {
            let watched = self.watched_paths.read().await;
            if watched.contains(path) {
                return Ok(());
            }
        }

        // 添加监控
        self.watcher.watch(path, RecursiveMode::NonRecursive)?;

        // 记录监控路径
        {
            let mut watched = self.watched_paths.write().await;
            watched.insert(path.to_path_buf());
        }

        debug!("开始监控日志文件: {:?}", path);
        Ok(())
    }

    /// 停止监控
    pub async fn unwatch(
        &mut self,
        path: &Path,
    ) {
        {
            let mut watched = self.watched_paths.write().await;
            watched.remove(path);
        }

        if let Err(e) = self.watcher.unwatch(path) {
            warn!("取消监控失败 {:?}: {}", path, e);
        } else {
            debug!("停止监控: {:?}", path);
        }
    }

    /// 检查是否是锁文件
    fn is_lock_file(path: &Path) -> bool {
        path.extension() == Some("lock".as_ref())
    }

    /// 检查是否是日志文件
    fn is_log_file(path: &Path) -> bool {
        path.extension() == Some("jsonl".as_ref())
    }

    /// 从锁文件路径提取会话 ID
    fn extract_session_id_from_lock(path: &Path) -> Option<String> {
        path.file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
    }

    /// 从日志文件路径提取会话 ID
    fn extract_session_id_from_log(path: &Path) -> Option<String> {
        path.file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
    }

    /// 解析锁文件以发现会话
    async fn parse_lock_file_for_discovery(path: &Path) -> Option<DiscoveredSession> {
        // 简化实现，实际应该复用 SessionDiscovery 的逻辑
        // 这里仅返回基本信息
        let content = tokio::fs::read_to_string(path).await.ok()?;
        let lock: serde_json::Value = serde_json::from_str(&content).ok()?;

        let pid = lock.get("pid")?.as_u64()? as u32;
        let workspace = lock
            .get("workspaceFolders")?
            .as_array()?
            .first()?
            .as_str()?;

        let project_path = PathBuf::from(workspace);
        let project_name = project_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        Some(DiscoveredSession {
            pid,
            project_path,
            project_name,
            log_path: None,
            start_time: None,
        })
    }
}

/// 轮询模式监控器
///
/// 当系统达到文件监控限制时使用
pub struct PollingWatcher {
    paths: Vec<PathBuf>,
    interval: std::time::Duration,
    event_sender: mpsc::Sender<WatchEvent>,
    last_modified: Arc<RwLock<std::collections::HashMap<PathBuf, std::time::SystemTime>>>,
}

impl PollingWatcher {
    /// 创建新的轮询监控器
    pub fn new(
        paths: Vec<PathBuf>,
        interval: std::time::Duration,
        event_sender: mpsc::Sender<WatchEvent>,
    ) -> Self {
        Self {
            paths,
            interval,
            event_sender,
            last_modified: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// 启动轮询
    pub async fn run(self) {
        let mut interval = tokio::time::interval(self.interval);

        loop {
            interval.tick().await;

            for path in &self.paths {
                if let Err(e) = self.check_file(path).await {
                    debug!("轮询检查文件失败 {:?}: {}", path, e);
                }
            }
        }
    }

    /// 检查单个文件
    async fn check_file(
        &self,
        path: &Path,
    ) -> Result<()> {
        let metadata = tokio::fs::metadata(path).await?;
        let modified = metadata.modified()?;

        let should_notify = {
            let last = self.last_modified.read().await;
            match last.get(path) {
                Some(last_modified) if &modified > last_modified => true,
                None => true,
                _ => false,
            }
        };

        if should_notify {
            let mut last = self.last_modified.write().await;
            last.insert(path.to_path_buf(), modified);

            if let Some(session_id) = Self::extract_session_id(path) {
                let _ = self
                    .event_sender
                    .send(WatchEvent::LogChanged {
                        session_id,
                        path: path.to_path_buf(),
                    })
                    .await;
            }
        }

        Ok(())
    }

    /// 提取会话 ID
    fn extract_session_id(path: &Path) -> Option<String> {
        path.file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
    }
}

/// 监控管理器
pub struct WatchManager {
    watcher: LogWatcher,
    event_receiver: Option<mpsc::Receiver<WatchEvent>>,
}

impl WatchManager {
    /// 创建并初始化监控管理器
    pub async fn new() -> Result<Self> {
        let (tx, rx) = mpsc::channel(100);
        let watcher = LogWatcher::new(tx)?;

        Ok(Self {
            watcher,
            event_receiver: Some(rx),
        })
    }

    /// 获取事件接收器的可变引用
    pub fn event_stream(&mut self) -> Option<&mut mpsc::Receiver<WatchEvent>> {
        self.event_receiver.as_mut()
    }

    /// 获取事件接收器的所有权
    /// 用于在需要 move 接收器的场景
    pub fn take_event_stream(&mut self) -> Option<mpsc::Receiver<WatchEvent>> {
        self.event_receiver.take()
    }

    /// 初始化并开始监控
    pub async fn initialize(
        &mut self,
    ) -> Result<()> {
        self.watcher.initialize().await
    }

    /// 添加会话日志监控
    pub async fn watch_session(
        &mut self,
        log_path: &Path,
    ) -> Result<()> {
        self.watcher.watch_log(log_path).await
    }

    /// 移除会话监控
    pub async fn unwatch_session(
        &mut self,
        log_path: &Path,
    ) {
        self.watcher.unwatch(log_path).await;
    }
}
