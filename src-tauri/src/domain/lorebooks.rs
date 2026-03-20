use serde::{Deserialize, Serialize};

use crate::domain::common::{Id, TimestampMs};
use crate::domain::content::{ContentWriteInput, StoredContent};
use crate::domain::messages::MessageRole;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LorebookSummary {
    pub id: Id,
    pub name: String,
    pub description: Option<String>,
    pub scan_depth: i64,
    pub token_budget: Option<i64>,
    pub insertion_strategy: String,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: serde_json::Value,
    pub created_at: TimestampMs,
    pub updated_at: TimestampMs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LorebookEntryKeyDetail {
    pub id: Id,
    pub entry_id: Id,
    pub key_type: String,
    pub match_type: String,
    pub pattern_text: String,
    pub case_sensitive: bool,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LorebookEntryDetail {
    pub id: Id,
    pub lorebook_id: Id,
    pub title: Option<String>,
    pub primary_content: StoredContent,
    pub activation_strategy: String,
    pub keyword_logic: String,
    pub insertion_position: String,
    pub insertion_order: i64,
    pub insertion_depth: Option<i64>,
    pub insertion_role: Option<MessageRole>,
    pub outlet_name: Option<String>,
    pub entry_scope: String,
    pub enabled: bool,
    pub sort_order: i64,
    pub keys: Vec<LorebookEntryKeyDetail>,
    pub config_json: serde_json::Value,
    pub created_at: TimestampMs,
    pub updated_at: TimestampMs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LorebookDetail {
    pub lorebook: LorebookSummary,
    pub entries: Vec<LorebookEntryDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLorebookInput {
    pub name: String,
    pub description: Option<String>,
    pub scan_depth: i64,
    pub token_budget: Option<i64>,
    pub insertion_strategy: String,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLorebookInput {
    pub name: String,
    pub description: Option<String>,
    pub scan_depth: i64,
    pub token_budget: Option<i64>,
    pub insertion_strategy: String,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLorebookEntryInput {
    pub lorebook_id: Id,
    pub title: Option<String>,
    pub primary_content: ContentWriteInput,
    pub activation_strategy: String,
    pub keyword_logic: String,
    pub insertion_position: String,
    pub insertion_order: i64,
    pub insertion_depth: Option<i64>,
    pub insertion_role: Option<MessageRole>,
    pub outlet_name: Option<String>,
    pub entry_scope: String,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLorebookEntryInput {
    pub title: Option<String>,
    pub primary_content: ContentWriteInput,
    pub activation_strategy: String,
    pub keyword_logic: String,
    pub insertion_position: String,
    pub insertion_order: i64,
    pub insertion_depth: Option<i64>,
    pub insertion_role: Option<MessageRole>,
    pub outlet_name: Option<String>,
    pub entry_scope: String,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LorebookMatchInput {
    pub conversation_id: Option<Id>,
    pub lorebook_id: Id,
    pub recent_messages: Vec<String>,
    pub max_entries: usize,
    pub include_disabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchedLorebookEntry {
    pub lorebook_entry_id: Id,
    pub score: f32,
    pub matched_keys: Vec<String>,
    pub content: StoredContent,
    pub config_json: serde_json::Value,
}
