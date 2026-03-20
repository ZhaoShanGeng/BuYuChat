use serde::{Deserialize, Serialize};

use crate::domain::common::{ChannelBindingDetail, ChannelBindingInput, Id, TimestampMs};
use crate::domain::content::{ContentWriteInput, StoredContent};
use crate::domain::messages::MessageRole;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetSummary {
    pub id: Id,
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: serde_json::Value,
    pub created_at: TimestampMs,
    pub updated_at: TimestampMs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetEntryDetail {
    pub id: Id,
    pub preset_id: Id,
    pub name: String,
    pub role: MessageRole,
    pub primary_content: StoredContent,
    pub position_type: String,
    pub list_order: i64,
    pub depth: Option<i64>,
    pub depth_order: i64,
    pub triggers_json: serde_json::Value,
    pub enabled: bool,
    pub is_pinned: bool,
    pub config_json: serde_json::Value,
    pub created_at: TimestampMs,
    pub updated_at: TimestampMs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetDetail {
    pub preset: PresetSummary,
    pub entries: Vec<PresetEntryDetail>,
    pub channel_bindings: Vec<ChannelBindingDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePresetInput {
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePresetInput {
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePresetEntryInput {
    pub preset_id: Id,
    pub name: String,
    pub role: MessageRole,
    pub primary_content: ContentWriteInput,
    pub position_type: String,
    pub list_order: i64,
    pub depth: Option<i64>,
    pub depth_order: i64,
    pub triggers_json: serde_json::Value,
    pub enabled: bool,
    pub is_pinned: bool,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePresetEntryInput {
    pub name: String,
    pub role: MessageRole,
    pub primary_content: ContentWriteInput,
    pub position_type: String,
    pub list_order: i64,
    pub depth: Option<i64>,
    pub depth_order: i64,
    pub triggers_json: serde_json::Value,
    pub enabled: bool,
    pub is_pinned: bool,
    pub config_json: serde_json::Value,
}

pub type PresetChannelBindingInput = ChannelBindingInput;
