//! 状态检测模块
//!
//! 解析 Claude Code 的 JSONL 日志文件，推断会话状态。

use crate::error::Result;
use crate::models::{Message, MessageMetadata, MessageRole, SessionStatus};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, trace};

/// 状态检测器
#[derive(Debug, Clone)]
pub struct StatusDetector;

/// Claude Code 日志事件
#[derive(Debug, Deserialize)]
struct LogEvent {
    #[serde(rename = "type")]
    event_type: String,
    #[serde(rename = "sessionId")]
    session_id: Option<String>,
    timestamp: DateTime<Utc>,
    uuid: Option<String>,
    #[serde(rename = "parentUuid")]
    parent_uuid: Option<String>,
    message: Option<LogMessage>,
    #[serde(default)]
    content: Option<Vec<ContentBlock>>,
}

#[derive(Debug, Deserialize)]
struct LogMessage {
    role: String,
    content: Vec<ContentBlock>,
}

#[derive(Debug, Deserialize, Clone)]
struct ContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    text: Option<String>,
    #[serde(rename = "tool_use")]
    tool_use: Option<serde_json::Value>,
    thinking: Option<String>,
}

impl StatusDetector {
    /// 从日志文件检测当前状态
    pub fn detect(log_path: &Path) -> Result<SessionStatus> {
        // 读取最后几条事件
        let events = Self::read_last_events(log_path, 5)?;

        if events.is_empty() {
            return Ok(SessionStatus::Unknown);
        }

        // 分析最后一条事件
        let last_event = events.last().unwrap();
        Self::infer_from_event(last_event, &events)
    }

    /// 提取最近 N 条消息
    pub fn extract_recent_messages(log_path: &Path, limit: usize) -> Result<Vec<Message>> {
        let events = Self::read_last_events(log_path, limit * 2)?;
        let mut messages = Vec::new();

        for event in events {
            if let Some(message) = Self::convert_to_message(&event) {
                messages.push(message);
            }
        }

        // 限制数量并反转顺序（最新的在最后）
        if messages.len() > limit {
            messages = messages.split_off(messages.len() - limit);
        }

        Ok(messages)
    }

    /// 提取第一条用户消息
    pub fn extract_first_user_message(log_path: &Path) -> Result<Option<Message>> {
        if !log_path.exists() {
            return Ok(None);
        }

        // 读取文件内容
        let content = std::fs::read_to_string(log_path)?;

        // 解析所有行，按时间排序
        let mut events: Vec<LogEvent> = content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .filter_map(|line| match serde_json::from_str::<LogEvent>(line) {
                Ok(event) => Some(event),
                Err(e) => {
                    trace!("解析日志行失败: {} - line: {}", e, line);
                    None
                }
            })
            .collect();

        // 按时间排序
        events.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        // 找到第一条用户消息
        for event in events {
            if let Some(message) = Self::convert_to_message(&event) {
                if message.role == MessageRole::User {
                    return Ok(Some(message));
                }
            }
        }

        Ok(None)
    }

    /// 读取最后 N 条事件
    fn read_last_events(log_path: &Path, count: usize) -> Result<Vec<LogEvent>> {
        if !log_path.exists() {
            return Ok(Vec::new());
        }

        // 读取文件内容
        let content = std::fs::read_to_string(log_path)?;

        // 解析所有行
        let mut events: Vec<LogEvent> = content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .filter_map(|line| match serde_json::from_str::<LogEvent>(line) {
                Ok(event) => Some(event),
                Err(e) => {
                    trace!("解析日志行失败: {} - line: {}", e, line);
                    None
                }
            })
            .collect();

        // 按时间排序
        events.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        // 只保留最后 N 条
        if events.len() > count {
            events = events.split_off(events.len() - count);
        }

        Ok(events)
    }

    /// 分析最后一条事件推断状态
    fn infer_from_event(last_event: &LogEvent, context: &[LogEvent]) -> Result<SessionStatus> {
        match last_event.event_type.as_str() {
            "user" => {
                // 用户刚输入，Claude 正在处理
                debug!("最后事件是用户输入，推断状态为 Running");
                Ok(SessionStatus::Running)
            }
            "assistant" => {
                // 分析 assistant 的响应内容
                Self::analyze_assistant_response(last_event, context)
            }
            "queue-operation" | "file-history-snapshot" => {
                // 操作队列事件，通常表示正在执行
                debug!("最后事件是操作队列，推断状态为 Running");
                Ok(SessionStatus::Running)
            }
            "error" | "tool_error" => {
                // 错误事件
                debug!("最后事件是错误，推断状态为 Blocked");
                Ok(SessionStatus::Blocked)
            }
            "summary" => {
                // 摘要事件，通常表示一轮对话结束
                // 如果下一条没有用户输入，说明在等待输入
                Ok(SessionStatus::WaitingInput)
            }
            _ => {
                trace!("未知事件类型: {}", last_event.event_type);
                Ok(SessionStatus::Unknown)
            }
        }
    }

