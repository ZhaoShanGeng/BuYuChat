use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use crate::db::models::ToolInvocationRow;
use crate::support::error::{AppError, Result};
use crate::support::{ids, time};

pub struct CreateToolInvocationRecord<'a> {
    pub generation_run_id: Option<&'a str>,
    pub workflow_run_node_execution_id: Option<&'a str>,
    pub message_version_id: Option<&'a str>,
    pub tool_kind: &'a str,
    pub tool_name: &'a str,
    pub plugin_id: Option<&'a str>,
    pub request_content_id: Option<&'a str>,
    pub config_json: &'a str,
    pub started_at: Option<i64>,
}

pub struct FinishToolInvocationRecord<'a> {
    pub status: &'a str,
    pub response_content_id: Option<&'a str>,
    pub config_json: &'a str,
    pub finished_at: Option<i64>,
}

pub async fn create_tool_invocation(
    db: &SqlitePool,
    input: &CreateToolInvocationRecord<'_>,
) -> Result<ToolInvocationRow> {
    let id = ids::new_id();
    let created_at = time::now_ms();
    sqlx::query(
        r#"
        INSERT INTO tool_invocations (
            id, generation_run_id, workflow_run_node_execution_id, message_version_id,
            tool_kind, tool_name, plugin_id, request_content_id, response_content_id,
            status, started_at, finished_at, created_at, config_json
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, NULL, 'running', ?, NULL, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(input.generation_run_id)
    .bind(input.workflow_run_node_execution_id)
    .bind(input.message_version_id)
    .bind(input.tool_kind)
    .bind(input.tool_name)
    .bind(input.plugin_id)
    .bind(input.request_content_id)
    .bind(input.started_at.unwrap_or(created_at))
    .bind(created_at)
    .bind(input.config_json)
    .execute(db)
    .await?;

    get_tool_invocation(db, &id).await
}

pub async fn finish_tool_invocation(
    db: &SqlitePool,
    id: &str,
    input: &FinishToolInvocationRecord<'_>,
) -> Result<ToolInvocationRow> {
    let affected = sqlx::query(
        r#"
        UPDATE tool_invocations
        SET status = ?, response_content_id = ?, config_json = ?, finished_at = ?
        WHERE id = ?
        "#,
    )
    .bind(input.status)
    .bind(input.response_content_id)
    .bind(input.config_json)
    .bind(input.finished_at.unwrap_or_else(time::now_ms))
    .bind(id)
    .execute(db)
    .await?
    .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "tool_invocation",
            id: id.to_string(),
        });
    }

    get_tool_invocation(db, id).await
}

pub async fn list_tool_invocations_by_run(
    db: &SqlitePool,
    generation_run_id: Option<&str>,
    workflow_run_id: Option<&str>,
) -> Result<Vec<ToolInvocationRow>> {
    let mut builder = QueryBuilder::<Sqlite>::new(
        r#"
        SELECT ti.*
        FROM tool_invocations ti
        LEFT JOIN workflow_run_node_executions wrne
            ON wrne.id = ti.workflow_run_node_execution_id
        WHERE 1 = 1
        "#,
    );

    if let Some(generation_run_id) = generation_run_id {
        builder.push(" AND ti.generation_run_id = ");
        builder.push_bind(generation_run_id);
    }
    if let Some(workflow_run_id) = workflow_run_id {
        builder.push(" AND wrne.workflow_run_id = ");
        builder.push_bind(workflow_run_id);
    }

    builder.push(" ORDER BY ti.created_at ASC, ti.id ASC");

    builder
        .build_query_as::<ToolInvocationRow>()
        .fetch_all(db)
        .await
        .map_err(Into::into)
}

pub async fn get_tool_invocation(db: &SqlitePool, id: &str) -> Result<ToolInvocationRow> {
    sqlx::query_as::<_, ToolInvocationRow>("SELECT * FROM tool_invocations WHERE id = ? LIMIT 1")
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::NotFound {
            entity: "tool_invocation",
            id: id.to_string(),
        })
}
