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

    /// 创建渠道连通性错误。
    pub fn channel_unreachable(message: impl Into<String>) -> Self {
        Self {
            error_code: "CHANNEL_UNREACHABLE".to_string(),
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
}
