use sqlx::{Sqlite, SqlitePool, Transaction};

use crate::db::models::{SummaryGroupRow, SummarySourceRow, SummaryUsageRow, SummaryVersionRow};
use crate::support::error::{AppError, Result};
use crate::support::{ids, time};

pub struct CreateSummaryGroupRecord<'a> {
    pub conversation_id: &'a str,
    pub scope_type: &'a str,
    pub scope_message_version_id: Option<&'a str>,
    pub scope_start_node_id: Option<&'a str>,
    pub scope_end_node_id: Option<&'a str>,
    pub scope_summary_group_id: Option<&'a str>,
    pub summary_kind: &'a str,
    pub default_generator_preset_id: Option<&'a str>,
    pub enabled: bool,
}

pub struct UpdateSummaryGroupRecord<'a> {
    pub default_generator_preset_id: Option<&'a str>,
    pub enabled: bool,
}

pub struct CreateSummaryVersionRecord<'a> {
    pub summary_group_id: &'a str,
    pub version_index: i64,
    pub is_active: bool,
    pub content_id: &'a str,
    pub generator_type: &'a str,
    pub generator_preset_id: Option<&'a str>,
    pub workflow_run_id: Option<&'a str>,
    pub generation_run_id: Option<&'a str>,
    pub config_json: &'a str,
    pub created_at: i64,
}

pub struct SummarySourceRecord<'a> {
    pub source_kind: &'a str,
    pub source_message_version_id: Option<&'a str>,
    pub source_start_node_id: Option<&'a str>,
    pub source_end_node_id: Option<&'a str>,
    pub source_summary_version_id: Option<&'a str>,
    pub sort_order: i64,
}

pub struct UpsertSummaryUsageRecord<'a> {
    pub usage_id: Option<&'a str>,
    pub summary_group_id: &'a str,
    pub summary_version_id: Option<&'a str>,
    pub usage_scope: &'a str,
    pub target_kind: &'a str,
    pub target_message_version_id: Option<&'a str>,
    pub target_start_node_id: Option<&'a str>,
    pub target_end_node_id: Option<&'a str>,
    pub conversation_id: Option<&'a str>,
    pub activation_mode: &'a str,
    pub replace_from_node_id: Option<&'a str>,
    pub replace_after_message_count: Option<i64>,
    pub replace_after_total_bytes: Option<i64>,
    pub enabled: bool,
    pub priority: i64,
    pub config_json: &'a str,
}

pub async fn list_summary_groups_by_conversation(
    db: &SqlitePool,
    conversation_id: &str,
) -> Result<Vec<SummaryGroupRow>> {
    sqlx::query_as::<_, SummaryGroupRow>(
        r#"
        SELECT *
        FROM summary_groups
        WHERE conversation_id = ?
        ORDER BY enabled DESC, updated_at DESC, created_at DESC
        "#,
    )
    .bind(conversation_id)
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

pub async fn get_summary_group(db: &SqlitePool, id: &str) -> Result<SummaryGroupRow> {
    sqlx::query_as::<_, SummaryGroupRow>("SELECT * FROM summary_groups WHERE id = ? LIMIT 1")
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::NotFound {
            entity: "summary_group",
            id: id.to_string(),
        })
}

pub async fn find_summary_group_by_scope(
    db: &SqlitePool,
    conversation_id: &str,
    scope_type: &str,
    scope_message_version_id: Option<&str>,
    scope_start_node_id: Option<&str>,
    scope_end_node_id: Option<&str>,
    scope_summary_group_id: Option<&str>,
    summary_kind: &str,
) -> Result<Option<SummaryGroupRow>> {
    sqlx::query_as::<_, SummaryGroupRow>(
        r#"
        SELECT *
        FROM summary_groups
        WHERE conversation_id = ?
          AND scope_type = ?
          AND summary_kind = ?
          AND ((scope_message_version_id IS NULL AND ? IS NULL) OR scope_message_version_id = ?)
          AND ((scope_start_node_id IS NULL AND ? IS NULL) OR scope_start_node_id = ?)
          AND ((scope_end_node_id IS NULL AND ? IS NULL) OR scope_end_node_id = ?)
          AND ((scope_summary_group_id IS NULL AND ? IS NULL) OR scope_summary_group_id = ?)
        LIMIT 1
        "#,
    )
    .bind(conversation_id)
    .bind(scope_type)
    .bind(summary_kind)
    .bind(scope_message_version_id)
    .bind(scope_message_version_id)
    .bind(scope_start_node_id)
    .bind(scope_start_node_id)
    .bind(scope_end_node_id)
    .bind(scope_end_node_id)
    .bind(scope_summary_group_id)
    .bind(scope_summary_group_id)
    .fetch_optional(db)
    .await
    .map_err(Into::into)
}

