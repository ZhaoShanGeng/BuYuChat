use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ContentObjectRow {
    pub id: String,
    pub content_type: String,
    pub storage_kind: String,
    pub text_content: Option<String>,
    pub primary_storage_uri: Option<String>,
    pub mime_type: Option<String>,
    pub size_bytes: Option<i64>,
    pub preview_text: Option<String>,
    pub sha256: Option<String>,
    pub config_json: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ContentChunkRow {
    pub id: String,
    pub content_id: String,
    pub chunk_index: i64,
    pub storage_uri: String,
    pub byte_offset: i64,
    pub byte_length: i64,
    pub compression: Option<String>,
    pub checksum: Option<String>,
}
