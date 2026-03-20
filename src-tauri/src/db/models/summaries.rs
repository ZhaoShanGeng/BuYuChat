use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SummaryGroupRow {
    pub id: String,
    pub conversation_id: String,
    pub scope_type: String,
    pub scope_message_version_id: Option<String>,
    pub scope_start_node_id: Option<String>,
    pub scope_end_node_id: Option<String>,
    pub scope_summary_group_id: Option<String>,
    pub summary_kind: String,
    pub default_generator_preset_id: Option<String>,
    pub enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SummaryVersionRow {
    pub id: String,
    pub summary_group_id: String,
    pub version_index: i64,
    pub is_active: bool,
    pub content_id: String,
    pub generator_type: String,
    pub generator_preset_id: Option<String>,
    pub workflow_run_id: Option<String>,
    pub generation_run_id: Option<String>,
    pub config_json: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SummarySourceRow {
    pub id: String,
    pub summary_version_id: String,
    pub source_kind: String,
    pub source_message_version_id: Option<String>,
    pub source_start_node_id: Option<String>,
    pub source_end_node_id: Option<String>,
    pub source_summary_version_id: Option<String>,
    pub sort_order: i64,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SummaryUsageRow {
    pub id: String,
    pub summary_group_id: String,
    pub summary_version_id: Option<String>,
    pub usage_scope: String,
    pub target_kind: String,
    pub target_message_version_id: Option<String>,
    pub target_start_node_id: Option<String>,
    pub target_end_node_id: Option<String>,
    pub conversation_id: Option<String>,
    pub activation_mode: String,
    pub replace_from_node_id: Option<String>,
    pub replace_after_message_count: Option<i64>,
    pub replace_after_total_bytes: Option<i64>,
    pub enabled: bool,
    pub priority: i64,
    pub config_json: String,
    pub created_at: i64,
    pub updated_at: i64,
}
