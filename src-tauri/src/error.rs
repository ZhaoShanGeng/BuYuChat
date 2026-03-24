//! Tauri IPC 与服务层共用的应用错误定义。

use serde::Serialize;
use thiserror::Error;

/// 通过 Tauri IPC 返回给前端的标准错误载荷。
#[derive(Debug, Clone, Error, Serialize, PartialEq, Eq)]
#[error("{message}")]
pub struct AppError {
    /// 供前端做本地化映射的机器可读错误码。
    pub error_code: String,
    /// 供日志与排障使用的调试消息。
    pub message: String,
}

impl AppError {
    /// 创建校验失败错误。
    pub fn validation(error_code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error_code: error_code.into(),
            message: message.into(),
        }
    }

    /// 创建资源不存在错误。
    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            error_code: "NOT_FOUND".to_string(),
            message: message.into(),
        }
    }

    /// 创建资源冲突错误。
    pub fn conflict(error_code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error_code: error_code.into(),
            message: message.into(),
        }
    }

    /// 创建渠道连通性错误。
    pub fn channel_unreachable(message: impl Into<String>) -> Self {
        Self {
            error_code: "CHANNEL_UNREACHABLE".to_string(),
            message: message.into(),
        }
    }

    /// 创建 AI 上游返回错误。
    pub fn ai_request_failed(message: impl Into<String>) -> Self {
        Self {
            error_code: "AI_REQUEST_FAILED".to_string(),
            message: message.into(),
        }
    }

    /// 创建内部错误。
    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            error_code: "INTERNAL_ERROR".to_string(),
            message: message.into(),
        }
    }

    /// 创建“未配置 Agent”错误。
    pub fn no_agent() -> Self {
        Self {
            error_code: "NO_AGENT".to_string(),
            message: "conversation has no agent configured".to_string(),
        }
    }

    /// 创建“Agent 已禁用”错误。
    pub fn agent_disabled() -> Self {
        Self {
            error_code: "AGENT_DISABLED".to_string(),
            message: "conversation agent is disabled".to_string(),
        }
    }

    /// 创建“未配置渠道”错误。
    pub fn no_channel() -> Self {
        Self {
            error_code: "NO_CHANNEL".to_string(),
            message: "conversation has no channel configured".to_string(),
        }
    }

    /// 创建“渠道已禁用”错误。
    pub fn channel_disabled() -> Self {
        Self {
            error_code: "CHANNEL_DISABLED".to_string(),
            message: "conversation channel is disabled".to_string(),
        }
    }

    /// 创建“未配置模型”错误。
    pub fn no_model() -> Self {
        Self {
            error_code: "NO_MODEL".to_string(),
            message: "conversation has no model configured".to_string(),
        }
    }

    /// 创建“不是最后一个 user 楼层”错误。
    pub fn not_last_user_node() -> Self {
        Self {
            error_code: "NOT_LAST_USER_NODE".to_string(),
            message: "user node is not the last node in conversation".to_string(),
        }
    }

    /// 创建“版本不属于指定楼层”错误。
    pub fn version_not_in_node() -> Self {
        Self {
            error_code: "VERSION_NOT_IN_NODE".to_string(),
            message: "version does not belong to the specified node".to_string(),
        }
    }
}
