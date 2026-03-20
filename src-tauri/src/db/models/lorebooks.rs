use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct LorebookRow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub scan_depth: i64,
    pub token_budget: Option<i64>,
    pub insertion_strategy: String,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct LorebookEntryRow {
    pub id: String,
    pub lorebook_id: String,
    pub title: Option<String>,
    pub primary_content_id: String,
    pub activation_strategy: String,
    pub keyword_logic: String,
    pub insertion_position: String,
    pub insertion_order: i64,
    pub insertion_depth: Option<i64>,
    pub insertion_role: Option<String>,
    pub outlet_name: Option<String>,
    pub entry_scope: String,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct LorebookEntryKeyRow {
    pub id: String,
    pub entry_id: String,
    pub key_type: String,
    pub match_type: String,
    pub pattern_text: String,
    pub case_sensitive: bool,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: String,
}
