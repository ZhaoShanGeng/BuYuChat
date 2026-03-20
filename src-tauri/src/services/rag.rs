use sqlx::SqlitePool;

use crate::db::models::RagRefRow;
use crate::db::repos::rag as repo;
use crate::domain::native_capabilities::{CreateRagRefInput, RagRefDetail};
use crate::services::content;
use crate::services::content_store::ContentStore;
use crate::support::error::{AppError, Result};

pub async fn record_rag_ref(
    db: &SqlitePool,
    store: &ContentStore,
    input: &CreateRagRefInput,
) -> Result<RagRefDetail> {
    validate_run_scope(
        input.generation_run_id.as_deref(),
        input.workflow_run_node_execution_id.as_deref(),
    )?;
    if let Some(generation_run_id) = input.generation_run_id.as_deref() {
        let _ = crate::db::repos::messages::get_generation_run(db, generation_run_id).await?;
    }
    if let Some(workflow_exec_id) = input.workflow_run_node_execution_id.as_deref() {
        let _ = crate::db::repos::workflows::get_workflow_run_node_execution(db, workflow_exec_id)
            .await?;
    }

    let excerpt_content_id = match &input.excerpt_content {
        Some(excerpt_content) => Some(
            content::create_content(db, store, excerpt_content)
                .await?
                .content_id,
        ),
        None => None,
    };

    let row = repo::create_rag_ref(
        db,
        &repo::CreateRagRefRecord {
            generation_run_id: input.generation_run_id.as_deref(),
            workflow_run_node_execution_id: input.workflow_run_node_execution_id.as_deref(),
            source_uri: input.source_uri.as_deref(),
            document_title: input.document_title.as_deref(),
            chunk_key: input.chunk_key.as_deref(),
            score: input.score,
            excerpt_content_id: excerpt_content_id.as_deref(),
            included_in_request: input.included_in_request,
            config_json: &input.config_json.to_string(),
        },
    )
    .await?;

    map_rag_row(db, store, row).await
}

pub async fn list_rag_refs_by_run(
    db: &SqlitePool,
    store: &ContentStore,
    generation_run_id: Option<&str>,
    workflow_run_id: Option<&str>,
) -> Result<Vec<RagRefDetail>> {
    validate_run_scope(generation_run_id, workflow_run_id)?;
    let rows = repo::list_rag_refs_by_run(db, generation_run_id, workflow_run_id).await?;
    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        items.push(map_rag_row(db, store, row).await?);
    }
    Ok(items)
}

async fn map_rag_row(
    db: &SqlitePool,
    store: &ContentStore,
    row: RagRefRow,
) -> Result<RagRefDetail> {
    Ok(RagRefDetail {
        id: row.id,
        generation_run_id: row.generation_run_id,
        workflow_run_node_execution_id: row.workflow_run_node_execution_id,
        source_uri: row.source_uri,
        document_title: row.document_title,
        chunk_key: row.chunk_key,
        score: row.score,
        excerpt_content: match row.excerpt_content_id.as_deref() {
            Some(content_id) => Some(content::get_content(db, store, content_id, false).await?),
            None => None,
        },
        included_in_request: row.included_in_request,
        created_at: row.created_at,
        config_json: serde_json::from_str(&row.config_json)?,
    })
}

fn validate_run_scope(
    generation_run_id: Option<&str>,
    workflow_run_id: Option<&str>,
) -> Result<()> {
    if generation_run_id.is_none() && workflow_run_id.is_none() {
        return Err(AppError::Validation(
            "at least one of generation_run_id or workflow_run_id is required".to_string(),
        ));
    }
    Ok(())
}