pub async fn create_summary_group(
    db: &SqlitePool,
    input: &CreateSummaryGroupRecord<'_>,
) -> Result<SummaryGroupRow> {
    let id = ids::new_id();
    let now = time::now_ms();

    sqlx::query(
        r#"
        INSERT INTO summary_groups (
            id, conversation_id, scope_type, scope_message_version_id, scope_start_node_id,
            scope_end_node_id, scope_summary_group_id, summary_kind, default_generator_preset_id,
            enabled, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(input.conversation_id)
    .bind(input.scope_type)
    .bind(input.scope_message_version_id)
    .bind(input.scope_start_node_id)
    .bind(input.scope_end_node_id)
    .bind(input.scope_summary_group_id)
    .bind(input.summary_kind)
    .bind(input.default_generator_preset_id)
    .bind(input.enabled)
    .bind(now)
    .bind(now)
    .execute(db)
    .await?;

    get_summary_group(db, &id).await
}

pub async fn update_summary_group(
    db: &SqlitePool,
    id: &str,
    input: &UpdateSummaryGroupRecord<'_>,
) -> Result<SummaryGroupRow> {
    let affected = sqlx::query(
        r#"
        UPDATE summary_groups
        SET default_generator_preset_id = ?, enabled = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(input.default_generator_preset_id)
    .bind(input.enabled)
    .bind(time::now_ms())
    .bind(id)
    .execute(db)
    .await?
    .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "summary_group",
            id: id.to_string(),
        });
    }

    get_summary_group(db, id).await
}

pub async fn get_summary_version(db: &SqlitePool, id: &str) -> Result<SummaryVersionRow> {
    sqlx::query_as::<_, SummaryVersionRow>("SELECT * FROM summary_versions WHERE id = ? LIMIT 1")
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::NotFound {
            entity: "summary_version",
            id: id.to_string(),
        })
}

pub async fn get_active_summary_version(
    db: &SqlitePool,
    summary_group_id: &str,
) -> Result<Option<SummaryVersionRow>> {
    sqlx::query_as::<_, SummaryVersionRow>(
        r#"
        SELECT *
        FROM summary_versions
        WHERE summary_group_id = ? AND is_active = 1
        LIMIT 1
        "#,
    )
    .bind(summary_group_id)
    .fetch_optional(db)
    .await
    .map_err(Into::into)
}

