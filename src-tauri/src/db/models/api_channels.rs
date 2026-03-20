use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ApiChannelRow {
    pub id: String,
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
    pub config_json: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ApiChannelModelRow {
    pub id: String,
    pub channel_id: String,
    pub model_id: String,
    pub display_name: Option<String>,
    pub model_type: Option<String>,
    pub context_window: Option<i64>,
    pub max_output_tokens: Option<i64>,
    pub capabilities_json: String,
    pub pricing_json: String,
    pub default_parameters_json: String,
    pub sort_order: i64,
    pub config_json: String,
}
