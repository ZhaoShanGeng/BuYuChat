//! 渠道类型默认配置。

use crate::error::AppError;

/// 单个渠道类型对应的默认配置。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChannelTypeConfig {
    /// 渠道类型标识。
    pub channel_type: &'static str,
    /// 默认鉴权类型。
    pub auth_type: &'static str,
    /// 默认模型列表接口。
    pub models_endpoint: &'static str,
    /// 默认聊天接口。
    pub chat_endpoint: &'static str,
    /// 默认流式接口。
    pub stream_endpoint: &'static str,
}

/// 返回指定渠道类型的默认配置。
pub fn config_for(channel_type: &str) -> Result<ChannelTypeConfig, AppError> {
    match channel_type {
        "openai_compatible" => Ok(ChannelTypeConfig {
            channel_type: "openai_compatible",
            auth_type: "bearer",
            models_endpoint: "/v1/models",
            chat_endpoint: "/v1/chat/completions",
            stream_endpoint: "/v1/chat/completions",
        }),
        other => Err(AppError::validation(
            "VALIDATION_ERROR",
            format!("unsupported channel_type '{other}'"),
        )),
    }
}
