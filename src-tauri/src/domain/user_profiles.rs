use serde::{Deserialize, Serialize};

use crate::domain::common::{Id, TimestampMs};
use crate::domain::content::{ContentWriteInput, StoredContent};
use crate::domain::messages::MessageRole;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfileSummary {
    pub id: Id,
    pub name: String,
    pub title: Option<String>,
    pub avatar_uri: Option<String>,
    pub enabled: bool,
    pub sort_order: i64,
    pub created_at: TimestampMs,
    pub updated_at: TimestampMs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfileDetail {
    pub summary: UserProfileSummary,
    pub description_content: Option<StoredContent>,
    pub injection_position: String,
    pub injection_depth: Option<i64>,
    pub injection_role: Option<MessageRole>,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserProfileInput {
    pub name: String,
    pub title: Option<String>,
    pub description_content: Option<ContentWriteInput>,
    pub avatar_uri: Option<String>,
    pub injection_position: String,
    pub injection_depth: Option<i64>,
    pub injection_role: Option<MessageRole>,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserProfileInput {
    pub name: String,
    pub title: Option<String>,
    pub description_content: Option<ContentWriteInput>,
    pub avatar_uri: Option<String>,
    pub injection_position: String,
    pub injection_depth: Option<i64>,
    pub injection_role: Option<MessageRole>,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: serde_json::Value,
}
