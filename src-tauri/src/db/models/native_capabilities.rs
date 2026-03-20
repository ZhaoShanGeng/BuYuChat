use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ToolInvocationRow {
    pub id: String,
    pub generation_run_id: Option<String>,
    pub workflow_run_node_execution_id: Option<String>,
    pub message_version_id: Option<String>,
    pub tool_kind: String,
    pub tool_name: String,
    pub plugin_id: Option<String>,
    pub request_content_id: Option<String>,
    pub response_content_id: Option<String>,
    pub status: String,
    pub started_at: Option<i64>,
    pub finished_at: Option<i64>,
    pub created_at: i64,
    pub config_json: String,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct RagRefRow {
    pub id: String,
    pub generation_run_id: Option<String>,
    pub workflow_run_node_execution_id: Option<String>,
    pub source_uri: Option<String>,
    pub document_title: Option<String>,
    pub chunk_key: Option<String>,
    pub score: Option<f32>,
    pub excerpt_content_id: Option<String>,
    pub included_in_request: bool,
    pub config_json: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct McpEventRow {
    pub id: String,
    pub generation_run_id: Option<String>,
    pub workflow_run_node_execution_id: Option<String>,
    pub server_name: String,
    pub event_kind: String,
    pub method_name: Option<String>,
    pub payload_content_id: Option<String>,
    pub status: String,
    pub config_json: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct VariableDefRow {
    pub id: String,
    pub var_key: String,
    pub name: String,
    pub value_type: String,
    pub scope_type: String,
    pub namespace: String,
    pub is_user_editable: bool,
    pub is_plugin_editable: bool,
    pub ai_can_create: bool,
    pub ai_can_update: bool,
    pub ai_can_delete: bool,
    pub ai_can_lock: bool,
    pub ai_can_unlock_own_lock: bool,
    pub ai_can_unlock_any_lock: bool,
    pub default_json: String,
    pub config_json: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct VariableValueRow {
    pub id: String,
    pub variable_def_id: String,
    pub scope_type: String,
    pub scope_id: String,
    pub value_json: String,
    pub value_content_id: Option<String>,
    pub source_message_version_id: Option<String>,
    pub updated_by_kind: String,
    pub updated_by_ref_id: Option<String>,
    pub event_no: i64,
    pub is_deleted: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct VariableEventRow {
    pub id: String,
    pub variable_value_id: String,
    pub event_no: i64,
    pub event_kind: String,
    pub value_json: String,
    pub value_content_id: Option<String>,
    pub source_message_version_id: Option<String>,
    pub updated_by_kind: String,
    pub updated_by_ref_id: Option<String>,
    pub created_at: i64,
    pub config_json: String,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct VariableLockRow {
    pub id: String,
    pub variable_value_id: String,
    pub lock_kind: String,
    pub owner_kind: String,
    pub owner_ref_id: Option<String>,
    pub unlock_policy: String,
    pub active: bool,
    pub config_json: String,
    pub created_at: i64,
    pub updated_at: i64,
}