pub async fn create_summary_version(
    tx: &mut Transaction<'_, Sqlite>,
    input: &CreateSummaryVersionRecord<'_>,
) -> Result<SummaryVersionRow> {
    let id = ids::new_id();

    sqlx::query(
        r#"
        INSERT INTO summary_versions (
            id, summary_group_id, version_index, is_active, content_id, generator_type,
            generator_preset_id, workflow_run_id, generation_run_id, config_json, created_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(input.summary_group_id)
    .bind(input.version_index)
    .bind(input.is_active)
    .bind(input.content_id)
    .bind(input.generator_type)
    .bind(input.generator_preset_id)
    .bind(input.workflow_run_id)
    .bind(input.generation_run_id)
    .bind(input.config_json)
    .bind(input.created_at)
    .execute(tx.as_mut())
    .await?;

    get_summary_version_with_executor(tx.as_mut(), &id).await
}

pub async fn list_summary_versions(
    db: &SqlitePool,
    summary_group_id: &str,
) -> Result<Vec<SummaryVersionRow>> {
    sqlx::query_as::<_, SummaryVersionRow>(
        r#"
        SELECT *
        FROM summary_versions
        WHERE summary_group_id = ?
        ORDER BY version_index DESC
        "#,
    )
    .bind(summary_group_id)
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

pub async fn set_active_summary_version(
    tx: &mut Transaction<'_, Sqlite>,
    summary_group_id: &str,
    summary_version_id: &str,
) -> Result<()> {
    let belongs = sqlx::query_scalar::<_, String>(
        "SELECT id FROM summary_versions WHERE id = ? AND summary_group_id = ? LIMIT 1",
    )
    .bind(summary_version_id)
    .bind(summary_group_id)
    .fetch_optional(tx.as_mut())
    .await?;

    if belongs.is_none() {
        return Err(AppError::NotFound {
            entity: "summary_version",
            id: summary_version_id.to_string(),
        });
    }

    sqlx::query("UPDATE summary_versions SET is_active = 0 WHERE summary_group_id = ?")
        .bind(summary_group_id)
        .execute(tx.as_mut())
        .await?;

    sqlx::query("UPDATE summary_versions SET is_active = 1 WHERE id = ?")
        .bind(summary_version_id)
        .execute(tx.as_mut())
        .await?;

    Ok(())
}

pub async fn replace_summary_sources(
    tx: &mut Transaction<'_, Sqlite>,
    summary_group_id: &str,
    summary_version_id: &str,
    items: &[SummarySourceRecord<'_>],
) -> Result<()> {
    sqlx::query("DELETE FROM summary_sources WHERE summary_version_id = ?")
        .bind(summary_version_id)
        .execute(tx.as_mut())
        .await?;

    for item in items {
        sqlx::query(
            r#"
            INSERT INTO summary_sources (
                id, summary_group_id, summary_version_id, source_kind,
                source_message_version_id, source_start_node_id, source_end_node_id,
                source_summary_version_id, sort_order
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(ids::new_id())
        .bind(summary_group_id)
        .bind(summary_version_id)
        .bind(item.source_kind)
        .bind(item.source_message_version_id)
        .bind(item.source_start_node_id)
        .bind(item.source_end_node_id)
        .bind(item.source_summary_version_id)
        .bind(item.sort_order)
        .execute(tx.as_mut())
        .await?;
    }

    Ok(())
}

pub async fn list_summary_sources(
    db: &SqlitePool,
    summary_version_id: &str,
) -> Result<Vec<SummarySourceRow>> {
    sqlx::query_as::<_, SummarySourceRow>(
        r#"
        SELECT *
        FROM summary_sources
        WHERE summary_version_id = ?
        ORDER BY sort_order ASC, id ASC
        "#,
    )
    .bind(summary_version_id)
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

pub async fn upsert_summary_usage(
    db: &SqlitePool,
    input: &UpsertSummaryUsageRecord<'_>,
) -> Result<SummaryUsageRow> {
    if let Some(usage_id) = input.usage_id {
        let affected = sqlx::query(
            r#"
            UPDATE summary_usages
            SET summary_group_id = ?,
                summary_version_id = ?,
                usage_scope = ?,
                target_kind = ?,
                target_message_version_id = ?,
                target_start_node_id = ?,
                target_end_node_id = ?,
                conversation_id = ?,
                activation_mode = ?,
                replace_from_node_id = ?,
                replace_after_message_count = ?,
                replace_after_total_bytes = ?,
                enabled = ?,
                priority = ?,
                config_json = ?,
                updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(input.summary_group_id)
        .bind(input.summary_version_id)
        .bind(input.usage_scope)
        .bind(input.target_kind)
        .bind(input.target_message_version_id)
        .bind(input.target_start_node_id)
        .bind(input.target_end_node_id)
        .bind(input.conversation_id)
        .bind(input.activation_mode)
        .bind(input.replace_from_node_id)
        .bind(input.replace_after_message_count)
        .bind(input.replace_after_total_bytes)
        .bind(input.enabled)
        .bind(input.priority)
        .bind(input.config_json)
        .bind(time::now_ms())
        .bind(usage_id)
        .execute(db)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFound {
                entity: "summary_usage",
                id: usage_id.to_string(),
            });
        }

        return get_summary_usage(db, usage_id).await;
    }

    let id = ids::new_id();
    let now = time::now_ms();
    sqlx::query(
        r#"
        INSERT INTO summary_usages (
            id, summary_group_id, summary_version_id, usage_scope, target_kind,
            target_message_version_id, target_start_node_id, target_end_node_id, conversation_id,
            activation_mode, replace_from_node_id, replace_after_message_count,
            replace_after_total_bytes, enabled, priority, config_json, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(input.summary_group_id)
    .bind(input.summary_version_id)
    .bind(input.usage_scope)
    .bind(input.target_kind)
    .bind(input.target_message_version_id)
    .bind(input.target_start_node_id)
    .bind(input.target_end_node_id)
    .bind(input.conversation_id)
    .bind(input.activation_mode)
    .bind(input.replace_from_node_id)
    .bind(input.replace_after_message_count)
    .bind(input.replace_after_total_bytes)
    .bind(input.enabled)
    .bind(input.priority)
    .bind(input.config_json)
    .bind(now)
    .bind(now)
    .execute(db)
    .await?;

    get_summary_usage(db, &id).await
}

pub async fn list_summary_usages(
    db: &SqlitePool,
    conversation_id: &str,
) -> Result<Vec<SummaryUsageRow>> {
    sqlx::query_as::<_, SummaryUsageRow>(
        r#"
        SELECT *
        FROM summary_usages
        WHERE conversation_id = ?
        ORDER BY enabled DESC, priority DESC, updated_at DESC, created_at DESC
        "#,
    )
    .bind(conversation_id)
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

pub async fn get_summary_usage(db: &SqlitePool, id: &str) -> Result<SummaryUsageRow> {
    sqlx::query_as::<_, SummaryUsageRow>("SELECT * FROM summary_usages WHERE id = ? LIMIT 1")
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::NotFound {
            entity: "summary_usage",
            id: id.to_string(),
        })
}

pub async fn delete_summary_usage(db: &SqlitePool, id: &str) -> Result<()> {
    let affected = sqlx::query("DELETE FROM summary_usages WHERE id = ?")
        .bind(id)
        .execute(db)
        .await?
        .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "summary_usage",
            id: id.to_string(),
        });
    }

    Ok(())
}

async fn get_summary_version_with_executor<'e, E>(
    executor: E,
    id: &str,
) -> Result<SummaryVersionRow>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    sqlx::query_as::<_, SummaryVersionRow>("SELECT * FROM summary_versions WHERE id = ? LIMIT 1")
        .bind(id)
        .fetch_optional(executor)
        .await?
        .ok_or_else(|| AppError::NotFound {
            entity: "summary_version",
            id: id.to_string(),
        })
}
