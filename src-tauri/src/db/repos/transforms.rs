use sqlx::{Sqlite, SqlitePool, Transaction};

use crate::db::models::{TransformBindingRow, TransformPipelineRow, TransformStepRow};
use crate::support::error::{AppError, Result};
use crate::support::{ids, time};

pub struct CreateTransformPipelineRecord<'a> {
    pub name: &'a str,
    pub pipeline_key: &'a str,
    pub pipeline_kind: &'a str,
    pub description_content_id: Option<&'a str>,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: &'a str,
}

pub struct UpdateTransformPipelineRecord<'a> {
    pub name: &'a str,
    pub pipeline_kind: &'a str,
    pub description_content_id: Option<&'a str>,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: &'a str,
}

pub struct TransformStepRecord<'a> {
    pub step_order: i64,
    pub step_type: &'a str,
    pub pattern: Option<&'a str>,
    pub replacement_template: Option<&'a str>,
    pub regex_flags: &'a str,
    pub max_replacements: Option<i64>,
    pub stop_on_match: bool,
    pub child_pipeline_id: Option<&'a str>,
    pub config_json: &'a str,
}

pub struct CreateOrUpdateTransformBindingRecord<'a> {
    pub pipeline_id: &'a str,
    pub conversation_id: Option<&'a str>,
    pub agent_id: Option<&'a str>,
    pub preset_id: Option<&'a str>,
    pub workflow_def_node_id: Option<&'a str>,
    pub apply_viewer: bool,
    pub apply_request: bool,
    pub apply_file: bool,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: &'a str,
}

pub async fn list_transform_pipelines(db: &SqlitePool) -> Result<Vec<TransformPipelineRow>> {
    sqlx::query_as::<_, TransformPipelineRow>(
        r#"
        SELECT *
        FROM transform_pipelines
        ORDER BY enabled DESC, sort_order ASC, created_at ASC
        "#,
    )
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

pub async fn get_transform_pipeline(db: &SqlitePool, id: &str) -> Result<TransformPipelineRow> {
    get_transform_pipeline_with_executor(db, id).await
}

pub async fn create_transform_pipeline(
    db: &SqlitePool,
    input: &CreateTransformPipelineRecord<'_>,
) -> Result<TransformPipelineRow> {
    let id = ids::new_id();
    let now = time::now_ms();

    sqlx::query(
        r#"
        INSERT INTO transform_pipelines (
            id, name, pipeline_key, pipeline_kind, description_content_id,
            enabled, sort_order, config_json, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(input.name)
    .bind(input.pipeline_key)
    .bind(input.pipeline_kind)
    .bind(input.description_content_id)
    .bind(input.enabled)
    .bind(input.sort_order)
    .bind(input.config_json)
    .bind(now)
    .bind(now)
    .execute(db)
    .await?;

    get_transform_pipeline(db, &id).await
}

pub async fn update_transform_pipeline(
    db: &SqlitePool,
    id: &str,
    input: &UpdateTransformPipelineRecord<'_>,
) -> Result<TransformPipelineRow> {
    let affected = sqlx::query(
        r#"
        UPDATE transform_pipelines
        SET name = ?,
            pipeline_kind = ?,
            description_content_id = ?,
            enabled = ?,
            sort_order = ?,
            config_json = ?,
            updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(input.name)
    .bind(input.pipeline_kind)
    .bind(input.description_content_id)
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
            entity: "transform_pipeline",
            id: id.to_string(),
        });
    }

    get_transform_pipeline(db, id).await
}

pub async fn delete_transform_pipeline(db: &SqlitePool, id: &str) -> Result<()> {
    let affected = sqlx::query("DELETE FROM transform_pipelines WHERE id = ?")
        .bind(id)
        .execute(db)
        .await?
        .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "transform_pipeline",
            id: id.to_string(),
        });
    }

    Ok(())
}

