use tauri::{AppHandle, State};

use crate::app::state::AppState;
use crate::commands::incremental;
use crate::domain::workflows::{
    CreateWorkflowDefInput, RunWorkflowInput, UpdateWorkflowDefInput, WorkflowDefDetail,
    WorkflowDefSummary, WorkflowEdgeInput, WorkflowNodeExecutionResult, WorkflowNodeInput,
    WorkflowRunResult,
};
use crate::support::error::Result;

#[tauri::command]
pub async fn list_workflow_defs(state: State<'_, AppState>) -> Result<Vec<WorkflowDefSummary>> {
    crate::services::workflows::list_workflow_defs(&state.db).await
}

#[tauri::command]
pub async fn get_workflow_def_detail(
    state: State<'_, AppState>,
    id: String,
) -> Result<WorkflowDefDetail> {
    crate::services::workflows::get_workflow_def_detail(&state.db, &id).await
}

#[tauri::command]
pub async fn create_workflow_def(
    app: AppHandle,
    state: State<'_, AppState>,
    input: CreateWorkflowDefInput,
) -> Result<WorkflowDefDetail> {
    let detail = crate::services::workflows::create_workflow_def(&state.db, &input).await?;
    incremental::emit_upsert(
        &app,
        "global",
        None,
        "workflow_def",
        Some(&detail.summary.id),
        &detail,
    )?;
    Ok(detail)
}

#[tauri::command]
pub async fn update_workflow_def(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    input: UpdateWorkflowDefInput,
) -> Result<WorkflowDefDetail> {
    let detail = crate::services::workflows::update_workflow_def(&state.db, &id, &input).await?;
    incremental::emit_upsert(
        &app,
        "global",
        None,
        "workflow_def",
        Some(&detail.summary.id),
        &detail,
    )?;
    Ok(detail)
}

#[tauri::command]
pub async fn delete_workflow_def(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<()> {
    crate::services::workflows::delete_workflow_def(&state.db, &id).await?;
    incremental::emit_delete(&app, "global", None, "workflow_def", &id)?;
    Ok(())
}

#[tauri::command]
pub async fn replace_workflow_nodes(
    app: AppHandle,
    state: State<'_, AppState>,
    workflow_def_id: String,
    items: Vec<WorkflowNodeInput>,
) -> Result<Vec<crate::domain::workflows::WorkflowNodeDetail>> {
    crate::services::workflows::replace_workflow_nodes(&state.db, &workflow_def_id, &items).await?;
    let detail =
        crate::services::workflows::get_workflow_def_detail(&state.db, &workflow_def_id).await?;
    incremental::emit_replace(
        &app,
        "workflow_def",
        Some(&workflow_def_id),
        "workflow_nodes",
        &detail.nodes,
    )?;
    Ok(detail.nodes)
}

#[tauri::command]
pub async fn replace_workflow_edges(
    app: AppHandle,
    state: State<'_, AppState>,
    workflow_def_id: String,
    items: Vec<WorkflowEdgeInput>,
) -> Result<Vec<crate::domain::workflows::WorkflowEdgeDetail>> {
    crate::services::workflows::replace_workflow_edges(&state.db, &workflow_def_id, &items).await?;
    let detail =
        crate::services::workflows::get_workflow_def_detail(&state.db, &workflow_def_id).await?;
    incremental::emit_replace(
        &app,
        "workflow_def",
        Some(&workflow_def_id),
        "workflow_edges",
        &detail.edges,
    )?;
    Ok(detail.edges)
}

#[tauri::command]
pub async fn run_workflow(
    app: AppHandle,
    state: State<'_, AppState>,
    input: RunWorkflowInput,
) -> Result<WorkflowRunResult> {
    let result = crate::services::workflows::run_workflow(
        &state.db,
        &state.content_store,
        &state.provider_registry,
        &input,
    )
    .await?;
    incremental::emit_upsert(
        &app,
        "workflow_def",
        Some(&input.workflow_def_id),
        "workflow_run",
        Some(&result.workflow_run_id),
        &result,
    )?;
    Ok(result)
}

#[tauri::command]
pub async fn list_workflow_run_node_executions(
    state: State<'_, AppState>,
    workflow_run_id: String,
) -> Result<Vec<WorkflowNodeExecutionResult>> {
    crate::services::workflows::list_workflow_run_node_executions(&state.db, &workflow_run_id).await
}
