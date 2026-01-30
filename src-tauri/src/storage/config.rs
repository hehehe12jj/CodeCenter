//! 配置存储管理

use crate::error::{AppError, Result};
use crate::models::AppConfig;
use std::path::PathBuf;
use tokio::fs;

pub struct ConfigStorage;

impl ConfigStorage {
    /// 获取配置文件路径
    pub fn config_path() -> Result<PathBuf> {
        let data_dir = super::Storage::data_dir()?;
        Ok(data_dir.join("config.json"))
    }

    /// 加载配置，如果不存在则创建默认配置
    pub async fn load() -> Result<AppConfig> {
        let path = Self::config_path()?;

        if !path.exists() {
            let default_config = AppConfig::default();
            Self::save(&default_config).await?;
            return Ok(default_config);
        }

        let content = fs::read_to_string(&path)
            .await
            .map_err(|e| AppError::StorageError(format!("读取配置失败: {}", e)))?;

        let config: AppConfig = serde_json::from_str(&content)
            .map_err(|e| AppError::Serialization(e.to_string()))?;

        Ok(config)
    }

    /// 保存配置
    pub async fn save(config: &AppConfig) -> Result<()> {
        let path = Self::config_path()?;

        // 确保目录存在
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| AppError::StorageError(format!("创建配置目录失败: {}", e)))?;
        }

        let json = serde_json::to_string_pretty(config)
            .map_err(|e| AppError::Serialization(e.to_string()))?;

        fs::write(&path, json)
            .await
            .map_err(|e| AppError::StorageError(format!("保存配置失败: {}", e)))?;

        Ok(())
    }

    /// 更新配置（部分更新）
    pub async fn update<F>(f: F) -> Result<()>
    where
        F: FnOnce(&mut AppConfig),
    {
        let mut config = Self::load().await?;
        f(&mut config);
        Self::save(&config).await
    }
}
