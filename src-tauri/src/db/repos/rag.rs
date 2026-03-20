use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use crate::db::models::RagRefRow;
use crate::support::error::{AppError, Result};
use crate::support::{ids, time};

pub struct CreateRagRefRecord<'a> {
    pub generation_run_id: Option<&'a str>,
    pub workflow_run_node_execution_id: Option<&'a str>,
    pub source_uri: Option<&'a str>,
    pub document_title: Option<&'a str>,
    pub chunk_key: Option<&'a str>,
    pub score: Option<f32>,
    pub excerpt_content_id: Option<&'a str>,
    pub included_in_request: bool,
    pub config_json: &'a str,
}

pub async fn create_rag_ref(db: &SqlitePool, input: &CreateRagRefRecord<'_>) -> Result<RagRefRow> {
    let id = ids::new_id();
    let created_at = time::now_ms();
    sqlx::query(
        r#"
        INSERT INTO rag_refs (
            id, generation_run_id, workflow_run_node_execution_id, source_uri, document_title,
            chunk_key, score, excerpt_content_id, included_in_request, created_at, config_json
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(input.generation_run_id)
    .bind(input.workflow_run_node_execution_id)
    .bind(input.source_uri)
    .bind(input.document_title)
    .bind(input.chunk_key)
    .bind(input.score)
    .bind(input.excerpt_content_id)
    .bind(input.included_in_request)
    .bind(created_at)
    .bind(input.config_json)
    .execute(db)
    .await?;

    get_rag_ref(db, &id).await
}

pub async fn list_rag_refs_by_run(
    db: &SqlitePool,
    generation_run_id: Option<&str>,
    workflow_run_id: Option<&str>,
) -> Result<Vec<RagRefRow>> {
    let mut builder = QueryBuilder::<Sqlite>::new(
        r#"
        SELECT rr.*
        FROM rag_refs rr
        LEFT JOIN workflow_run_node_executions wrne
            ON wrne.id = rr.workflow_run_node_execution_id
        WHERE 1 = 1
        "#,
    );

    if let Some(generation_run_id) = generation_run_id {
        builder.push(" AND rr.generation_run_id = ");
        builder.push_bind(generation_run_id);
    }
    if let Some(workflow_run_id) = workflow_run_id {
        builder.push(" AND wrne.workflow_run_id = ");
        builder.push_bind(workflow_run_id);
    }

    builder.push(" ORDER BY rr.created_at ASC, rr.id ASC");

    builder
        .build_query_as::<RagRefRow>()
        .fetch_all(db)
        .await
        .map_err(Into::into)
}

pub async fn get_rag_ref(db: &SqlitePool, id: &str) -> Result<RagRefRow> {
    sqlx::query_as::<_, RagRefRow>("SELECT * FROM rag_refs WHERE id = ? LIMIT 1")
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::NotFound {
            entity: "rag_ref",
            id: id.to_string(),
        })
}