    /// 分析 assistant 响应
    fn analyze_assistant_response(
        event: &LogEvent,
        _context: &[LogEvent],
    ) -> Result<SessionStatus> {
        let content = match &event.content {
            Some(c) => c,
            None => match &event.message {
                Some(m) => &m.content,
                None => return Ok(SessionStatus::Unknown),
            },
        };

        // 检查是否包含工具调用
        let has_tool_use = content.iter().any(|block| {
            block.block_type == "tool_use" || block.tool_use.is_some()
        });

        if has_tool_use {
            debug!("Assistant 响应包含工具调用，推断状态为 Running");
            return Ok(SessionStatus::Running);
        }

        // 提取文本内容
        let text: String = content
            .iter()
            .filter_map(|block| {
                if block.block_type == "text" {
                    block.text.clone()
                } else {
                    None
                }
            })
            .collect();

        // 检查是否在等待用户确认
        if Self::is_waiting_for_input(&text) {
            debug!("Assistant 响应包含等待输入信号，推断状态为 WaitingInput");
            return Ok(SessionStatus::WaitingInput);
        }

        // 检查是否包含错误/阻塞信号
        if Self::is_blocked(&text) {
            debug!("Assistant 响应包含阻塞信号，推断状态为 Blocked");
            return Ok(SessionStatus::Blocked);
        }

        // 默认情况下，assistant 响应表示正在处理或已完成
        // 如果没有检测到问题，认为已完成或等待输入
        if text.ends_with('?') || text.contains("Would you like") {
            Ok(SessionStatus::WaitingInput)
        } else {
            Ok(SessionStatus::Running)
        }
    }

    /// 检测是否在等待用户输入
    fn is_waiting_for_input(text: &str) -> bool {
        let indicators = [
            "请确认",
            "请输入",
            "请选择",
            "是否继续",
            "确定要",
            "Would you like",
            "Please confirm",
            "Enter your",
            "Choose one",
            "Do you want",
            "yes/no",
            "Y/n",
            "继续吗",
            "确认吗",
            "?",
            "？",
        ];

        indicators.iter().any(|&indicator| text.contains(indicator))
    }

    /// 检测是否被阻塞
    fn is_blocked(text: &str) -> bool {
        let indicators = [
            "error",
            "错误",
            "permission denied",
            "权限拒绝",
            "timeout",
            "超时",
            "failed",
            "失败",
            "unable to",
            "无法",
            "cannot",
            "不能",
        ];

        indicators.iter().any(|&indicator| {
            text.to_lowercase().contains(indicator)
        })
    }

    /// 将日志事件转换为消息
    fn convert_to_message(event: &LogEvent) -> Option<Message> {
        let role = match event.event_type.as_str() {
            "user" => MessageRole::User,
            "assistant" => MessageRole::Assistant,
            _ => return None, // 只转换用户和助手消息
        };

        let content = match &event.content {
            Some(c) => Self::extract_text_content(c),
            None => match &event.message {
                Some(m) => Self::extract_text_content(&m.content),
                None => return None,
            },
        };

        if content.is_empty() {
            return None;
        }

        // 检查是否包含代码块
        let has_code = content.contains("```");

        Some(Message {
            id: event.uuid.clone().unwrap_or_default(),
            role,
            content,
            timestamp: event.timestamp,
            metadata: Some(MessageMetadata {
                has_code,
                token_count: None,
            }),
        })
    }

    /// 提取文本内容
    fn extract_text_content(content: &[ContentBlock]) -> String {
        content
            .iter()
            .filter_map(|block| {
                if block.block_type == "text" {
                    block.text.clone()
                } else if block.block_type == "thinking" {
                    block.thinking.clone()
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// 会话状态缓存
#[derive(Debug, Clone)]
pub struct StatusCache {
    cache: HashMap<String, (SessionStatus, DateTime<Utc>)>,
    ttl_seconds: i64,
}

impl StatusCache {
    /// 创建新的状态缓存
    pub fn new(ttl_seconds: i64) -> Self {
        Self {
            cache: HashMap::new(),
            ttl_seconds,
        }
    }

    /// 获取缓存的状态
    pub fn get(&self, session_id: &str) -> Option<SessionStatus> {
        self.cache.get(session_id).and_then(|(status, cached_at)| {
            let now = Utc::now();
            let elapsed = now.signed_duration_since(*cached_at);

            if elapsed.num_seconds() < self.ttl_seconds {
                Some(*status)
            } else {
                None
            }
        })
    }

    /// 缓存状态
    pub fn set(&mut self, session_id: String, status: SessionStatus) {
        self.cache.insert(session_id, (status, Utc::now()));
    }

    /// 使缓存失效
    pub fn invalidate(&mut self, session_id: &str) {
        self.cache.remove(session_id);
    }

    /// 清理过期缓存
    pub fn cleanup(&mut self) {
        let now = Utc::now();
        self.cache.retain(|_, (_, cached_at)| {
            let elapsed = now.signed_duration_since(*cached_at);
            elapsed.num_seconds() < self.ttl_seconds
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_waiting_for_input() {
        assert!(StatusDetector::is_waiting_for_input("Would you like to continue?"));
        assert!(StatusDetector::is_waiting_for_input("请确认是否继续"));
        assert!(!StatusDetector::is_waiting_for_input("我已经完成了任务"));
    }

    #[test]
    fn test_is_blocked() {
        assert!(StatusDetector::is_blocked("Error: permission denied"));
        assert!(StatusDetector::is_blocked("操作失败，无法访问"));
        assert!(!StatusDetector::is_blocked("任务执行成功"));
    }
}
