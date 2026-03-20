use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PresetRow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PresetEntryRow {
    pub id: String,
    pub preset_id: String,
    pub name: Option<String>,
    pub role: String,
    pub primary_content_id: String,
    pub position_type: String,
    pub list_order: i64,
    pub depth: Option<i64>,
    pub depth_order: i64,
    pub triggers_json: String,
    pub enabled: bool,
    pub is_pinned: bool,
    pub config_json: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PresetChannelBindingRow {
    pub id: String,
    pub preset_id: String,
    pub channel_id: String,
    pub channel_model_id: Option<String>,
    pub binding_type: String,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: String,
    pub created_at: i64,
    pub updated_at: i64,
}
