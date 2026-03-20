use sqlx::SqlitePool;

use crate::db::models::{WorkflowDefEdgeRow, WorkflowDefNodeRow, WorkflowDefRow};
use crate::db::repos::{
    agents, api_channels as channel_repo, lorebooks, presets, user_profiles, workflows as repo,
};
use crate::domain::workflows::{
    CreateWorkflowDefInput, RunWorkflowInput, UpdateWorkflowDefInput, WorkflowDefDetail,
    WorkflowDefSummary, WorkflowEdgeDetail, WorkflowEdgeInput, WorkflowNodeDetail,
    WorkflowNodeExecutionResult, WorkflowNodeInput, WorkflowRunResult,
};
use crate::providers::ProviderRegistry;
use crate::services::content_store::ContentStore;
use crate::services::workflows_runtime;
use crate::support::error::{AppError, Result};

pub async fn list_workflow_defs(db: &SqlitePool) -> Result<Vec<WorkflowDefSummary>> {
    repo::list_workflow_defs(db)
        .await?
        .into_iter()
        .map(map_workflow_def_summary)
        .collect()
}

pub async fn get_workflow_def_detail(db: &SqlitePool, id: &str) -> Result<WorkflowDefDetail> {
    let summary = repo::get_workflow_def(db, id).await?;
    build_workflow_def_detail(db, summary).await
}

pub async fn create_workflow_def(
    db: &SqlitePool,
    input: &CreateWorkflowDefInput,
) -> Result<WorkflowDefDetail> {
    let row = repo::create_workflow_def(
        db,
        &repo::CreateWorkflowDefRecord {
            name: &input.name,
            description: input.description.as_deref(),
            enabled: input.enabled,
            sort_order: input.sort_order,
            config_json: &input.config_json.to_string(),
        },
    )
    .await?;
    build_workflow_def_detail(db, row).await
}

pub async fn update_workflow_def(
    db: &SqlitePool,
    id: &str,
    input: &UpdateWorkflowDefInput,
) -> Result<WorkflowDefDetail> {
    let row = repo::update_workflow_def(
        db,
        id,
        &repo::UpdateWorkflowDefRecord {
            name: &input.name,
            description: input.description.as_deref(),
            enabled: input.enabled,
            sort_order: input.sort_order,
            config_json: &input.config_json.to_string(),
        },
    )
    .await?;
    build_workflow_def_detail(db, row).await
}

pub async fn delete_workflow_def(db: &SqlitePool, id: &str) -> Result<()> {
    repo::delete_workflow_def(db, id).await
}

