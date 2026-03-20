use serde::{Deserialize, Serialize};

use crate::domain::common::{
    ChannelBindingDetail, ChannelBindingInput, Id, ResourceBindingDetail, ResourceBindingInput,
    TimestampMs,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationParticipantInput {
    pub agent_id: Option<Id>,
    pub display_name: Option<String>,
    pub participant_type: String,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationParticipantDetail {
    pub id: Id,
    pub conversation_id: Id,
    pub agent_id: Option<Id>,
    pub display_name: Option<String>,
    pub participant_type: String,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: serde_json::Value,
    pub created_at: TimestampMs,
    pub updated_at: TimestampMs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSummary {
    pub id: Id,
    pub title: String,
    pub description: Option<String>,
    pub conversation_mode: String,
    pub archived: bool,
    pub pinned: bool,
    pub config_json: serde_json::Value,
    pub created_at: TimestampMs,
    pub updated_at: TimestampMs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationDetail {
    pub summary: ConversationSummary,
    pub participants: Vec<ConversationParticipantDetail>,
    pub preset_bindings: Vec<ResourceBindingDetail>,
    pub lorebook_bindings: Vec<ResourceBindingDetail>,
    pub user_profile_bindings: Vec<ResourceBindingDetail>,
    pub channel_bindings: Vec<ChannelBindingDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateConversationInput {
    pub title: String,
    pub description: Option<String>,
    pub conversation_mode: String,
    pub archived: bool,
    pub pinned: bool,
    pub participants: Vec<ConversationParticipantInput>,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConversationMetaInput {
    pub title: String,
    pub description: Option<String>,
    pub archived: bool,
    pub pinned: bool,
    pub config_json: serde_json::Value,
}

pub type ConversationResourceBindingInput = ResourceBindingInput;
pub type ConversationChannelBindingInput = ChannelBindingInput;
