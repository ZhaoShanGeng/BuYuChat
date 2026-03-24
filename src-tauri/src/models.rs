use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub channel_type: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub auth_type: Option<String>,
    pub models_endpoint: Option<String>,
    pub chat_endpoint: Option<String>,
    pub stream_endpoint: Option<String>,
    pub enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChannelTestResult {
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreateChannelInput {
    pub name: String,
    pub base_url: String,
    pub channel_type: Option<String>,
    pub api_key: Option<String>,
    pub auth_type: Option<String>,
    pub models_endpoint: Option<String>,
    pub chat_endpoint: Option<String>,
    pub stream_endpoint: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct UpdateChannelInput {
    pub name: Option<String>,
    pub base_url: Option<String>,
    pub channel_type: Option<String>,
    pub api_key: Option<String>,
    pub auth_type: Option<String>,
    pub models_endpoint: Option<String>,
    pub chat_endpoint: Option<String>,
    pub stream_endpoint: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewChannel {
    pub id: String,
    pub name: String,
    pub channel_type: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub auth_type: Option<String>,
    pub models_endpoint: Option<String>,
    pub chat_endpoint: Option<String>,
    pub stream_endpoint: Option<String>,
    pub enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelPatch {
    pub name: Option<String>,
    pub base_url: Option<String>,
    pub channel_type: Option<String>,
    pub api_key: Option<String>,
    pub auth_type: Option<String>,
    pub models_endpoint: Option<String>,
    pub chat_endpoint: Option<String>,
    pub stream_endpoint: Option<String>,
    pub enabled: Option<bool>,
    pub updated_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestChannelRequest {
    pub url: String,
    pub auth_header: Option<(String, String)>,
}
