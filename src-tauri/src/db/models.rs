use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct ConversationRow {
    pub id: String,
    pub title: String,
    pub model_id: String,
    pub provider: String,
    pub assistant_id: Option<String>,
    pub system_prompt: Option<String>,
    pub pinned: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Default, FromRow, serde::Serialize)]
pub struct MessageRow {
    pub id: String,
    pub conversation_id: String,
    pub parent_message_id: Option<String>,
    pub version_group_id: String,
    pub version_index: i64,
    pub is_active: bool,
    pub role: String,
    pub content: Option<String>,
    pub content_parts: Option<String>,
    pub tool_calls: Option<String>,
    pub tool_call_id: Option<String>,
    pub citations_json: Option<String>,
    pub tokens_used: Option<i64>,
    pub provider: Option<String>,
    pub model_id: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct TurnRow {
    pub id: String,
    pub conversation_id: String,
    pub parent_turn_id: Option<String>,
    pub role: String,
    pub active_version_id: Option<String>,
    pub deleted_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct TurnVersionRow {
    pub id: String,
    pub turn_id: String,
    pub version_index: i64,
    pub content: Option<String>,
    pub content_parts: Option<String>,
    pub tool_calls: Option<String>,
    pub tool_call_id: Option<String>,
    pub citations_json: Option<String>,
    pub tokens_used: Option<i64>,
    pub provider: Option<String>,
    pub model_id: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct ProviderConfigRow {
    pub id: String,
    pub provider: String,
    pub api_key_id: Option<String>,
    pub base_url: Option<String>,
    pub extra_json: Option<String>,
    pub enabled: bool,
    pub updated_at: i64,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct CustomChannelRow {
    pub id: String,
    pub name: String,
    pub channel_type: String,
    pub base_url: String,
    pub auth_json: String,
    pub endpoints_json: String,
    pub stream_protocol: String,
    pub request_template_json: String,
    pub response_mapping_json: String,
    pub stream_mapping_json: String,
    pub models_json: String,
    pub enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
}
