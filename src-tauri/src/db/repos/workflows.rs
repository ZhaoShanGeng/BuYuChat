use sqlx::{Sqlite, SqlitePool, Transaction};

use crate::db::models::{
    WorkflowDefEdgeRow, WorkflowDefNodeRow, WorkflowDefRow, WorkflowRunNodeExecutionRow,
    WorkflowRunRow, WorkflowRunWriteRow,
};
use crate::support::error::{AppError, Result};
use crate::support::{ids, time};

pub struct CreateWorkflowDefRecord<'a> {
    pub name: &'a str,
    pub description: Option<&'a str>,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: &'a str,
}

pub struct UpdateWorkflowDefRecord<'a> {
    pub name: &'a str,
    pub description: Option<&'a str>,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: &'a str,
}

pub struct WorkflowDefNodeRecord<'a> {
    pub node_key: &'a str,
    pub name: Option<&'a str>,
    pub node_type: &'a str,
    pub agent_id: Option<&'a str>,
    pub plugin_id: Option<&'a str>,
    pub preset_id: Option<&'a str>,
    pub lorebook_id: Option<&'a str>,
    pub user_profile_id: Option<&'a str>,
    pub api_channel_id: Option<&'a str>,
    pub api_channel_model_id: Option<&'a str>,
    pub workspace_mode: &'a str,
    pub history_read_mode: &'a str,
    pub summary_write_mode: &'a str,
    pub message_write_mode: &'a str,
    pub visible_output_mode: &'a str,
    pub config_json: &'a str,
}

pub struct WorkflowDefEdgeRecord<'a> {
    pub from_node_id: &'a str,
    pub to_node_id: &'a str,
    pub edge_type: &'a str,
    pub priority: i64,
    pub condition_expr: Option<&'a str>,
    pub label: Option<&'a str>,
    pub enabled: bool,
    pub config_json: &'a str,
}

pub struct CreateWorkflowRunRecord<'a> {
    pub workflow_def_id: &'a str,
    pub trigger_conversation_id: Option<&'a str>,
    pub workspace_conversation_id: Option<&'a str>,
    pub workspace_mode: &'a str,
    pub trigger_message_version_id: Option<&'a str>,
    pub entry_node_id: Option<&'a str>,
    pub status: &'a str,
    pub result_message_version_id: Option<&'a str>,
    pub request_snapshot_content_id: Option<&'a str>,
    pub result_content_id: Option<&'a str>,
    pub config_json: &'a str,
    pub started_at: Option<i64>,
    pub finished_at: Option<i64>,
}

pub struct FinishWorkflowRunRecord<'a> {
    pub status: &'a str,
    pub result_message_version_id: Option<&'a str>,
    pub result_content_id: Option<&'a str>,
    pub finished_at: Option<i64>,
    pub config_json: &'a str,
}

pub struct CreateWorkflowRunNodeExecutionRecord<'a> {
    pub workflow_run_id: &'a str,
    pub workflow_def_node_id: &'a str,
    pub parent_execution_id: Option<&'a str>,
    pub incoming_edge_id: Option<&'a str>,
    pub branch_key: Option<&'a str>,
    pub loop_iteration: i64,
    pub retry_index: i64,
    pub status: &'a str,
    pub generation_run_id: Option<&'a str>,
    pub input_snapshot_content_id: Option<&'a str>,
    pub output_content_id: Option<&'a str>,
    pub error_content_id: Option<&'a str>,
    pub started_at: Option<i64>,
    pub finished_at: Option<i64>,
    pub config_json: &'a str,
}

pub struct FinishWorkflowRunNodeExecutionRecord<'a> {
    pub status: &'a str,
    pub generation_run_id: Option<&'a str>,
    pub output_content_id: Option<&'a str>,
    pub error_content_id: Option<&'a str>,
    pub finished_at: Option<i64>,
    pub config_json: &'a str,
}

