use std::{fs, path::Path};

use sha2::{Digest, Sha256};
use sqlx::SqlitePool;

use crate::db::models::{ContentChunkRow, ContentObjectRow};
use crate::db::repos::content as repo;
use crate::domain::content::{ContentStorageKind, ContentType, ContentWriteInput, StoredContent};
use crate::services::content_store::{ContentStore, ImportedFile};
use crate::support::error::{AppError, Result};
use crate::support::{ids, time};

pub async fn create_content(
    db: &SqlitePool,
    store: &ContentStore,
    input: &ContentWriteInput,
) -> Result<StoredContent> {
    let created_at = time::now_ms();
    let content_id = ids::new_id();

    let text_content = match (&input.text_content, &input.source_file_path) {
        (Some(text), _) => Some(text.clone()),
        (None, Some(path)) if !prefers_file_ref(input) => Some(fs::read_to_string(path)?),
        _ => None,
    };

    let size_bytes = match (
        &text_content,
        &input.source_file_path,
        input.size_bytes_hint,
    ) {
        (Some(text), _, _) => text.len() as u64,
        (None, Some(path), _) => fs::metadata(path)?.len(),
        (None, None, Some(size)) => size,
        (None, None, None) => 0,
    };

    let storage_kind = if input.primary_storage_uri.is_some() {
        ContentStorageKind::FileRef
    } else {
        map_storage_kind(store.choose_storage_kind(
            input.content_type.as_str(),
            input.mime_type.as_deref(),
            size_bytes,
        ))
    };

    let preview_text = input
        .preview_text
        .clone()
        .or_else(|| text_content.as_deref().map(derive_preview_text));

    let mut tx = db.begin().await?;

    let stored = match storage_kind {
        ContentStorageKind::Inline => {
            let text = text_content.ok_or_else(|| {
                AppError::Validation("inline content requires text_content".to_string())
            })?;

            let row = repo::create_content_object(
                &mut tx,
                &repo::InsertContentObject {
                    id: &content_id,
                    content_type: input.content_type.as_str(),
                    storage_kind: storage_kind.as_str(),
                    text_content: Some(&text),
                    primary_storage_uri: input.primary_storage_uri.as_deref(),
                    mime_type: input.mime_type.as_deref(),
                    size_bytes: Some(size_bytes as i64),
                    preview_text: preview_text.as_deref(),
                    sha256: Some(&sha256_hex(text.as_bytes())),
                    config_json: &input.config_json.to_string(),
                    created_at,
                },
            )
            .await?;

            map_content_row(row, Vec::new(), true)
        }
        ContentStorageKind::Chunked => {
            let text = text_content.ok_or_else(|| {
                AppError::Validation("chunked content requires text_content".to_string())
            })?;
            let chunks = store.persist_text_chunks(&content_id, text.as_bytes())?;

            let row = repo::create_content_object(
                &mut tx,
                &repo::InsertContentObject {
                    id: &content_id,
                    content_type: input.content_type.as_str(),
                    storage_kind: storage_kind.as_str(),
                    text_content: None,
                    primary_storage_uri: input.primary_storage_uri.as_deref(),
                    mime_type: input.mime_type.as_deref(),
                    size_bytes: Some(size_bytes as i64),
                    preview_text: preview_text.as_deref(),
                    sha256: Some(&sha256_hex(text.as_bytes())),
                    config_json: &input.config_json.to_string(),
                    created_at,
                },
            )
            .await?;

            for chunk in &chunks {
                repo::insert_content_chunk(
                    &mut tx,
                    &repo::InsertContentChunk {
                        id: &chunk.id,
                        content_id: &content_id,
                        chunk_index: chunk.chunk_index as i64,
                        storage_uri: &chunk.storage_uri,
                        byte_offset: chunk.byte_offset,
                        byte_length: chunk.byte_length,
                        compression: chunk.compression.as_deref(),
                        checksum: chunk.checksum.as_deref(),
                    },
                )
                .await?;
            }

            Ok(StoredContent {
                content_id: row.id,
                content_type: ContentType::parse(&row.content_type)?,
                storage_kind: ContentStorageKind::parse(&row.storage_kind)?,
                mime_type: row.mime_type,
                size_bytes: row.size_bytes.unwrap_or_default().max(0) as u64,
                preview_text: row.preview_text,
                primary_storage_uri: row.primary_storage_uri,
                text_content: None,
                chunk_count: chunks.len() as u32,
                sha256: row.sha256,
                config_json: serde_json::from_str(&row.config_json)?,
            })
        }
        ContentStorageKind::FileRef => {
            let imported = resolve_file_ref(store, input)?;
            let row = repo::create_content_object(
                &mut tx,
                &repo::InsertContentObject {
                    id: &content_id,
                    content_type: input.content_type.as_str(),
                    storage_kind: storage_kind.as_str(),
                    text_content: None,
                    primary_storage_uri: Some(&imported.primary_storage_uri),
                    mime_type: input.mime_type.as_deref(),
                    size_bytes: Some(imported.size_bytes as i64),
                    preview_text: preview_text.as_deref(),
                    sha256: imported.sha256.as_deref(),
                    config_json: &input.config_json.to_string(),
                    created_at,
                },
            )
            .await?;

            map_content_row(row, Vec::new(), false)
        }
    };

    let stored = stored?;
    tx.commit().await?;
    Ok(stored)
}

