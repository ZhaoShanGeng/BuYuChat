use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Error, Serialize, PartialEq, Eq)]
#[error("{message}")]
pub struct AppError {
    pub error_code: String,
    pub message: String,
}

impl AppError {
    pub fn validation(error_code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error_code: error_code.into(),
            message: message.into(),
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            error_code: "NOT_FOUND".to_string(),
            message: message.into(),
        }
    }

    pub fn channel_unreachable(message: impl Into<String>) -> Self {
        Self {
            error_code: "CHANNEL_UNREACHABLE".to_string(),
            message: message.into(),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            error_code: "INTERNAL_ERROR".to_string(),
            message: message.into(),
        }
    }
}