pub struct CreateWorkflowRunWriteRecord<'a> {
    pub workflow_run_id: &'a str,
    pub workflow_run_node_execution_id: Option<&'a str>,
    pub write_kind: &'a str,
    pub apply_mode: &'a str,
    pub content_id: &'a str,
    pub target_conversation_id: Option<&'a str>,
    pub target_message_node_id: Option<&'a str>,
    pub target_summary_group_id: Option<&'a str>,
    pub target_lorebook_entry_id: Option<&'a str>,
    pub target_preset_entry_id: Option<&'a str>,
    pub target_agent_id: Option<&'a str>,
    pub target_user_profile_id: Option<&'a str>,
    pub target_plugin_id: Option<&'a str>,
    pub target_slot: Option<&'a str>,
    pub visible_to_user: bool,
    pub config_json: &'a str,
}

pub async fn list_workflow_defs(db: &SqlitePool) -> Result<Vec<WorkflowDefRow>> {
    sqlx::query_as::<_, WorkflowDefRow>(
        r#"
        SELECT *
        FROM workflow_defs
        ORDER BY enabled DESC, sort_order ASC, updated_at DESC
        "#,
    )
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

pub async fn get_workflow_def(db: &SqlitePool, id: &str) -> Result<WorkflowDefRow> {
    sqlx::query_as::<_, WorkflowDefRow>("SELECT * FROM workflow_defs WHERE id = ? LIMIT 1")
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::NotFound {
            entity: "workflow_def",
            id: id.to_string(),
        })
}

pub async fn create_workflow_def(
    db: &SqlitePool,
    input: &CreateWorkflowDefRecord<'_>,
) -> Result<WorkflowDefRow> {
    let id = ids::new_id();
    let now = time::now_ms();
    sqlx::query(
        r#"
        INSERT INTO workflow_defs (
            id, name, description, enabled, sort_order, config_json, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(input.name)
    .bind(input.description)
    .bind(input.enabled)
    .bind(input.sort_order)
    .bind(input.config_json)
    .bind(now)
    .bind(now)
    .execute(db)
    .await?;

    get_workflow_def(db, &id).await
}

pub async fn update_workflow_def(
    db: &SqlitePool,
    id: &str,
    input: &UpdateWorkflowDefRecord<'_>,
) -> Result<WorkflowDefRow> {
    let affected = sqlx::query(
        r#"
        UPDATE workflow_defs
        SET name = ?, description = ?, enabled = ?, sort_order = ?, config_json = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(input.name)
    .bind(input.description)
    .bind(input.enabled)
    .bind(input.sort_order)
    .bind(input.config_json)
    .bind(time::now_ms())
    .bind(id)
    .execute(db)
    .await?
    .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "workflow_def",
            id: id.to_string(),
        });
    }

    get_workflow_def(db, id).await
}

pub async fn delete_workflow_def(db: &SqlitePool, id: &str) -> Result<()> {
    let affected = sqlx::query("DELETE FROM workflow_defs WHERE id = ?")
        .bind(id)
        .execute(db)
        .await?
        .rows_affected();
    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "workflow_def",
            id: id.to_string(),
        });
    }
    Ok(())
}

pub async fn list_workflow_def_nodes(
    db: &SqlitePool,
    workflow_def_id: &str,
) -> Result<Vec<WorkflowDefNodeRow>> {
    sqlx::query_as::<_, WorkflowDefNodeRow>(
        r#"
        SELECT *
        FROM workflow_def_nodes
        WHERE workflow_def_id = ?
        ORDER BY node_key ASC, id ASC
        "#,
    )
    .bind(workflow_def_id)
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

pub async fn get_workflow_def_node(db: &SqlitePool, id: &str) -> Result<WorkflowDefNodeRow> {
    sqlx::query_as::<_, WorkflowDefNodeRow>("SELECT * FROM workflow_def_nodes WHERE id = ? LIMIT 1")
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::NotFound {
            entity: "workflow_def_node",
            id: id.to_string(),
        })
}

