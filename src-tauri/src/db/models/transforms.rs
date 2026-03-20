use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct TransformPipelineRow {
    pub id: String,
    pub name: String,
    pub pipeline_key: String,
    pub pipeline_kind: String,
    pub description_content_id: Option<String>,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct TransformStepRow {
    pub id: String,
    pub pipeline_id: String,
    pub step_order: i64,
    pub step_type: String,
    pub pattern: Option<String>,
    pub replacement_template: Option<String>,
    pub regex_flags: String,
    pub max_replacements: Option<i64>,
    pub stop_on_match: bool,
    pub child_pipeline_id: Option<String>,
    pub config_json: String,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct TransformBindingRow {
    pub id: String,
    pub pipeline_id: String,
    pub conversation_id: Option<String>,
    pub agent_id: Option<String>,
    pub preset_id: Option<String>,
    pub workflow_def_node_id: Option<String>,
    pub apply_viewer: bool,
    pub apply_request: bool,
    pub apply_file: bool,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: String,
    pub created_at: i64,
    pub updated_at: i64,
}
