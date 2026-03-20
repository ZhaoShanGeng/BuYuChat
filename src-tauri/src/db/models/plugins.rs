use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PluginDefRow {
    pub id: String,
    pub name: String,
    pub plugin_key: String,
    pub version: String,
    pub runtime_kind: String,
    pub entrypoint: Option<String>,
    pub enabled: bool,
    pub sort_order: i64,
    pub capabilities_json: String,
    pub permissions_json: String,
    pub config_json: String,
    pub created_at: i64,
    pub updated_at: i64,
}
