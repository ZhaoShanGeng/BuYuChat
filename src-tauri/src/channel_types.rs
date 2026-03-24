use crate::errors::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChannelTypeConfig {
    pub channel_type: &'static str,
    pub auth_type: &'static str,
    pub models_endpoint: &'static str,
    pub chat_endpoint: &'static str,
    pub stream_endpoint: &'static str,
}

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
