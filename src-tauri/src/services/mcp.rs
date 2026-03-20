use sqlx::SqlitePool;

use crate::db::models::McpEventRow;
use crate::db::repos::mcp as repo;
use crate::domain::native_capabilities::{CreateMcpEventInput, McpEventDetail};
use crate::services::content;
use crate::services::content_store::ContentStore;
use crate::support::error::{AppError, Result};

pub async fn record_mcp_event(
    db: &SqlitePool,
    store: &ContentStore,
    input: &CreateMcpEventInput,
) -> Result<McpEventDetail> {
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

    let payload_content_id = match &input.payload_content {
        Some(payload_content) => Some(
            content::create_content(db, store, payload_content)
                .await?
                .content_id,
        ),
        None => None,
    };

    let row = repo::create_mcp_event(
        db,
        &repo::CreateMcpEventRecord {
            generation_run_id: input.generation_run_id.as_deref(),
            workflow_run_node_execution_id: input.workflow_run_node_execution_id.as_deref(),
            server_name: &input.server_name,
            event_kind: &input.event_kind,
            method_name: input.method_name.as_deref(),
            payload_content_id: payload_content_id.as_deref(),
            status: &input.status,
            config_json: &input.config_json.to_string(),
        },
    )
    .await?;

    map_mcp_row(db, store, row).await
}

pub async fn list_mcp_events_by_run(
    db: &SqlitePool,
    store: &ContentStore,
    generation_run_id: Option<&str>,
    workflow_run_id: Option<&str>,
) -> Result<Vec<McpEventDetail>> {
    validate_run_scope(generation_run_id, workflow_run_id)?;
    let rows = repo::list_mcp_events_by_run(db, generation_run_id, workflow_run_id).await?;
    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        items.push(map_mcp_row(db, store, row).await?);
    }
    Ok(items)
}

async fn map_mcp_row(
    db: &SqlitePool,
    store: &ContentStore,
    row: McpEventRow,
) -> Result<McpEventDetail> {
    Ok(McpEventDetail {
        id: row.id,
        generation_run_id: row.generation_run_id,
        workflow_run_node_execution_id: row.workflow_run_node_execution_id,
        server_name: row.server_name,
        event_kind: row.event_kind,
        method_name: row.method_name,
        payload_content: match row.payload_content_id.as_deref() {
            Some(content_id) => Some(content::get_content(db, store, content_id, false).await?),
            None => None,
        },
        status: row.status,
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
