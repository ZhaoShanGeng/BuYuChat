use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use crate::db::models::McpEventRow;
use crate::support::error::{AppError, Result};
use crate::support::{ids, time};

pub struct CreateMcpEventRecord<'a> {
    pub generation_run_id: Option<&'a str>,
    pub workflow_run_node_execution_id: Option<&'a str>,
    pub server_name: &'a str,
    pub event_kind: &'a str,
    pub method_name: Option<&'a str>,
    pub payload_content_id: Option<&'a str>,
    pub status: &'a str,
    pub config_json: &'a str,
}

pub async fn create_mcp_event(
    db: &SqlitePool,
    input: &CreateMcpEventRecord<'_>,
) -> Result<McpEventRow> {
    let id = ids::new_id();
    let created_at = time::now_ms();
    sqlx::query(
        r#"
        INSERT INTO mcp_events (
            id, generation_run_id, workflow_run_node_execution_id, server_name, event_kind,
            method_name, payload_content_id, status, created_at, config_json
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(input.generation_run_id)
    .bind(input.workflow_run_node_execution_id)
    .bind(input.server_name)
    .bind(input.event_kind)
    .bind(input.method_name)
    .bind(input.payload_content_id)
    .bind(input.status)
    .bind(created_at)
    .bind(input.config_json)
    .execute(db)
    .await?;

    get_mcp_event(db, &id).await
}

pub async fn list_mcp_events_by_run(
    db: &SqlitePool,
    generation_run_id: Option<&str>,
    workflow_run_id: Option<&str>,
) -> Result<Vec<McpEventRow>> {
    let mut builder = QueryBuilder::<Sqlite>::new(
        r#"
        SELECT me.*
        FROM mcp_events me
        LEFT JOIN workflow_run_node_executions wrne
            ON wrne.id = me.workflow_run_node_execution_id
        WHERE 1 = 1
        "#,
    );

    if let Some(generation_run_id) = generation_run_id {
        builder.push(" AND me.generation_run_id = ");
        builder.push_bind(generation_run_id);
    }
    if let Some(workflow_run_id) = workflow_run_id {
        builder.push(" AND wrne.workflow_run_id = ");
        builder.push_bind(workflow_run_id);
    }

    builder.push(" ORDER BY me.created_at ASC, me.id ASC");

    builder
        .build_query_as::<McpEventRow>()
        .fetch_all(db)
        .await
        .map_err(Into::into)
}

pub async fn get_mcp_event(db: &SqlitePool, id: &str) -> Result<McpEventRow> {
    sqlx::query_as::<_, McpEventRow>("SELECT * FROM mcp_events WHERE id = ? LIMIT 1")
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::NotFound {
            entity: "mcp_event",
            id: id.to_string(),
        })
}
