use crate::error::Result;
use crate::models::AppConfig;
use crate::monitor::SessionMonitor;
use crate::storage::{config::ConfigStorage, Storage};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 全局应用状态
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<RwLock<AppConfig>>,
    pub storage: Arc<Storage>,
    pub monitor: Arc<RwLock<SessionMonitor>>,
}

impl AppState {
    /// 异步初始化应用状态
    pub async fn init() -> Result<Self> {
        // 先创建存储，它会创建必要的目录
        let storage = Storage::new().await?;
        // 再加载配置（现在目录已存在）
        let config = ConfigStorage::load().await?;
        let monitor = SessionMonitor::new().await?;

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            storage: Arc::new(storage),
            monitor: Arc::new(RwLock::new(monitor)),
        })
    }

    /// 获取存储实例
    pub fn storage(&self) -> &Storage {
        &self.storage
    }

    /// 重新加载配置
    pub async fn reload_config(&self) -> Result<()> {
        let new_config = ConfigStorage::load().await?;
        let mut config = self.config.write().await;
        *config = new_config;
        Ok(())
    }

    /// 保存配置
    pub async fn save_config(&self) -> Result<()> {
        let config = self.config.read().await;
        ConfigStorage::save(&*config).await
    }
}
