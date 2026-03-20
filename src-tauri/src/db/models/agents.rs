use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AgentRow {
    pub id: String,
    pub name: String,
    pub title: Option<String>,
    pub description_content_id: Option<String>,
    pub personality_content_id: Option<String>,
    pub scenario_content_id: Option<String>,
    pub example_messages_content_id: Option<String>,
    pub main_prompt_override_content_id: Option<String>,
    pub post_history_instructions_content_id: Option<String>,
    pub character_note_content_id: Option<String>,
    pub creator_notes_content_id: Option<String>,
    pub character_note_depth: Option<i64>,
    pub character_note_role: Option<String>,
    pub talkativeness: i64,
    pub avatar_uri: Option<String>,
    pub creator_name: Option<String>,
    pub character_version: Option<String>,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AgentGreetingRow {
    pub id: String,
    pub agent_id: String,
    pub greeting_type: String,
    pub name: Option<String>,
    pub primary_content_id: String,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AgentMediaRow {
    pub id: String,
    pub agent_id: String,
    pub media_type: String,
    pub content_id: String,
    pub name: Option<String>,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AgentResourceBindingRow {
    pub id: String,
    pub agent_id: String,
    pub resource_id: String,
    pub binding_type: String,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AgentChannelBindingRow {
    pub id: String,
    pub agent_id: String,
    pub channel_id: String,
    pub channel_model_id: Option<String>,
    pub binding_type: String,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: String,
    pub created_at: i64,
    pub updated_at: i64,
}
