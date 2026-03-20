use serde::{Deserialize, Serialize};

use crate::domain::common::{Id, TimestampMs};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefSummary {
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
pub struct WorkflowNodeInput {
    pub node_key: String,
    pub name: Option<String>,
    pub node_type: String,
    pub agent_id: Option<Id>,
    pub plugin_id: Option<Id>,
    pub preset_id: Option<Id>,
    pub lorebook_id: Option<Id>,
    pub user_profile_id: Option<Id>,
    pub api_channel_id: Option<Id>,
    pub api_channel_model_id: Option<Id>,
    pub workspace_mode: String,
    pub history_read_mode: String,
    pub summary_write_mode: String,
    pub message_write_mode: String,
    pub visible_output_mode: String,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNodeDetail {
    pub id: Id,
    pub workflow_def_id: Id,
    pub node_key: String,
    pub name: Option<String>,
    pub node_type: String,
    pub agent_id: Option<Id>,
    pub plugin_id: Option<Id>,
    pub preset_id: Option<Id>,
    pub lorebook_id: Option<Id>,
    pub user_profile_id: Option<Id>,
    pub api_channel_id: Option<Id>,
    pub api_channel_model_id: Option<Id>,
    pub workspace_mode: String,
    pub history_read_mode: String,
    pub summary_write_mode: String,
    pub message_write_mode: String,
    pub visible_output_mode: String,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEdgeInput {
    pub from_node_id: Id,
    pub to_node_id: Id,
    pub edge_type: String,
    pub priority: i64,
    pub condition_expr: Option<String>,
    pub label: Option<String>,
    pub enabled: bool,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEdgeDetail {
    pub id: Id,
    pub workflow_def_id: Id,
    pub from_node_id: Id,
    pub to_node_id: Id,
    pub edge_type: String,
    pub priority: i64,
    pub condition_expr: Option<String>,
    pub label: Option<String>,
    pub enabled: bool,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefDetail {
    pub summary: WorkflowDefSummary,
    pub nodes: Vec<WorkflowNodeDetail>,
    pub edges: Vec<WorkflowEdgeDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorkflowDefInput {
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWorkflowDefInput {
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunWorkflowInput {
    pub workflow_def_id: Id,
    pub conversation_id: Option<Id>,
    pub trigger_message_version_id: Option<Id>,
    pub responder_participant_id: Option<Id>,
    pub isolated_conversation_title: Option<String>,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRunResult {
    pub workflow_run_id: Id,
    pub status: String,
    pub entry_node_id: Option<Id>,
    pub workspace_conversation_id: Option<Id>,
    pub result_message_version_id: Option<Id>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNodeExecutionResult {
    pub execution_id: Id,
    pub workflow_run_id: Id,
    pub workflow_def_node_id: Id,
    pub status: String,
    pub output_content_id: Option<Id>,
}
