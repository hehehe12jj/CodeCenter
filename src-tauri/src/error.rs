use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Project not found: {0}")]
    ProjectNotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Process error: {0}")]
    ProcessError(String),

    #[error("Monitor error: {0}")]
    MonitorError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Serialization(err.to_string())
    }
}

impl From<notify::Error> for AppError {
    fn from(err: notify::Error) -> Self {
        AppError::MonitorError(err.to_string())
    }
}

impl From<AppError> for String {
    fn from(err: AppError) -> String {
        err.to_string()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
