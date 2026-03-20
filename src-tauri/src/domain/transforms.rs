use serde::{Deserialize, Serialize};

use crate::domain::common::{Id, TimestampMs};
use crate::domain::content::StoredContent;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransformStage {
    Viewer,
    Request,
    File,
}

impl TransformStage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Viewer => "viewer",
            Self::Request => "request",
            Self::File => "file",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformPipeline {
    pub id: Id,
    pub name: String,
    pub pipeline_key: String,
    pub pipeline_kind: String,
    pub description_content: Option<StoredContent>,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: serde_json::Value,
    pub created_at: TimestampMs,
    pub updated_at: TimestampMs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformStep {
    pub id: Id,
    pub pipeline_id: Id,
    pub step_order: i64,
    pub step_type: String,
    pub pattern: Option<String>,
    pub replacement_template: Option<String>,
    pub regex_flags: String,
    pub max_replacements: Option<i64>,
    pub stop_on_match: bool,
    pub child_pipeline_id: Option<Id>,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTransformPipelineInput {
    pub name: String,
    pub pipeline_key: String,
    pub pipeline_kind: String,
    pub description_content_id: Option<Id>,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTransformPipelineInput {
    pub name: String,
    pub pipeline_kind: String,
    pub description_content_id: Option<Id>,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformStepInput {
    pub step_order: i64,
    pub step_type: String,
    pub pattern: Option<String>,
    pub replacement_template: Option<String>,
    pub regex_flags: String,
    pub max_replacements: Option<i64>,
    pub stop_on_match: bool,
    pub child_pipeline_id: Option<Id>,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformBinding {
    pub id: Id,
    pub pipeline_id: Id,
    pub conversation_id: Option<Id>,
    pub agent_id: Option<Id>,
    pub preset_id: Option<Id>,
    pub workflow_def_node_id: Option<Id>,
    pub apply_viewer: bool,
    pub apply_request: bool,
    pub apply_file: bool,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: serde_json::Value,
    pub created_at: TimestampMs,
    pub updated_at: TimestampMs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformBindingInput {
    pub pipeline_id: Id,
    pub conversation_id: Option<Id>,
    pub agent_id: Option<Id>,
    pub preset_id: Option<Id>,
    pub workflow_def_node_id: Option<Id>,
    pub apply_viewer: bool,
    pub apply_request: bool,
    pub apply_file: bool,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyTransformsInput {
    pub stage: TransformStage,
    pub conversation_id: Option<Id>,
    pub agent_id: Option<Id>,
    pub preset_id: Option<Id>,
    pub workflow_def_node_id: Option<Id>,
    pub source_content: StoredContent,
    pub generation_run_id: Option<Id>,
    pub workflow_run_id: Option<Id>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyTransformsResult {
    pub content: StoredContent,
    pub applied_pipeline_ids: Vec<Id>,
    pub changed: bool,
}