pub async fn get_content(
    db: &SqlitePool,
    store: &ContentStore,
    content_id: &str,
    include_body: bool,
) -> Result<StoredContent> {
    let row = repo::get_content_object(db, content_id).await?;
    let chunks = if matches!(
        ContentStorageKind::parse(&row.storage_kind)?,
        ContentStorageKind::Chunked
    ) {
        repo::list_content_chunks(db, content_id).await?
    } else {
        Vec::new()
    };

    let mut stored = map_content_row(row, chunks.clone(), include_body)?;
    if include_body && matches!(stored.storage_kind, ContentStorageKind::Chunked) {
        stored.text_content = Some(store.read_text_chunks(&chunks)?);
    }

    Ok(stored)
}

fn map_content_row(
    row: ContentObjectRow,
    chunks: Vec<ContentChunkRow>,
    include_inline_text: bool,
) -> Result<StoredContent> {
    Ok(StoredContent {
        content_id: row.id,
        content_type: ContentType::parse(&row.content_type)?,
        storage_kind: ContentStorageKind::parse(&row.storage_kind)?,
        mime_type: row.mime_type,
        size_bytes: row.size_bytes.unwrap_or_default().max(0) as u64,
        preview_text: row.preview_text,
        primary_storage_uri: row.primary_storage_uri,
        text_content: if include_inline_text {
            row.text_content
        } else {
            None
        },
        chunk_count: chunks.len() as u32,
        sha256: row.sha256,
        config_json: serde_json::from_str(&row.config_json)?,
    })
}

fn resolve_file_ref(store: &ContentStore, input: &ContentWriteInput) -> Result<ImportedFile> {
    if let Some(uri) = &input.primary_storage_uri {
        return Ok(ImportedFile {
            primary_storage_uri: uri.clone(),
            size_bytes: input.size_bytes_hint.unwrap_or_default(),
            sha256: None,
        });
    }

    let source = input.source_file_path.as_deref().ok_or_else(|| {
        AppError::Validation(
            "file_ref content requires source_file_path or primary_storage_uri".to_string(),
        )
    })?;

    store.import_file(Path::new(source))
}

fn prefers_file_ref(input: &ContentWriteInput) -> bool {
    matches!(
        input.content_type,
        ContentType::Image
            | ContentType::Audio
            | ContentType::Video
            | ContentType::File
            | ContentType::Binary
    )
}

fn derive_preview_text(text: &str) -> String {
    text.chars().take(1024).collect()
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format_digest(&hasher.finalize())
}

fn format_digest(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{byte:02x}");
    }
    out
}

fn map_storage_kind(
    kind: crate::services::content_store::ContentStorageKind,
) -> ContentStorageKind {
    match kind {
        crate::services::content_store::ContentStorageKind::Inline => ContentStorageKind::Inline,
        crate::services::content_store::ContentStorageKind::Chunked => ContentStorageKind::Chunked,
        crate::services::content_store::ContentStorageKind::FileRef => ContentStorageKind::FileRef,
    }
}
