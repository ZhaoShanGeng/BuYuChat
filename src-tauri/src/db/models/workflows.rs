use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct WorkflowDefRow {
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
pub struct WorkflowDefNodeRow {
    pub id: String,
    pub workflow_def_id: String,
    pub node_key: String,
    pub name: Option<String>,
    pub node_type: String,
    pub agent_id: Option<String>,
    pub plugin_id: Option<String>,
    pub preset_id: Option<String>,
    pub lorebook_id: Option<String>,
    pub user_profile_id: Option<String>,
    pub api_channel_id: Option<String>,
    pub api_channel_model_id: Option<String>,
    pub workspace_mode: String,
    pub history_read_mode: String,
    pub summary_write_mode: String,
    pub message_write_mode: String,
    pub visible_output_mode: String,
    pub config_json: String,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct WorkflowDefEdgeRow {
    pub id: String,
    pub workflow_def_id: String,
    pub from_node_id: String,
    pub to_node_id: String,
    pub edge_type: String,
    pub priority: i64,
    pub condition_expr: Option<String>,
    pub label: Option<String>,
    pub enabled: bool,
    pub config_json: String,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct WorkflowRunRow {
    pub id: String,
    pub workflow_def_id: String,
    pub trigger_conversation_id: Option<String>,
    pub workspace_conversation_id: Option<String>,
    pub workspace_mode: String,
    pub trigger_message_version_id: Option<String>,
    pub entry_node_id: Option<String>,
    pub status: String,
    pub result_message_version_id: Option<String>,
    pub request_snapshot_content_id: Option<String>,
    pub result_content_id: Option<String>,
    pub config_json: String,
    pub started_at: Option<i64>,
    pub finished_at: Option<i64>,
    pub created_at: i64,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct WorkflowRunNodeExecutionRow {
    pub id: String,
    pub workflow_run_id: String,
    pub workflow_def_node_id: String,
    pub parent_execution_id: Option<String>,
    pub incoming_edge_id: Option<String>,
    pub branch_key: Option<String>,
    pub loop_iteration: i64,
    pub retry_index: i64,
    pub status: String,
    pub generation_run_id: Option<String>,
    pub input_snapshot_content_id: Option<String>,
    pub output_content_id: Option<String>,
    pub error_content_id: Option<String>,
    pub started_at: Option<i64>,
    pub finished_at: Option<i64>,
    pub created_at: i64,
    pub config_json: String,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct WorkflowRunWriteRow {
    pub id: String,
    pub workflow_run_id: String,
    pub workflow_run_node_execution_id: Option<String>,
    pub write_kind: String,
    pub apply_mode: String,
    pub content_id: String,
    pub target_conversation_id: Option<String>,
    pub target_message_node_id: Option<String>,
    pub target_summary_group_id: Option<String>,
    pub target_lorebook_entry_id: Option<String>,
    pub target_preset_entry_id: Option<String>,
    pub target_agent_id: Option<String>,
    pub target_user_profile_id: Option<String>,
    pub target_plugin_id: Option<String>,
    pub target_slot: Option<String>,
    pub visible_to_user: bool,
    pub created_at: i64,
    pub config_json: String,
}
