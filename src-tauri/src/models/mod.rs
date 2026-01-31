use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 自定义时间序列化模块 - 使用 RFC3339 格式
mod datetime_serde {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(datetime: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&datetime.to_rfc3339())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        DateTime::parse_from_rfc3339(&s)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(serde::de::Error::custom)
    }
}

pub mod status;

pub use status::SessionStatus;

/// 会话唯一标识符
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub String);

impl SessionId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

/// 会话基础信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: String,
    pub title: String,
    pub project_name: String,
    pub project_path: String,
    pub agent_type: String,
    pub status: SessionStatus,
    #[serde(with = "datetime_serde")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "datetime_serde")]
    pub last_active_at: DateTime<Utc>,
    pub summary: Option<String>,
    pub is_archived: bool,
}

impl Session {
    pub fn new(
        title: impl Into<String>,
        project_name: impl Into<String>,
        project_path: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: SessionId::new().0,
            title: title.into(),
            project_name: project_name.into(),
            project_path: project_path.into(),
            agent_type: "claude".to_string(),
            status: SessionStatus::Running,
            created_at: now,
            last_active_at: now,
            summary: None,
            is_archived: false,
        }
    }
}

/// 消息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: String,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: Option<MessageMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageMetadata {
    pub has_code: bool,
    pub token_count: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
}

/// 会话详情
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionDetail {
    #[serde(flatten)]
    pub session: Session,
    pub messages: Vec<Message>,
    pub process_info: Option<ProcessInfo>,
    pub stats: SessionStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessInfo {
    pub pid: u32,
    pub start_time: DateTime<Utc>,
    pub command_line: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionStats {
    pub message_count: u32,
    pub total_tokens: Option<u32>,
    pub duration_secs: u64,
}

/// 项目信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub path: String,
    pub name: String,
    pub last_accessed_at: DateTime<Utc>,
    pub session_count: u32,
    pub config: Option<ProjectConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectConfig {
    pub auto_start: bool,
    pub default_agent: String,
    pub env_vars: std::collections::HashMap<String, String>,
}

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub version: String,
    pub settings: Settings,
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub auto_refresh_interval_ms: u64,
    pub max_session_history: usize,
    pub notification_enabled: bool,
    pub message_load_limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiConfig {
    pub theme: String,
    pub sidebar_collapsed: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: "0.1.0".to_string(),
            settings: Settings {
                auto_refresh_interval_ms: 5000,
                max_session_history: 100,
                notification_enabled: true,
                message_load_limit: 30,
            },
            ui: UiConfig {
                theme: "dark".to_string(),
                sidebar_collapsed: false,
            },
        }
    }
}
