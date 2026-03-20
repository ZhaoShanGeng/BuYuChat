use sqlx::{Sqlite, SqlitePool, Transaction};

use crate::db::models::{ContentChunkRow, ContentObjectRow};
use crate::support::error::{AppError, Result};

pub struct InsertContentObject<'a> {
    pub id: &'a str,
    pub content_type: &'a str,
    pub storage_kind: &'a str,
    pub text_content: Option<&'a str>,
    pub primary_storage_uri: Option<&'a str>,
    pub mime_type: Option<&'a str>,
    pub size_bytes: Option<i64>,
    pub preview_text: Option<&'a str>,
    pub sha256: Option<&'a str>,
    pub config_json: &'a str,
    pub created_at: i64,
}

pub struct InsertContentChunk<'a> {
    pub id: &'a str,
    pub content_id: &'a str,
    pub chunk_index: i64,
    pub storage_uri: &'a str,
    pub byte_offset: i64,
    pub byte_length: i64,
    pub compression: Option<&'a str>,
    pub checksum: Option<&'a str>,
}

pub async fn create_content_object(
    tx: &mut Transaction<'_, Sqlite>,
    input: &InsertContentObject<'_>,
) -> Result<ContentObjectRow> {
    sqlx::query(
        r#"
        INSERT INTO content_objects (
            id, content_type, storage_kind, text_content, primary_storage_uri,
            mime_type, size_bytes, preview_text, sha256, config_json, created_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(input.id)
    .bind(input.content_type)
    .bind(input.storage_kind)
    .bind(input.text_content)
    .bind(input.primary_storage_uri)
    .bind(input.mime_type)
    .bind(input.size_bytes)
    .bind(input.preview_text)
    .bind(input.sha256)
    .bind(input.config_json)
    .bind(input.created_at)
    .execute(tx.as_mut())
    .await?;

    get_content_object_with_executor(tx.as_mut(), input.id).await
}

pub async fn insert_content_chunk(
    tx: &mut Transaction<'_, Sqlite>,
    input: &InsertContentChunk<'_>,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO content_chunks (
            id, content_id, chunk_index, storage_uri, byte_offset,
            byte_length, compression, checksum
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(input.id)
    .bind(input.content_id)
    .bind(input.chunk_index)
    .bind(input.storage_uri)
    .bind(input.byte_offset)
    .bind(input.byte_length)
    .bind(input.compression)
    .bind(input.checksum)
    .execute(tx.as_mut())
    .await?;

    Ok(())
}

pub async fn get_content_object(db: &SqlitePool, id: &str) -> Result<ContentObjectRow> {
    get_content_object_with_executor(db, id).await
}

pub async fn list_content_chunks(
    db: &SqlitePool,
    content_id: &str,
) -> Result<Vec<ContentChunkRow>> {
    sqlx::query_as::<_, ContentChunkRow>(
        r#"
        SELECT *
        FROM content_chunks
        WHERE content_id = ?
        ORDER BY chunk_index ASC
        "#,
    )
    .bind(content_id)
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

async fn get_content_object_with_executor<'e, E>(executor: E, id: &str) -> Result<ContentObjectRow>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    sqlx::query_as::<_, ContentObjectRow>("SELECT * FROM content_objects WHERE id = ? LIMIT 1")
        .bind(id)
        .fetch_optional(executor)
        .await?
        .ok_or_else(|| AppError::NotFound {
            entity: "content_object",
            id: id.to_string(),
        })
}