pub async fn replace_transform_steps(
    tx: &mut Transaction<'_, Sqlite>,
    pipeline_id: &str,
    items: &[TransformStepRecord<'_>],
) -> Result<Vec<TransformStepRow>> {
    sqlx::query("DELETE FROM transform_steps WHERE pipeline_id = ?")
        .bind(pipeline_id)
        .execute(tx.as_mut())
        .await?;

    for item in items {
        sqlx::query(
            r#"
            INSERT INTO transform_steps (
                id, pipeline_id, step_order, step_type, pattern,
                replacement_template, regex_flags, max_replacements,
                stop_on_match, child_pipeline_id, config_json
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(ids::new_id())
        .bind(pipeline_id)
        .bind(item.step_order)
        .bind(item.step_type)
        .bind(item.pattern)
        .bind(item.replacement_template)
        .bind(item.regex_flags)
        .bind(item.max_replacements)
        .bind(item.stop_on_match)
        .bind(item.child_pipeline_id)
        .bind(item.config_json)
        .execute(tx.as_mut())
        .await?;
    }

    list_transform_steps_with_executor(tx.as_mut(), pipeline_id).await
}

pub async fn list_transform_steps(
    db: &SqlitePool,
    pipeline_id: &str,
) -> Result<Vec<TransformStepRow>> {
    list_transform_steps_with_executor(db, pipeline_id).await
}

pub async fn list_transform_bindings(db: &SqlitePool) -> Result<Vec<TransformBindingRow>> {
    sqlx::query_as::<_, TransformBindingRow>(
        r#"
        SELECT *
        FROM transform_bindings
        ORDER BY enabled DESC, sort_order ASC, created_at ASC
        "#,
    )
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

pub async fn get_transform_binding(db: &SqlitePool, id: &str) -> Result<TransformBindingRow> {
    sqlx::query_as::<_, TransformBindingRow>(
        "SELECT * FROM transform_bindings WHERE id = ? LIMIT 1",
    )
    .bind(id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::NotFound {
        entity: "transform_binding",
        id: id.to_string(),
    })
}

pub async fn create_transform_binding(
    db: &SqlitePool,
    input: &CreateOrUpdateTransformBindingRecord<'_>,
) -> Result<TransformBindingRow> {
    let id = ids::new_id();
    let now = time::now_ms();

    sqlx::query(
        r#"
        INSERT INTO transform_bindings (
            id, pipeline_id, conversation_id, agent_id, preset_id, workflow_def_node_id,
            apply_viewer, apply_request, apply_file, enabled, sort_order, config_json,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(input.pipeline_id)
    .bind(input.conversation_id)
    .bind(input.agent_id)
    .bind(input.preset_id)
    .bind(input.workflow_def_node_id)
    .bind(input.apply_viewer)
    .bind(input.apply_request)
    .bind(input.apply_file)
    .bind(input.enabled)
    .bind(input.sort_order)
    .bind(input.config_json)
    .bind(now)
    .bind(now)
    .execute(db)
    .await?;

    get_transform_binding(db, &id).await
}

pub async fn update_transform_binding(
    db: &SqlitePool,
    id: &str,
    input: &CreateOrUpdateTransformBindingRecord<'_>,
) -> Result<TransformBindingRow> {
    let affected = sqlx::query(
        r#"
        UPDATE transform_bindings
        SET pipeline_id = ?,
            conversation_id = ?,
            agent_id = ?,
            preset_id = ?,
            workflow_def_node_id = ?,
            apply_viewer = ?,
            apply_request = ?,
            apply_file = ?,
            enabled = ?,
            sort_order = ?,
            config_json = ?,
            updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(input.pipeline_id)
    .bind(input.conversation_id)
    .bind(input.agent_id)
    .bind(input.preset_id)
    .bind(input.workflow_def_node_id)
    .bind(input.apply_viewer)
    .bind(input.apply_request)
    .bind(input.apply_file)
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
            entity: "transform_binding",
            id: id.to_string(),
        });
    }

    get_transform_binding(db, id).await
}

pub async fn delete_transform_binding(db: &SqlitePool, id: &str) -> Result<()> {
    let affected = sqlx::query("DELETE FROM transform_bindings WHERE id = ?")
        .bind(id)
        .execute(db)
        .await?
        .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "transform_binding",
            id: id.to_string(),
        });
    }

    Ok(())
}

pub async fn list_matching_transform_bindings(
    db: &SqlitePool,
    stage: &str,
    conversation_id: Option<&str>,
    agent_id: Option<&str>,
    preset_id: Option<&str>,
    workflow_def_node_id: Option<&str>,
) -> Result<Vec<TransformBindingRow>> {
    let mut rows = Vec::new();

    if let Some(conversation_id) = conversation_id {
        rows.extend(list_bindings_for_scope(db, "conversation", stage, conversation_id).await?);
    }
    if let Some(agent_id) = agent_id {
        rows.extend(list_bindings_for_scope(db, "agent", stage, agent_id).await?);
    }
    if let Some(preset_id) = preset_id {
        rows.extend(list_bindings_for_scope(db, "preset", stage, preset_id).await?);
    }
    if let Some(workflow_def_node_id) = workflow_def_node_id {
        rows.extend(
            list_bindings_for_scope(db, "workflow_def_node", stage, workflow_def_node_id).await?,
        );
    }

    rows.sort_by(|left, right| {
        left.sort_order
            .cmp(&right.sort_order)
            .then(left.created_at.cmp(&right.created_at))
            .then(left.id.cmp(&right.id))
    });
    Ok(rows)
}

async fn list_bindings_for_scope(
    db: &SqlitePool,
    scope: &str,
    stage: &str,
    scope_id: &str,
) -> Result<Vec<TransformBindingRow>> {
    let sql = match (scope, stage) {
        ("conversation", "viewer") => {
            "SELECT * FROM transform_bindings WHERE conversation_id = ? AND enabled = 1 AND apply_viewer = 1 ORDER BY sort_order ASC, created_at ASC"
        }
        ("conversation", "request") => {
            "SELECT * FROM transform_bindings WHERE conversation_id = ? AND enabled = 1 AND apply_request = 1 ORDER BY sort_order ASC, created_at ASC"
        }
        ("conversation", "file") => {
            "SELECT * FROM transform_bindings WHERE conversation_id = ? AND enabled = 1 AND apply_file = 1 ORDER BY sort_order ASC, created_at ASC"
        }
        ("agent", "viewer") => {
            "SELECT * FROM transform_bindings WHERE agent_id = ? AND enabled = 1 AND apply_viewer = 1 ORDER BY sort_order ASC, created_at ASC"
        }
        ("agent", "request") => {
            "SELECT * FROM transform_bindings WHERE agent_id = ? AND enabled = 1 AND apply_request = 1 ORDER BY sort_order ASC, created_at ASC"
        }
        ("agent", "file") => {
            "SELECT * FROM transform_bindings WHERE agent_id = ? AND enabled = 1 AND apply_file = 1 ORDER BY sort_order ASC, created_at ASC"
        }
        ("preset", "viewer") => {
            "SELECT * FROM transform_bindings WHERE preset_id = ? AND enabled = 1 AND apply_viewer = 1 ORDER BY sort_order ASC, created_at ASC"
        }
        ("preset", "request") => {
            "SELECT * FROM transform_bindings WHERE preset_id = ? AND enabled = 1 AND apply_request = 1 ORDER BY sort_order ASC, created_at ASC"
        }
        ("preset", "file") => {
            "SELECT * FROM transform_bindings WHERE preset_id = ? AND enabled = 1 AND apply_file = 1 ORDER BY sort_order ASC, created_at ASC"
        }
        ("workflow_def_node", "viewer") => {
            "SELECT * FROM transform_bindings WHERE workflow_def_node_id = ? AND enabled = 1 AND apply_viewer = 1 ORDER BY sort_order ASC, created_at ASC"
        }
        ("workflow_def_node", "request") => {
            "SELECT * FROM transform_bindings WHERE workflow_def_node_id = ? AND enabled = 1 AND apply_request = 1 ORDER BY sort_order ASC, created_at ASC"
        }
        ("workflow_def_node", "file") => {
            "SELECT * FROM transform_bindings WHERE workflow_def_node_id = ? AND enabled = 1 AND apply_file = 1 ORDER BY sort_order ASC, created_at ASC"
        }
        _ => {
            return Err(AppError::Validation(format!(
                "unsupported transform scope/stage combination '{scope}:{stage}'"
            )))
        }
    };

    sqlx::query_as::<_, TransformBindingRow>(sql)
        .bind(scope_id)
        .fetch_all(db)
        .await
        .map_err(Into::into)
}

async fn get_transform_pipeline_with_executor<'e, E>(
    executor: E,
    id: &str,
) -> Result<TransformPipelineRow>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    sqlx::query_as::<_, TransformPipelineRow>(
        "SELECT * FROM transform_pipelines WHERE id = ? LIMIT 1",
    )
    .bind(id)
    .fetch_optional(executor)
    .await?
    .ok_or_else(|| AppError::NotFound {
        entity: "transform_pipeline",
        id: id.to_string(),
    })
}

async fn list_transform_steps_with_executor<'e, E>(
    executor: E,
    pipeline_id: &str,
) -> Result<Vec<TransformStepRow>>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    sqlx::query_as::<_, TransformStepRow>(
        r#"
        SELECT *
        FROM transform_steps
        WHERE pipeline_id = ?
        ORDER BY step_order ASC
        "#,
    )
    .bind(pipeline_id)
    .fetch_all(executor)
    .await
    .map_err(Into::into)
}