pub async fn replace_workflow_nodes(
    db: &SqlitePool,
    workflow_def_id: &str,
    items: &[WorkflowNodeInput],
) -> Result<()> {
    let _ = repo::get_workflow_def(db, workflow_def_id).await?;
    validate_nodes(db, items).await?;

    let owned = items
        .iter()
        .map(|item| OwnedWorkflowNode {
            node_key: item.node_key.clone(),
            name: item.name.clone(),
            node_type: item.node_type.clone(),
            agent_id: item.agent_id.clone(),
            plugin_id: item.plugin_id.clone(),
            preset_id: item.preset_id.clone(),
            lorebook_id: item.lorebook_id.clone(),
            user_profile_id: item.user_profile_id.clone(),
            api_channel_id: item.api_channel_id.clone(),
            api_channel_model_id: item.api_channel_model_id.clone(),
            workspace_mode: item.workspace_mode.clone(),
            history_read_mode: item.history_read_mode.clone(),
            summary_write_mode: item.summary_write_mode.clone(),
            message_write_mode: item.message_write_mode.clone(),
            visible_output_mode: item.visible_output_mode.clone(),
            config_json: item.config_json.to_string(),
        })
        .collect::<Vec<_>>();

    let records = owned
        .iter()
        .map(|item| repo::WorkflowDefNodeRecord {
            node_key: &item.node_key,
            name: item.name.as_deref(),
            node_type: &item.node_type,
            agent_id: item.agent_id.as_deref(),
            plugin_id: item.plugin_id.as_deref(),
            preset_id: item.preset_id.as_deref(),
            lorebook_id: item.lorebook_id.as_deref(),
            user_profile_id: item.user_profile_id.as_deref(),
            api_channel_id: item.api_channel_id.as_deref(),
            api_channel_model_id: item.api_channel_model_id.as_deref(),
            workspace_mode: &item.workspace_mode,
            history_read_mode: &item.history_read_mode,
            summary_write_mode: &item.summary_write_mode,
            message_write_mode: &item.message_write_mode,
            visible_output_mode: &item.visible_output_mode,
            config_json: &item.config_json,
        })
        .collect::<Vec<_>>();

    let mut tx = db.begin().await?;
    repo::replace_workflow_def_nodes(&mut tx, workflow_def_id, &records).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn replace_workflow_edges(
    db: &SqlitePool,
    workflow_def_id: &str,
    items: &[WorkflowEdgeInput],
) -> Result<()> {
    let _ = repo::get_workflow_def(db, workflow_def_id).await?;
    let nodes = repo::list_workflow_def_nodes(db, workflow_def_id).await?;
    validate_edges(&nodes, items)?;

    let owned = items
        .iter()
        .map(|item| OwnedWorkflowEdge {
            from_node_id: item.from_node_id.clone(),
            to_node_id: item.to_node_id.clone(),
            edge_type: item.edge_type.clone(),
            priority: item.priority,
            condition_expr: item.condition_expr.clone(),
            label: item.label.clone(),
            enabled: item.enabled,
            config_json: item.config_json.to_string(),
        })
        .collect::<Vec<_>>();
    let records = owned
        .iter()
        .map(|item| repo::WorkflowDefEdgeRecord {
            from_node_id: &item.from_node_id,
            to_node_id: &item.to_node_id,
            edge_type: &item.edge_type,
            priority: item.priority,
            condition_expr: item.condition_expr.as_deref(),
            label: item.label.as_deref(),
            enabled: item.enabled,
            config_json: &item.config_json,
        })
        .collect::<Vec<_>>();

    let mut tx = db.begin().await?;
    repo::replace_workflow_def_edges(&mut tx, workflow_def_id, &records).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn run_workflow(
    db: &SqlitePool,
    store: &ContentStore,
    providers: &ProviderRegistry,
    input: &RunWorkflowInput,
) -> Result<WorkflowRunResult> {
    workflows_runtime::run_workflow(db, store, providers, input).await
}

pub async fn list_workflow_run_node_executions(
    db: &SqlitePool,
    workflow_run_id: &str,
) -> Result<Vec<WorkflowNodeExecutionResult>> {
    repo::list_workflow_run_node_executions(db, workflow_run_id)
        .await?
        .into_iter()
        .map(map_execution_result)
        .collect()
}

async fn build_workflow_def_detail(
    db: &SqlitePool,
    row: WorkflowDefRow,
) -> Result<WorkflowDefDetail> {
    let nodes = repo::list_workflow_def_nodes(db, &row.id).await?;
    let edges = repo::list_workflow_def_edges(db, &row.id).await?;

    Ok(WorkflowDefDetail {
        summary: map_workflow_def_summary(row)?,
        nodes: nodes
            .into_iter()
            .map(map_workflow_node_detail)
            .collect::<Result<Vec<_>>>()?,
        edges: edges
            .into_iter()
            .map(map_workflow_edge_detail)
            .collect::<Result<Vec<_>>>()?,
    })
}

async fn validate_nodes(db: &SqlitePool, items: &[WorkflowNodeInput]) -> Result<()> {
    for item in items {
        if let Some(agent_id) = item.agent_id.as_deref() {
            let _ = agents::get_agent(db, agent_id).await?;
        }
        if let Some(preset_id) = item.preset_id.as_deref() {
            let _ = presets::get_preset(db, preset_id).await?;
        }
        if let Some(lorebook_id) = item.lorebook_id.as_deref() {
            let _ = lorebooks::get_lorebook(db, lorebook_id).await?;
        }
        if let Some(user_profile_id) = item.user_profile_id.as_deref() {
            let _ = user_profiles::get_user_profile(db, user_profile_id).await?;
        }
        if let Some(channel_id) = item.api_channel_id.as_deref() {
            let _ = channel_repo::get_channel(db, channel_id).await?;
        }
        if let Some(channel_model_id) = item.api_channel_model_id.as_deref() {
            let model = channel_repo::get_channel_model_by_id(db, channel_model_id).await?;
            if let Some(channel_id) = item.api_channel_id.as_deref() {
                if model.channel_id != channel_id {
                    return Err(AppError::Validation(
                        "workflow node api_channel_model_id does not belong to api_channel_id"
                            .to_string(),
                    ));
                }
            }
        }
    }

    Ok(())
}

fn validate_edges(nodes: &[WorkflowDefNodeRow], items: &[WorkflowEdgeInput]) -> Result<()> {
    let node_ids = nodes
        .iter()
        .map(|item| item.id.as_str())
        .collect::<std::collections::HashSet<_>>();
    for item in items {
        if !node_ids.contains(item.from_node_id.as_str()) {
            return Err(AppError::Validation(format!(
                "workflow edge from_node_id '{}' does not belong to this workflow",
                item.from_node_id
            )));
        }
        if !node_ids.contains(item.to_node_id.as_str()) {
            return Err(AppError::Validation(format!(
                "workflow edge to_node_id '{}' does not belong to this workflow",
                item.to_node_id
            )));
        }
    }
    Ok(())
}

fn map_workflow_def_summary(row: WorkflowDefRow) -> Result<WorkflowDefSummary> {
    Ok(WorkflowDefSummary {
        id: row.id,
        name: row.name,
        description: row.description,
        enabled: row.enabled,
        sort_order: row.sort_order,
        config_json: parse_json(&row.config_json, "workflow_defs.config_json")?,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

fn map_workflow_node_detail(row: WorkflowDefNodeRow) -> Result<WorkflowNodeDetail> {
    Ok(WorkflowNodeDetail {
        id: row.id,
        workflow_def_id: row.workflow_def_id,
        node_key: row.node_key,
        name: row.name,
        node_type: row.node_type,
        agent_id: row.agent_id,
        plugin_id: row.plugin_id,
        preset_id: row.preset_id,
        lorebook_id: row.lorebook_id,
        user_profile_id: row.user_profile_id,
        api_channel_id: row.api_channel_id,
        api_channel_model_id: row.api_channel_model_id,
        workspace_mode: row.workspace_mode,
        history_read_mode: row.history_read_mode,
        summary_write_mode: row.summary_write_mode,
        message_write_mode: row.message_write_mode,
        visible_output_mode: row.visible_output_mode,
        config_json: parse_json(&row.config_json, "workflow_def_nodes.config_json")?,
    })
}

fn map_workflow_edge_detail(row: WorkflowDefEdgeRow) -> Result<WorkflowEdgeDetail> {
    Ok(WorkflowEdgeDetail {
        id: row.id,
        workflow_def_id: row.workflow_def_id,
        from_node_id: row.from_node_id,
        to_node_id: row.to_node_id,
        edge_type: row.edge_type,
        priority: row.priority,
        condition_expr: row.condition_expr,
        label: row.label,
        enabled: row.enabled,
        config_json: parse_json(&row.config_json, "workflow_def_edges.config_json")?,
    })
}

fn map_execution_result(
    row: crate::db::models::WorkflowRunNodeExecutionRow,
) -> Result<WorkflowNodeExecutionResult> {
    Ok(WorkflowNodeExecutionResult {
        execution_id: row.id,
        workflow_run_id: row.workflow_run_id,
        workflow_def_node_id: row.workflow_def_node_id,
        status: row.status,
        output_content_id: row.output_content_id,
    })
}

fn parse_json(raw: &str, field: &'static str) -> Result<serde_json::Value> {
    serde_json::from_str(raw)
        .map_err(|err| AppError::Validation(format!("failed to parse {field} as json: {err}")))
}

struct OwnedWorkflowNode {
    node_key: String,
    name: Option<String>,
    node_type: String,
    agent_id: Option<String>,
    plugin_id: Option<String>,
    preset_id: Option<String>,
    lorebook_id: Option<String>,
    user_profile_id: Option<String>,
    api_channel_id: Option<String>,
    api_channel_model_id: Option<String>,
    workspace_mode: String,
    history_read_mode: String,
    summary_write_mode: String,
    message_write_mode: String,
    visible_output_mode: String,
    config_json: String,
}

struct OwnedWorkflowEdge {
    from_node_id: String,
    to_node_id: String,
    edge_type: String,
    priority: i64,
    condition_expr: Option<String>,
    label: Option<String>,
    enabled: bool,
    config_json: String,
}
