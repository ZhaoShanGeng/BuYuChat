use serde::{Deserialize, Serialize};

use crate::domain::common::{Id, TimestampMs};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiChannel {
    pub id: Id,
    pub name: String,
    pub channel_type: String,
    pub base_url: String,
    pub auth_type: String,
    pub api_key: Option<String>,
    pub models_endpoint: Option<String>,
    pub chat_endpoint: Option<String>,
    pub stream_endpoint: Option<String>,
    pub models_mode: String,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: serde_json::Value,
    pub created_at: TimestampMs,
    pub updated_at: TimestampMs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiChannelModel {
    pub id: Id,
    pub channel_id: Id,
    pub model_id: String,
    pub display_name: Option<String>,
    pub model_type: Option<String>,
    pub context_window: Option<i64>,
    pub max_output_tokens: Option<i64>,
    pub capabilities_json: serde_json::Value,
    pub pricing_json: serde_json::Value,
    pub default_parameters_json: serde_json::Value,
    pub sort_order: i64,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiChannelInput {
    pub name: String,
    pub channel_type: String,
    pub base_url: String,
    pub auth_type: String,
    pub api_key: Option<String>,
    pub models_endpoint: Option<String>,
    pub chat_endpoint: Option<String>,
    pub stream_endpoint: Option<String>,
    pub models_mode: String,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateApiChannelInput {
    pub name: String,
    pub channel_type: String,
    pub base_url: String,
    pub auth_type: String,
    pub api_key: Option<String>,
    pub models_endpoint: Option<String>,
    pub chat_endpoint: Option<String>,
    pub stream_endpoint: Option<String>,
    pub models_mode: String,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertApiChannelModelInput {
    pub channel_id: Id,
    pub model_id: String,
    pub display_name: Option<String>,
    pub model_type: Option<String>,
    pub context_window: Option<i64>,
    pub max_output_tokens: Option<i64>,
    pub capabilities_json: serde_json::Value,
    pub pricing_json: serde_json::Value,
    pub default_parameters_json: serde_json::Value,
    pub sort_order: i64,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiChannelTestResponse {
    pub model_id: String,
    pub response_text: String,
}
