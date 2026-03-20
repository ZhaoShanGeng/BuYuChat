pub type Id = String;
pub type TimestampMs = i64;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelBindingInput {
    pub channel_id: Id,
    pub channel_model_id: Option<Id>,
    pub binding_type: String,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelBindingDetail {
    pub id: Id,
    pub channel_id: Id,
    pub channel_model_id: Option<Id>,
    pub binding_type: String,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: serde_json::Value,
    pub created_at: TimestampMs,
    pub updated_at: TimestampMs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceBindingInput {
    pub resource_id: Id,
    pub binding_type: String,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceBindingDetail {
    pub id: Id,
    pub resource_id: Id,
    pub binding_type: String,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: serde_json::Value,
    pub created_at: TimestampMs,
    pub updated_at: TimestampMs,
}