pub async fn list_workflow_def_edges(
    db: &SqlitePool,
    workflow_def_id: &str,
) -> Result<Vec<WorkflowDefEdgeRow>> {
    sqlx::query_as::<_, WorkflowDefEdgeRow>(
        r#"
        SELECT *
        FROM workflow_def_edges
        WHERE workflow_def_id = ?
        ORDER BY priority ASC, id ASC
        "#,
    )
    .bind(workflow_def_id)
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

pub async fn replace_workflow_def_nodes(
    tx: &mut Transaction<'_, Sqlite>,
    workflow_def_id: &str,
    items: &[WorkflowDefNodeRecord<'_>],
) -> Result<Vec<WorkflowDefNodeRow>> {
    sqlx::query("DELETE FROM workflow_def_nodes WHERE workflow_def_id = ?")
        .bind(workflow_def_id)
        .execute(tx.as_mut())
        .await?;

    let mut ids_inserted = Vec::with_capacity(items.len());
    for item in items {
        let id = ids::new_id();
        sqlx::query(
            r#"
            INSERT INTO workflow_def_nodes (
                id, workflow_def_id, node_key, name, node_type, agent_id, plugin_id, preset_id,
                lorebook_id, user_profile_id, api_channel_id, api_channel_model_id, workspace_mode,
                history_read_mode, summary_write_mode, message_write_mode, visible_output_mode, config_json
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(workflow_def_id)
        .bind(item.node_key)
        .bind(item.name)
        .bind(item.node_type)
        .bind(item.agent_id)
        .bind(item.plugin_id)
        .bind(item.preset_id)
        .bind(item.lorebook_id)
        .bind(item.user_profile_id)
        .bind(item.api_channel_id)
        .bind(item.api_channel_model_id)
        .bind(item.workspace_mode)
        .bind(item.history_read_mode)
        .bind(item.summary_write_mode)
        .bind(item.message_write_mode)
        .bind(item.visible_output_mode)
        .bind(item.config_json)
        .execute(tx.as_mut())
        .await?;
        ids_inserted.push(id);
    }

    list_workflow_def_nodes_with_executor(tx.as_mut(), workflow_def_id).await
}

pub async fn replace_workflow_def_edges(
    tx: &mut Transaction<'_, Sqlite>,
    workflow_def_id: &str,
    items: &[WorkflowDefEdgeRecord<'_>],
) -> Result<Vec<WorkflowDefEdgeRow>> {
    sqlx::query("DELETE FROM workflow_def_edges WHERE workflow_def_id = ?")
        .bind(workflow_def_id)
        .execute(tx.as_mut())
        .await?;

    for item in items {
        sqlx::query(
            r#"
            INSERT INTO workflow_def_edges (
                id, workflow_def_id, from_node_id, to_node_id, edge_type, priority,
                condition_expr, label, enabled, config_json
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(ids::new_id())
        .bind(workflow_def_id)
        .bind(item.from_node_id)
        .bind(item.to_node_id)
        .bind(item.edge_type)
        .bind(item.priority)
        .bind(item.condition_expr)
        .bind(item.label)
        .bind(item.enabled)
        .bind(item.config_json)
        .execute(tx.as_mut())
        .await?;
    }

    list_workflow_def_edges_with_executor(tx.as_mut(), workflow_def_id).await
}

pub async fn create_workflow_run(
    db: &SqlitePool,
    input: &CreateWorkflowRunRecord<'_>,
) -> Result<WorkflowRunRow> {
    let id = ids::new_id();
    let created_at = time::now_ms();
    sqlx::query(
        r#"
        INSERT INTO workflow_runs (
            id, workflow_def_id, trigger_conversation_id, workspace_conversation_id, workspace_mode,
            trigger_message_version_id, entry_node_id, status, result_message_version_id,
            request_snapshot_content_id, result_content_id, config_json, started_at, finished_at, created_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(input.workflow_def_id)
    .bind(input.trigger_conversation_id)
    .bind(input.workspace_conversation_id)
    .bind(input.workspace_mode)
    .bind(input.trigger_message_version_id)
    .bind(input.entry_node_id)
    .bind(input.status)
    .bind(input.result_message_version_id)
    .bind(input.request_snapshot_content_id)
    .bind(input.result_content_id)
    .bind(input.config_json)
    .bind(input.started_at)
    .bind(input.finished_at)
    .bind(created_at)
    .execute(db)
    .await?;
    get_workflow_run(db, &id).await
}

pub async fn get_workflow_run(db: &SqlitePool, id: &str) -> Result<WorkflowRunRow> {
    sqlx::query_as::<_, WorkflowRunRow>("SELECT * FROM workflow_runs WHERE id = ? LIMIT 1")
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::NotFound {
            entity: "workflow_run",
            id: id.to_string(),
        })
}

pub async fn finish_workflow_run(
    db: &SqlitePool,
    id: &str,
    input: &FinishWorkflowRunRecord<'_>,
) -> Result<WorkflowRunRow> {
    let affected = sqlx::query(
        r#"
        UPDATE workflow_runs
        SET status = ?, result_message_version_id = ?, result_content_id = ?, finished_at = ?, config_json = ?
        WHERE id = ?
        "#,
    )
    .bind(input.status)
    .bind(input.result_message_version_id)
    .bind(input.result_content_id)
    .bind(input.finished_at)
    .bind(input.config_json)
    .bind(id)
    .execute(db)
    .await?
    .rows_affected();
    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "workflow_run",
            id: id.to_string(),
        });
    }
    get_workflow_run(db, id).await
}

pub async fn create_workflow_run_node_execution(
    db: &SqlitePool,
    input: &CreateWorkflowRunNodeExecutionRecord<'_>,
) -> Result<WorkflowRunNodeExecutionRow> {
    let id = ids::new_id();
    let created_at = time::now_ms();
    sqlx::query(
        r#"
        INSERT INTO workflow_run_node_executions (
            id, workflow_run_id, workflow_def_node_id, parent_execution_id, incoming_edge_id,
            branch_key, loop_iteration, retry_index, status, generation_run_id,
            input_snapshot_content_id, output_content_id, error_content_id, started_at,
            finished_at, created_at, config_json
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(input.workflow_run_id)
    .bind(input.workflow_def_node_id)
    .bind(input.parent_execution_id)
    .bind(input.incoming_edge_id)
    .bind(input.branch_key)
    .bind(input.loop_iteration)
    .bind(input.retry_index)
    .bind(input.status)
    .bind(input.generation_run_id)
    .bind(input.input_snapshot_content_id)
    .bind(input.output_content_id)
    .bind(input.error_content_id)
    .bind(input.started_at)
    .bind(input.finished_at)
    .bind(created_at)
    .bind(input.config_json)
    .execute(db)
    .await?;
    get_workflow_run_node_execution(db, &id).await
}

pub async fn get_workflow_run_node_execution(
    db: &SqlitePool,
    id: &str,
) -> Result<WorkflowRunNodeExecutionRow> {
    sqlx::query_as::<_, WorkflowRunNodeExecutionRow>(
        "SELECT * FROM workflow_run_node_executions WHERE id = ? LIMIT 1",
    )
    .bind(id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::NotFound {
        entity: "workflow_run_node_execution",
        id: id.to_string(),
    })
}

pub async fn finish_workflow_run_node_execution(
    db: &SqlitePool,
    id: &str,
    input: &FinishWorkflowRunNodeExecutionRecord<'_>,
) -> Result<WorkflowRunNodeExecutionRow> {
    let affected = sqlx::query(
        r#"
        UPDATE workflow_run_node_executions
        SET status = ?, generation_run_id = ?, output_content_id = ?, error_content_id = ?, finished_at = ?, config_json = ?
        WHERE id = ?
        "#,
    )
    .bind(input.status)
    .bind(input.generation_run_id)
    .bind(input.output_content_id)
    .bind(input.error_content_id)
    .bind(input.finished_at)
    .bind(input.config_json)
    .bind(id)
    .execute(db)
    .await?
    .rows_affected();
    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "workflow_run_node_execution",
            id: id.to_string(),
        });
    }
    get_workflow_run_node_execution(db, id).await
}

pub async fn list_workflow_run_node_executions(
    db: &SqlitePool,
    workflow_run_id: &str,
) -> Result<Vec<WorkflowRunNodeExecutionRow>> {
    sqlx::query_as::<_, WorkflowRunNodeExecutionRow>(
        r#"
        SELECT *
        FROM workflow_run_node_executions
        WHERE workflow_run_id = ?
        ORDER BY created_at ASC, id ASC
        "#,
    )
    .bind(workflow_run_id)
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

pub async fn create_workflow_run_write(
    db: &SqlitePool,
    input: &CreateWorkflowRunWriteRecord<'_>,
) -> Result<WorkflowRunWriteRow> {
    let id = ids::new_id();
    let created_at = time::now_ms();
    sqlx::query(
        r#"
        INSERT INTO workflow_run_writes (
            id, workflow_run_id, workflow_run_node_execution_id, write_kind, apply_mode, content_id,
            target_conversation_id, target_message_node_id, target_summary_group_id,
            target_lorebook_entry_id, target_preset_entry_id, target_agent_id, target_user_profile_id,
            target_plugin_id, target_slot, visible_to_user, created_at, config_json
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(input.workflow_run_id)
    .bind(input.workflow_run_node_execution_id)
    .bind(input.write_kind)
    .bind(input.apply_mode)
    .bind(input.content_id)
    .bind(input.target_conversation_id)
    .bind(input.target_message_node_id)
    .bind(input.target_summary_group_id)
    .bind(input.target_lorebook_entry_id)
    .bind(input.target_preset_entry_id)
    .bind(input.target_agent_id)
    .bind(input.target_user_profile_id)
    .bind(input.target_plugin_id)
    .bind(input.target_slot)
    .bind(input.visible_to_user)
    .bind(created_at)
    .bind(input.config_json)
    .execute(db)
    .await?;
    get_workflow_run_write(db, &id).await
}

pub async fn get_workflow_run_write(db: &SqlitePool, id: &str) -> Result<WorkflowRunWriteRow> {
    sqlx::query_as::<_, WorkflowRunWriteRow>(
        "SELECT * FROM workflow_run_writes WHERE id = ? LIMIT 1",
    )
    .bind(id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::NotFound {
        entity: "workflow_run_write",
        id: id.to_string(),
    })
}

pub async fn list_workflow_run_writes(
    db: &SqlitePool,
    workflow_run_id: &str,
) -> Result<Vec<WorkflowRunWriteRow>> {
    sqlx::query_as::<_, WorkflowRunWriteRow>(
        r#"
        SELECT *
        FROM workflow_run_writes
        WHERE workflow_run_id = ?
        ORDER BY created_at ASC, id ASC
        "#,
    )
    .bind(workflow_run_id)
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

async fn list_workflow_def_nodes_with_executor<'e, E>(
    executor: E,
    workflow_def_id: &str,
) -> Result<Vec<WorkflowDefNodeRow>>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    sqlx::query_as::<_, WorkflowDefNodeRow>(
        r#"
        SELECT *
        FROM workflow_def_nodes
        WHERE workflow_def_id = ?
        ORDER BY node_key ASC, id ASC
        "#,
    )
    .bind(workflow_def_id)
    .fetch_all(executor)
    .await
    .map_err(Into::into)
}

async fn list_workflow_def_edges_with_executor<'e, E>(
    executor: E,
    workflow_def_id: &str,
) -> Result<Vec<WorkflowDefEdgeRow>>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    sqlx::query_as::<_, WorkflowDefEdgeRow>(
        r#"
        SELECT *
        FROM workflow_def_edges
        WHERE workflow_def_id = ?
        ORDER BY priority ASC, id ASC
        "#,
    )
    .bind(workflow_def_id)
    .fetch_all(executor)
    .await
    .map_err(Into::into)
}
