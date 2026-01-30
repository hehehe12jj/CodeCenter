use serde::{Deserialize, Serialize};

/// 会话状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    /// 运行中 - 脉冲绿点
    Running,
    /// 等待输入 - 黄色点
    WaitingInput,
    /// 已完成 - 蓝色点
    Completed,
    /// 执行阻塞 - 红色脉冲
    Blocked,
    /// 未知状态
    Unknown,
}

impl Default for SessionStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

impl SessionStatus {
    /// 获取状态显示文本
    pub fn display_name(&self) -> &'static str {
        match self {
            SessionStatus::Running => "运行中",
            SessionStatus::WaitingInput => "等待输入",
            SessionStatus::Completed => "已完成",
            SessionStatus::Blocked => "执行阻塞",
            SessionStatus::Unknown => "未知",
        }
    }

    /// 获取状态颜色（用于前端显示）
    pub fn color(&self) -> &'static str {
        match self {
            SessionStatus::Running => "#22c55e",
            SessionStatus::WaitingInput => "#eab308",
            SessionStatus::Completed => "#3b82f6",
            SessionStatus::Blocked => "#ef4444",
            SessionStatus::Unknown => "#6b7280",
        }
    }

    /// 是否显示脉冲动画
    pub fn is_pulsing(&self) -> bool {
        matches!(self, SessionStatus::Running | SessionStatus::Blocked)
    }
}
