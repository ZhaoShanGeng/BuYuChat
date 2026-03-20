use serde::{Deserialize, Serialize};

use crate::domain::common::{Id, TimestampMs};
use crate::domain::content::{ContentWriteInput, StoredContent};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInvocationDetail {
    pub id: Id,
    pub generation_run_id: Option<Id>,
    pub workflow_run_node_execution_id: Option<Id>,
    pub message_version_id: Option<Id>,
    pub tool_kind: String,
    pub tool_name: String,
    pub plugin_id: Option<Id>,
    pub request_content: Option<StoredContent>,
    pub response_content: Option<StoredContent>,
    pub status: String,
    pub started_at: Option<TimestampMs>,
    pub finished_at: Option<TimestampMs>,
    pub created_at: TimestampMs,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartToolInvocationInput {
    pub generation_run_id: Option<Id>,
    pub workflow_run_node_execution_id: Option<Id>,
    pub message_version_id: Option<Id>,
    pub tool_kind: String,
    pub tool_name: String,
    pub plugin_id: Option<Id>,
    pub request_content: Option<ContentWriteInput>,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinishToolInvocationInput {
    pub status: String,
    pub response_content: Option<ContentWriteInput>,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagRefDetail {
    pub id: Id,
    pub generation_run_id: Option<Id>,
    pub workflow_run_node_execution_id: Option<Id>,
    pub source_uri: Option<String>,
    pub document_title: Option<String>,
    pub chunk_key: Option<String>,
    pub score: Option<f32>,
    pub excerpt_content: Option<StoredContent>,
    pub included_in_request: bool,
    pub created_at: TimestampMs,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRagRefInput {
    pub generation_run_id: Option<Id>,
    pub workflow_run_node_execution_id: Option<Id>,
    pub source_uri: Option<String>,
    pub document_title: Option<String>,
    pub chunk_key: Option<String>,
    pub score: Option<f32>,
    pub excerpt_content: Option<ContentWriteInput>,
    pub included_in_request: bool,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpEventDetail {
    pub id: Id,
    pub generation_run_id: Option<Id>,
    pub workflow_run_node_execution_id: Option<Id>,
    pub server_name: String,
    pub event_kind: String,
    pub method_name: Option<String>,
    pub payload_content: Option<StoredContent>,
    pub status: String,
    pub created_at: TimestampMs,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMcpEventInput {
    pub generation_run_id: Option<Id>,
    pub workflow_run_node_execution_id: Option<Id>,
    pub server_name: String,
    pub event_kind: String,
    pub method_name: Option<String>,
    pub payload_content: Option<ContentWriteInput>,
    pub status: String,
    pub config_json: serde_json::Value,
}
