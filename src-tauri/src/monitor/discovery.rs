//! 会话发现模块
//!
//! 扫描 Claude Code 的锁文件和日志目录，发现活跃会话。

use crate::error::{AppError, Result};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// 会话发现器
#[derive(Debug, Clone)]
pub struct SessionDiscovery {
    /// Claude Code 配置目录 (~/.claude)
    pub claude_dir: PathBuf,
    /// IDE 锁文件目录 (~/.claude/ide)
    pub ide_dir: PathBuf,
    /// 项目日志目录 (~/.claude/projects)
    pub projects_dir: PathBuf,
}

/// 锁文件内容结构
#[derive(Debug, Deserialize)]
struct LockFile {
    pid: u32,
    #[serde(rename = "workspaceFolders")]
    workspace_folders: Vec<String>,
    #[serde(rename = "ideName")]
    ide_name: String,
}

/// 发现的原始会话信息
#[derive(Debug, Clone)]
pub struct DiscoveredSession {
    pub pid: u32,
    pub project_path: PathBuf,
    pub project_name: String,
    pub log_path: Option<PathBuf>,
    pub start_time: Option<DateTime<Utc>>,
}

impl SessionDiscovery {
    /// 创建新的会话发现器
    pub fn new() -> Result<Self> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| AppError::StorageError("无法获取用户主目录".to_string()))?;

        let claude_dir = home_dir.join(".claude");
        let ide_dir = claude_dir.join("ide");
        let projects_dir = claude_dir.join("projects");

        Ok(Self {
            claude_dir,
            ide_dir,
            projects_dir,
        })
    }

    /// 发现所有活跃会话
    ///
    /// 策略：
    /// 1. 扫描 ~/.claude/ide/*.lock 文件获取活跃 PID 和项目路径
    /// 2. 扫描 ~/.claude/projects/ 目录下的日志文件
    /// 3. 合并两个来源，去重
    pub async fn discover_sessions(&self) -> Result<Vec<DiscoveredSession>> {
        let mut sessions: HashMap<PathBuf, DiscoveredSession> = HashMap::new();

        // 1. 从锁文件发现会话
        match self.discover_from_lock_files().await {
            Ok(lock_sessions) => {
                for session in lock_sessions {
                    sessions.insert(session.project_path.clone(), session);
                }
            }
            Err(e) => {
                warn!("从锁文件发现会话失败: {}", e);
            }
        }

        // 2. 从日志目录发现会话
        match self.discover_from_logs().await {
            Ok(log_sessions) => {
                for session in log_sessions {
                    // 如果锁文件已发现该项目，补充日志路径
                    if let Some(existing) = sessions.get_mut(&session.project_path) {
                        if existing.log_path.is_none() && session.log_path.is_some() {
                            existing.log_path = session.log_path;
                        }
                    } else {
                        sessions.insert(session.project_path.clone(), session);
                    }
                }
            }
            Err(e) => {
                warn!("从日志目录发现会话失败: {}", e);
            }
        }

        let result: Vec<DiscoveredSession> = sessions.into_values().collect();
        info!("发现 {} 个活跃会话", result.len());

        Ok(result)
    }

    /// 从锁文件发现会话
    async fn discover_from_lock_files(&self) -> Result<Vec<DiscoveredSession>> {
        let mut sessions = Vec::new();

        if !self.ide_dir.exists() {
            debug!("IDE 目录不存在: {:?}", self.ide_dir);
            return Ok(sessions);
        }

        let mut entries = tokio::fs::read_dir(&self.ide_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            // 只处理 .lock 文件
            if path.extension() != Some("lock".as_ref()) {
                continue;
            }

            match self.parse_lock_file(&path).await {
                Ok(Some(session)) => {
                    debug!(
                        "从锁文件发现会话: pid={}, path={:?}",
                        session.pid, session.project_path
                    );
                    sessions.push(session);
                }
                Ok(None) => {
                    debug!("跳过无效锁文件: {:?}", path);
                }
                Err(e) => {
                    warn!("解析锁文件失败 {:?}: {}", path, e);
                }
            }
        }

        Ok(sessions)
    }

    /// 解析单个锁文件
    async fn parse_lock_file(&self, path: &Path) -> Result<Option<DiscoveredSession>> {
        let content = tokio::fs::read_to_string(path).await?;
        let lock: LockFile = serde_json::from_str(&content)?;

        // 验证进程是否仍然存在
        if !self.process_exists(lock.pid) {
            debug!("进程 {} 不存在，跳过", lock.pid);
            return Ok(None);
        }

        // 获取第一个工作目录
        let project_path = lock
            .workspace_folders
            .first()
            .ok_or_else(|| AppError::StorageError("锁文件中没有工作目录".to_string()))?;

        let project_path = PathBuf::from(project_path);
        let project_name = project_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // 尝试获取日志文件路径
        let log_path = self.find_log_file(&project_path).await.ok();

        // 获取进程启动时间
        let start_time = self.get_process_start_time(lock.pid).ok();

        Ok(Some(DiscoveredSession {
            pid: lock.pid,
            project_path,
            project_name,
            log_path,
            start_time,
        }))
    }

    /// 从日志目录发现会话
    async fn discover_from_logs(&self) -> Result<Vec<DiscoveredSession>> {
        let mut sessions = Vec::new();

        if !self.projects_dir.exists() {
            debug!("项目目录不存在: {:?}", self.projects_dir);
            return Ok(sessions);
        }

        let mut entries = tokio::fs::read_dir(&self.projects_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let project_encoded_dir = entry.path();

            if !project_encoded_dir.is_dir() {
                continue;
            }

            // 尝试从该项目的日志文件中发现会话
            match self.discover_project_logs(&project_encoded_dir).await {
                Ok(Some(session)) => {
                    sessions.push(session);
                }
                Ok(None) => {}
                Err(e) => {
                    warn!("扫描项目日志失败 {:?}: {}", project_encoded_dir, e);
                }
            }
        }

        Ok(sessions)
    }

    /// 扫描单个项目的日志
    async fn discover_project_logs(
        &self,
        project_dir: &Path,
    ) -> Result<Option<DiscoveredSession>> {
        let mut entries = tokio::fs::read_dir(project_dir).await?;

        // 查找最近的 .jsonl 文件
        let mut latest_log: Option<(PathBuf, DateTime<Utc>)> = None;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.extension() != Some("jsonl".as_ref()) {
                continue;
            }

            // 获取文件修改时间
            let metadata = entry.metadata().await?;
            let modified = metadata.modified()?;
            let modified: DateTime<Utc> = modified.into();

            match &latest_log {
                None => latest_log = Some((path, modified)),
                Some((_, last_modified)) if modified > *last_modified => {
                    latest_log = Some((path, modified));
                }
                _ => {}
            }
        }

        let (log_path, last_modified) = match latest_log {
            Some(l) => l,
            None => return Ok(None),
        };

        // 如果日志文件超过 30 分钟没有更新，认为是旧会话
        let now = Utc::now();
        let duration = now.signed_duration_since(last_modified);
        if duration.num_minutes() > 30 {
            debug!("日志文件 {:?} 已过期，跳过", log_path);
            return Ok(None);
        }

        // 从路径解析项目信息
        // 路径格式: ~/.claude/projects/{encoded-project-path}/{session-id}.jsonl
        let project_encoded = project_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let project_path = decode_project_path(project_encoded);
        let project_name = project_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(Some(DiscoveredSession {
            pid: 0, // 从日志发现无法获取 PID
            project_path,
            project_name,
            log_path: Some(log_path),
            start_time: Some(last_modified),
        }))
    }

    /// 查找项目的日志文件
    async fn find_log_file(&self, project_path: &Path) -> Result<PathBuf> {
        let encoded = encode_project_path(project_path);
        let project_dir = self.projects_dir.join(&encoded);

        if !project_dir.exists() {
            return Err(AppError::StorageError(format!(
                "项目日志目录不存在: {:?}",
                project_dir
            )));
        }

        let mut entries = tokio::fs::read_dir(&project_dir).await?;
        let mut latest_log: Option<(PathBuf, DateTime<Utc>)> = None;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.extension() != Some("jsonl".as_ref()) {
                continue;
            }

            let metadata = entry.metadata().await?;
            let modified = metadata.modified()?;
            let modified: DateTime<Utc> = modified.into();

            match &latest_log {
                None => latest_log = Some((path, modified)),
                Some((_, last_modified)) if modified > *last_modified => {
                    latest_log = Some((path, modified));
                }
                _ => {}
            }
        }

        latest_log
            .map(|(path, _)| path)
            .ok_or_else(|| AppError::StorageError("未找到日志文件".to_string()))
    }

    /// 检查进程是否存在
    pub fn process_exists(&self, pid: u32) -> bool {
        #[cfg(unix)]
        {
            // Unix: 发送信号 0 检查进程是否存在
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;

            kill(Pid::from_raw(pid as i32), Signal::SIGCONT).is_ok()
        }

        #[cfg(windows)]
        {
            // Windows: 尝试打开进程
            use windows::Win32::Foundation::{CloseHandle, HANDLE};
            use windows::Win32::System::Threading::OpenProcess;
            use windows::Win32::System::Threading::PROCESS_QUERY_INFORMATION;

            unsafe {
                let handle: HANDLE = OpenProcess(PROCESS_QUERY_INFORMATION, false, pid);
                if handle.is_invalid() {
                    false
                } else {
                    CloseHandle(handle);
                    true
                }
            }
        }
    }

    /// 获取进程启动时间
    fn get_process_start_time(&self, pid: u32) -> Result<DateTime<Utc>> {
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;

            let output = Command::new("ps")
                .args(["-o", "lstart=", "-p", &pid.to_string()])
                .output()
                .map_err(|e| AppError::Internal(e.to_string()))?;

            let lstart = String::from_utf8_lossy(&output.stdout);
            // 解析 ps 输出的日期格式
            // Mon Jan 29 10:15:30 2026
            let naive_dt = chrono::NaiveDateTime::parse_from_str(
                lstart.trim(),
                "%a %b %e %H:%M:%S %Y",
            )
            .map_err(|e| AppError::Internal(format!("解析进程启动时间失败: {}", e)))?;

            Ok(DateTime::from_naive_utc_and_offset(naive_dt, Utc))
        }

        #[cfg(target_os = "linux")]
        {
            // Linux: 从 /proc/{pid}/stat 读取
            let stat_path = format!("/proc/{}/stat", pid);
            let content = std::fs::read_to_string(&stat_path)
                .map_err(|e| AppError::Internal(format!("读取进程状态失败: {}", e)))?;

            // 解析启动时间（第22个字段，单位为 clock ticks）
            // 这里简化处理，使用文件修改时间作为近似
            let metadata = std::fs::metadata(&stat_path)
                .map_err(|e| AppError::Internal(format!("读取文件元数据失败: {}", e)))?;

            let created: DateTime<Utc> = metadata
                .modified()
                .map_err(|e| AppError::Internal(format!("获取修改时间失败: {}", e)))?
                .into();

            Ok(created)
        }

        #[cfg(windows)]
        {
            // Windows: 使用 WinAPI 获取进程创建时间
            // 简化处理，返回当前时间
            Ok(Utc::now())
        }
    }

    /// 检查项目是否有对应的活跃锁文件
    /// 遍历所有锁文件，检查 workspace_folders 是否包含目标项目路径
    pub async fn has_active_lock_file(&self, project_path: &Path) -> bool {
        let project_path_str = project_path.to_string_lossy();
        debug!("[has_active_lock_file] 检查项目: {}", project_path_str);

        if !self.ide_dir.exists() {
            debug!("[has_active_lock_file] IDE 目录不存在: {:?}", self.ide_dir);
            return false;
        }

        let mut entries = match tokio::fs::read_dir(&self.ide_dir).await {
            Ok(entries) => entries,
            Err(e) => {
                debug!("[has_active_lock_file] 读取目录失败: {}", e);
                return false;
            }
        };

        let mut lock_file_count = 0;
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.extension() != Some("lock".as_ref()) {
                continue;
            }
            lock_file_count += 1;

            debug!("[has_active_lock_file] 检查锁文件: {:?}", path);

            // 解析锁文件内容
            match tokio::fs::read_to_string(&path).await {
                Ok(content) => {
                    match serde_json::from_str::<LockFile>(&content) {
                        Ok(lock) => {
                            debug!("[has_active_lock_file] 锁文件 PID: {}, workspaces: {:?}", lock.pid, lock.workspace_folders);

                            // 检查 workspace_folders 是否包含目标项目路径
                            let found = lock.workspace_folders.iter().any(|p| {
                                let matches = PathBuf::from(p) == project_path;
                                debug!("[has_active_lock_file] 比较: '{}' vs '{}' = {}", p, project_path_str, matches);
                                matches
                            });

                            if found {
                                // 额外检查进程是否仍然存在
                                if self.process_exists(lock.pid) {
                                    debug!("[has_active_lock_file] 找到匹配且进程存在, PID: {}", lock.pid);
                                    return true;
                                } else {
                                    debug!("[has_active_lock_file] 找到匹配但进程不存在, PID: {}", lock.pid);
                                }
                            }
                        }
                        Err(e) => {
                            debug!("[has_active_lock_file] 解析锁文件 JSON 失败: {}", e);
                        }
                    }
                }
                Err(e) => {
                    debug!("[has_active_lock_file] 读取锁文件失败: {}", e);
                }
            }
        }

        debug!("[has_active_lock_file] 检查了 {} 个锁文件, 未找到匹配", lock_file_count);
        false
    }
}

/// 编码项目路径为文件名安全的字符串
/// Claude Code 使用的编码方式：将 / 替换为 -
fn encode_project_path(path: &Path) -> String {
    let path_str = path.to_string_lossy();
    // 替换路径分隔符为单连字符（与 Claude Code 日志目录格式一致）
    path_str.replace('/', "-").replace('\\', "-")
}

/// 解码项目路径
fn decode_project_path(encoded: &str) -> PathBuf {
    // 将单连字符替换回路径分隔符
    let decoded = encoded.replace('-', "/");
    PathBuf::from(decoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_encoding() {
        let path = PathBuf::from("/Users/hejj/projects/backend-api");
        let encoded = encode_project_path(&path);
        let decoded = decode_project_path(&encoded);
        assert_eq!(path, decoded);
    }
}
