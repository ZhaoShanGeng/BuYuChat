use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UserProfileRow {
    pub id: String,
    pub name: String,
    pub title: Option<String>,
    pub description_content_id: Option<String>,
    pub avatar_uri: Option<String>,
    pub injection_position: String,
    pub injection_depth: Option<i64>,
    pub injection_role: Option<String>,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: String,
    pub created_at: i64,
    pub updated_at: i64,
}
