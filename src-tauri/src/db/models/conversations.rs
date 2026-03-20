use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ConversationRow {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub conversation_mode: String,
    pub archived: bool,
    pub pinned: bool,
    pub config_json: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ConversationParticipantRow {
    pub id: String,
    pub conversation_id: String,
    pub agent_id: Option<String>,
    pub display_name: Option<String>,
    pub participant_type: String,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ConversationResourceBindingRow {
    pub id: String,
    pub conversation_id: String,
    pub resource_id: String,
    pub binding_type: String,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ConversationChannelBindingRow {
    pub id: String,
    pub conversation_id: String,
    pub channel_id: String,
    pub channel_model_id: Option<String>,
    pub binding_type: String,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: String,
    pub created_at: i64,
    pub updated_at: i64,
}
