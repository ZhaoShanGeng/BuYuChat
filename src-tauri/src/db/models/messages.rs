use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct MessageNodeRow {
    pub id: String,
    pub conversation_id: String,
    pub author_participant_id: String,
    pub role: String,
    pub reply_to_node_id: Option<String>,
    pub order_key: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct MessageVersionRow {
    pub id: String,
    pub node_id: String,
    pub version_index: i64,
    pub is_active: bool,
    pub primary_content_id: String,
    pub context_policy: String,
    pub viewer_policy: String,
    pub api_channel_id: Option<String>,
    pub api_channel_model_id: Option<String>,
    pub prompt_tokens: Option<i64>,
    pub completion_tokens: Option<i64>,
    pub total_tokens: Option<i64>,
    pub finish_reason: Option<String>,
    pub generation_run_id: Option<String>,
    pub config_json: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct MessageVersionContentRefRow {
    pub id: String,
    pub message_version_id: String,
    pub content_id: String,
    pub plugin_id: Option<String>,
    pub ref_role: String,
    pub sort_order: i64,
    pub config_json: String,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct GenerationRunRow {
    pub id: String,
    pub conversation_id: String,
    pub trigger_node_id: Option<String>,
    pub trigger_message_version_id: Option<String>,
    pub responder_participant_id: Option<String>,
    pub api_channel_id: Option<String>,
    pub api_channel_model_id: Option<String>,
    pub preset_id: Option<String>,
    pub preset_source_scope: Option<String>,
    pub lorebook_id: Option<String>,
    pub lorebook_source_scope: Option<String>,
    pub user_profile_id: Option<String>,
    pub user_profile_source_scope: Option<String>,
    pub api_channel_source_scope: Option<String>,
    pub api_channel_model_source_scope: Option<String>,
    pub run_type: String,
    pub request_parameters_json: String,
    pub request_payload_content_id: Option<String>,
    pub response_payload_content_id: Option<String>,
    pub status: String,
    pub error_text: Option<String>,
    pub started_at: Option<i64>,
    pub created_at: i64,
    pub finished_at: Option<i64>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct GenerationRunContextItemRow {
    pub id: String,
    pub generation_run_id: String,
    pub sequence_no: i64,
    pub send_role: String,
    pub rendered_content_id: String,
    pub source_kind: String,
    pub source_message_node_id: Option<String>,
    pub source_message_version_id: Option<String>,
    pub source_summary_version_id: Option<String>,
    pub source_preset_entry_id: Option<String>,
    pub source_lorebook_entry_id: Option<String>,
    pub source_user_profile_id: Option<String>,
    pub source_agent_id: Option<String>,
    pub source_agent_greeting_id: Option<String>,
    pub source_tool_invocation_id: Option<String>,
    pub source_rag_ref_id: Option<String>,
    pub source_mcp_event_id: Option<String>,
    pub source_plugin_id: Option<String>,
    pub included_in_request: bool,
    pub config_json: String,
}
