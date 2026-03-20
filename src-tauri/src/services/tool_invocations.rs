use sqlx::SqlitePool;

use crate::db::models::ToolInvocationRow;
use crate::db::repos::tool_invocations as repo;
use crate::db::repos::{messages as messages_repo, plugins as plugins_repo};
use crate::domain::native_capabilities::{
    FinishToolInvocationInput, StartToolInvocationInput, ToolInvocationDetail,
};
use crate::services::content;
use crate::services::content_store::ContentStore;
use crate::support::error::{AppError, Result};

pub async fn start_tool_invocation(
    db: &SqlitePool,
    store: &ContentStore,
    input: &StartToolInvocationInput,
) -> Result<ToolInvocationDetail> {
    validate_run_scope(
        input.generation_run_id.as_deref(),
        input.workflow_run_node_execution_id.as_deref(),
    )?;
    if let Some(generation_run_id) = input.generation_run_id.as_deref() {
        let _ = messages_repo::get_generation_run(db, generation_run_id).await?;
    }
    if let Some(workflow_exec_id) = input.workflow_run_node_execution_id.as_deref() {
        let _ = crate::db::repos::workflows::get_workflow_run_node_execution(db, workflow_exec_id)
            .await?;
    }
    if let Some(message_version_id) = input.message_version_id.as_deref() {
        let _ = messages_repo::get_message_version(db, message_version_id).await?;
    }
    if let Some(plugin_id) = input.plugin_id.as_deref() {
        let _ = plugins_repo::get_plugin(db, plugin_id).await?;
    }

    let request_content_id = match &input.request_content {
        Some(request_content) => Some(
            content::create_content(db, store, request_content)
                .await?
                .content_id,
        ),
        None => None,
    };

    let row = repo::create_tool_invocation(
        db,
        &repo::CreateToolInvocationRecord {
            generation_run_id: input.generation_run_id.as_deref(),
            workflow_run_node_execution_id: input.workflow_run_node_execution_id.as_deref(),
            message_version_id: input.message_version_id.as_deref(),
            tool_kind: &input.tool_kind,
            tool_name: &input.tool_name,
            plugin_id: input.plugin_id.as_deref(),
            request_content_id: request_content_id.as_deref(),
            config_json: &input.config_json.to_string(),
            started_at: None,
        },
    )
    .await?;

    map_tool_invocation_row(db, store, row).await
}

pub async fn finish_tool_invocation(
    db: &SqlitePool,
    store: &ContentStore,
    id: &str,
    input: &FinishToolInvocationInput,
) -> Result<ToolInvocationDetail> {
    let response_content_id = match &input.response_content {
        Some(response_content) => Some(
            content::create_content(db, store, response_content)
                .await?
                .content_id,
        ),
        None => None,
    };

    let row = repo::finish_tool_invocation(
        db,
        id,
        &repo::FinishToolInvocationRecord {
            status: &input.status,
            response_content_id: response_content_id.as_deref(),
            config_json: &input.config_json.to_string(),
            finished_at: None,
        },
    )
    .await?;

    map_tool_invocation_row(db, store, row).await
}

pub async fn list_tool_invocations_by_run(
    db: &SqlitePool,
    store: &ContentStore,
    generation_run_id: Option<&str>,
    workflow_run_id: Option<&str>,
) -> Result<Vec<ToolInvocationDetail>> {
    validate_run_scope(generation_run_id, workflow_run_id)?;
    let rows = repo::list_tool_invocations_by_run(db, generation_run_id, workflow_run_id).await?;
    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        items.push(map_tool_invocation_row(db, store, row).await?);
    }
    Ok(items)
}

async fn map_tool_invocation_row(
    db: &SqlitePool,
    store: &ContentStore,
    row: ToolInvocationRow,
) -> Result<ToolInvocationDetail> {
    Ok(ToolInvocationDetail {
        id: row.id,
        generation_run_id: row.generation_run_id,
        workflow_run_node_execution_id: row.workflow_run_node_execution_id,
        message_version_id: row.message_version_id,
        tool_kind: row.tool_kind,
        tool_name: row.tool_name,
        plugin_id: row.plugin_id,
        request_content: load_optional_content(db, store, row.request_content_id.as_deref())
            .await?,
        response_content: load_optional_content(db, store, row.response_content_id.as_deref())
            .await?,
        status: row.status,
        started_at: row.started_at,
        finished_at: row.finished_at,
        created_at: row.created_at,
        config_json: serde_json::from_str(&row.config_json)?,
    })
}

async fn load_optional_content(
    db: &SqlitePool,
    store: &ContentStore,
    content_id: Option<&str>,
) -> Result<Option<crate::domain::content::StoredContent>> {
    match content_id {
        Some(content_id) => Ok(Some(
            content::get_content(db, store, content_id, false).await?,
        )),
        None => Ok(None),
    }
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
