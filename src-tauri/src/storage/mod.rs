//! 本地 JSON 文件存储模块
//!
//! 提供会话、项目、配置的持久化存储功能
//! 数据目录: ~/.codeagent/

use crate::error::{AppError, Result};
use crate::models::{Project, Session, SessionDetail};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;

pub mod config;

pub use config::ConfigStorage;

/// 存储管理器
#[derive(Debug, Clone)]
pub struct Storage {
    /// 数据根目录
    data_dir: PathBuf,
}

impl Storage {
    /// 创建存储管理器实例
    pub async fn new() -> Result<Self> {
        let data_dir = Self::data_dir()?;
        Self::ensure_dir(&data_dir).await?;
        Self::ensure_dir(&data_dir.join("sessions")).await?;
        Self::ensure_dir(&data_dir.join("cache")).await?;

        Ok(Self { data_dir })
    }

    /// 获取数据目录路径 (~/.codeagent/)
    pub fn data_dir() -> Result<PathBuf> {
        dirs::home_dir()
            .map(|home| home.join(".codeagent"))
            .ok_or_else(|| AppError::StorageError("无法获取用户主目录".to_string()))
    }

    /// 确保目录存在
    async fn ensure_dir(path: &Path) -> Result<()> {
        if !path.exists() {
            fs::create_dir_all(path)
                .await
                .map_err(|e| AppError::StorageError(format!("创建目录失败: {}", e)))?;
        }
        Ok(())
    }

    /// 保存会话列表索引
    pub async fn save_session_index(&self, sessions: &[Session]) -> Result<()> {
        let path = self.data_dir.join("sessions").join("index.json");
        let sessions_vec: Vec<Session> = sessions.to_vec();
        self.write_json(path, &sessions_vec).await
    }

    /// 读取会话列表索引
    pub async fn load_session_index(&self) -> Result<Vec<Session>> {
        let path = self.data_dir.join("sessions").join("index.json");
        if !path.exists() {
            return Ok(vec![]);
        }
        self.read_json(path).await
    }

    /// 保存会话详情
    pub async fn save_session_detail(&self, detail: &SessionDetail) -> Result<()> {
        let filename = format!("{}.json", detail.session.id);
        let path = self.data_dir.join("sessions").join(filename);
        self.write_json(path, detail).await
    }

    /// 读取会话详情
    pub async fn load_session_detail(&self, session_id: &str) -> Result<SessionDetail> {
        let filename = format!("{}.json", session_id);
        let path = self.data_dir.join("sessions").join(filename);
        self.read_json(path).await
    }

    /// 删除会话详情
    pub async fn delete_session_detail(&self, session_id: &str) -> Result<()> {
        let filename = format!("{}.json", session_id);
        let path = self.data_dir.join("sessions").join(filename);
        fs::remove_file(&path)
            .await
            .map_err(|e| AppError::StorageError(format!("删除文件失败: {}", e)))?;
        Ok(())
    }

    /// 保存项目列表
    pub async fn save_projects(&self, projects: &[Project]) -> Result<()> {
        let path = self.data_dir.join("projects.json");
        let projects_vec: Vec<Project> = projects.to_vec();
        self.write_json(path, &projects_vec).await
    }

    /// 读取项目列表
    pub async fn load_projects(&self) -> Result<Vec<Project>> {
        let path = self.data_dir.join("projects.json");
        if !path.exists() {
            return Ok(vec![]);
        }
        self.read_json(path).await
    }

    /// 写入 JSON 文件
    async fn write_json<T: serde::Serialize>(&self, path: PathBuf, data: &T) -> Result<()> {
        let json = serde_json::to_string_pretty(data)
            .map_err(|e| AppError::Serialization(e.to_string()))?;

        let mut file = fs::File::create(&path)
            .await
            .map_err(|e| AppError::StorageError(format!("创建文件失败: {}", e)))?;

        file.write_all(json.as_bytes())
            .await
            .map_err(|e| AppError::StorageError(format!("写入文件失败: {}", e)))?;

        Ok(())
    }

    /// 读取 JSON 文件
    async fn read_json<T: serde::de::DeserializeOwned>(&self, path: PathBuf) -> Result<T> {
        if !path.exists() {
            return Err(AppError::StorageError("文件不存在".to_string()));
        }

        let content = fs::read_to_string(&path)
            .await
            .map_err(|e| AppError::StorageError(format!("读取文件失败: {}", e)))?;

        let data = serde_json::from_str(&content)
            .map_err(|e| AppError::Serialization(e.to_string()))?;

        Ok(data)
    }

    /// 获取所有活跃会话（未归档）
    pub async fn get_active_sessions(&self) -> Result<Vec<Session>> {
        let sessions = self.load_session_index().await?;
        Ok(sessions.into_iter().filter(|s| !s.is_archived).collect())
    }

    /// 获取归档会话
    pub async fn get_archived_sessions(&self) -> Result<Vec<Session>> {
        let sessions = self.load_session_index().await?;
        Ok(sessions.into_iter().filter(|s| s.is_archived).collect())
    }

    /// 更新会话
    pub async fn update_session(&self, updated: &Session) -> Result<()> {
        let mut sessions = self.load_session_index().await?;

        if let Some(index) = sessions.iter().position(|s| s.id == updated.id) {
            sessions[index] = updated.clone();
        } else {
            sessions.push(updated.clone());
        }

        self.save_session_index(&sessions).await
    }

    /// 根据 ID 获取会话
    pub async fn get_session(&self, id: &str) -> Result<Session> {
        let sessions = self.load_session_index().await?;
        sessions
            .into_iter()
            .find(|s| s.id == id)
            .ok_or_else(|| AppError::SessionNotFound(id.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_storage() -> (Storage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage {
            data_dir: temp_dir.path().to_path_buf(),
        };
        (storage, temp_dir)
    }

    #[tokio::test]
    async fn test_session_index_roundtrip() {
        let (storage, _temp) = create_test_storage().await;

        let sessions = vec![
            Session::new("测试会话1", "project1", "/path/to/project1"),
            Session::new("测试会话2", "project2", "/path/to/project2"),
        ];

        storage.save_session_index(&sessions).await.unwrap();
        let loaded = storage.load_session_index().await.unwrap();

        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].title, "测试会话1");
        assert_eq!(loaded[1].title, "测试会话2");
    }
}
