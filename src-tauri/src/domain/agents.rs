use serde::{Deserialize, Serialize};

use crate::domain::common::{
    ChannelBindingDetail, ChannelBindingInput, Id, ResourceBindingDetail, ResourceBindingInput,
    TimestampMs,
};
use crate::domain::content::{ContentWriteInput, StoredContent};
use crate::domain::messages::MessageRole;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSummary {
    pub id: Id,
    pub name: String,
    pub title: Option<String>,
    pub avatar_uri: Option<String>,
    pub enabled: bool,
    pub sort_order: i64,
    pub created_at: TimestampMs,
    pub updated_at: TimestampMs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentGreetingDetail {
    pub id: Id,
    pub agent_id: Id,
    pub greeting_type: String,
    pub name: Option<String>,
    pub primary_content: StoredContent,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: serde_json::Value,
    pub created_at: TimestampMs,
    pub updated_at: TimestampMs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMediaDetail {
    pub id: Id,
    pub agent_id: Id,
    pub media_type: String,
    pub name: Option<String>,
    pub content: StoredContent,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: serde_json::Value,
    pub created_at: TimestampMs,
    pub updated_at: TimestampMs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDetail {
    pub summary: AgentSummary,
    pub description_content: Option<StoredContent>,
    pub personality_content: Option<StoredContent>,
    pub scenario_content: Option<StoredContent>,
    pub example_messages_content: Option<StoredContent>,
    pub main_prompt_override_content: Option<StoredContent>,
    pub post_history_instructions_content: Option<StoredContent>,
    pub character_note_content: Option<StoredContent>,
    pub creator_notes_content: Option<StoredContent>,
    pub character_note_depth: Option<i64>,
    pub character_note_role: Option<MessageRole>,
    pub talkativeness: i64,
    pub creator_name: Option<String>,
    pub character_version: Option<String>,
    pub greetings: Vec<AgentGreetingDetail>,
    pub media: Vec<AgentMediaDetail>,
    pub preset_bindings: Vec<ResourceBindingDetail>,
    pub lorebook_bindings: Vec<ResourceBindingDetail>,
    pub user_profile_bindings: Vec<ResourceBindingDetail>,
    pub channel_bindings: Vec<ChannelBindingDetail>,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAgentInput {
    pub name: String,
    pub title: Option<String>,
    pub description_content: Option<ContentWriteInput>,
    pub personality_content: Option<ContentWriteInput>,
    pub scenario_content: Option<ContentWriteInput>,
    pub example_messages_content: Option<ContentWriteInput>,
    pub main_prompt_override_content: Option<ContentWriteInput>,
    pub post_history_instructions_content: Option<ContentWriteInput>,
    pub character_note_content: Option<ContentWriteInput>,
    pub creator_notes_content: Option<ContentWriteInput>,
    pub character_note_depth: Option<i64>,
    pub character_note_role: Option<MessageRole>,
    pub talkativeness: i64,
    pub avatar_uri: Option<String>,
    pub creator_name: Option<String>,
    pub character_version: Option<String>,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAgentInput {
    pub name: String,
    pub title: Option<String>,
    pub description_content: Option<ContentWriteInput>,
    pub personality_content: Option<ContentWriteInput>,
    pub scenario_content: Option<ContentWriteInput>,
    pub example_messages_content: Option<ContentWriteInput>,
    pub main_prompt_override_content: Option<ContentWriteInput>,
    pub post_history_instructions_content: Option<ContentWriteInput>,
    pub character_note_content: Option<ContentWriteInput>,
    pub creator_notes_content: Option<ContentWriteInput>,
    pub character_note_depth: Option<i64>,
    pub character_note_role: Option<MessageRole>,
    pub talkativeness: i64,
    pub avatar_uri: Option<String>,
    pub creator_name: Option<String>,
    pub character_version: Option<String>,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAgentGreetingInput {
    pub greeting_type: String,
    pub name: Option<String>,
    pub primary_content: ContentWriteInput,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAgentGreetingInput {
    pub greeting_type: String,
    pub name: Option<String>,
    pub primary_content: ContentWriteInput,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddAgentMediaInput {
    pub media_type: String,
    pub name: Option<String>,
    pub content: ContentWriteInput,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: serde_json::Value,
}

pub type AgentResourceBindingInput = ResourceBindingInput;
pub type AgentChannelBindingInput = ChannelBindingInput;
