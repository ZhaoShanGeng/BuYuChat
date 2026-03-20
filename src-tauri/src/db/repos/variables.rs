use sqlx::{Sqlite, SqlitePool, Transaction};

use crate::db::models::{VariableDefRow, VariableEventRow, VariableLockRow, VariableValueRow};
use crate::support::error::{AppError, Result};
use crate::support::{ids, time};

pub struct CreateVariableDefRecord<'a> {
    pub var_key: &'a str,
    pub name: &'a str,
    pub value_type: &'a str,
    pub scope_type: &'a str,
    pub namespace: &'a str,
    pub is_user_editable: bool,
    pub is_plugin_editable: bool,
    pub ai_can_create: bool,
    pub ai_can_update: bool,
    pub ai_can_delete: bool,
    pub ai_can_lock: bool,
    pub ai_can_unlock_own_lock: bool,
    pub ai_can_unlock_any_lock: bool,
    pub default_json: &'a str,
    pub config_json: &'a str,
}

pub struct UpdateVariableDefRecord<'a> {
    pub name: &'a str,
    pub namespace: &'a str,
    pub is_user_editable: bool,
    pub is_plugin_editable: bool,
    pub ai_can_create: bool,
    pub ai_can_update: bool,
    pub ai_can_delete: bool,
    pub ai_can_lock: bool,
    pub ai_can_unlock_own_lock: bool,
    pub ai_can_unlock_any_lock: bool,
    pub default_json: &'a str,
    pub config_json: &'a str,
}

pub struct UpsertVariableValueRecord<'a> {
    pub variable_def_id: &'a str,
    pub scope_type: &'a str,
    pub scope_id: &'a str,
    pub value_json: &'a str,
    pub value_content_id: Option<&'a str>,
    pub source_message_version_id: Option<&'a str>,
    pub updated_by_kind: &'a str,
    pub updated_by_ref_id: Option<&'a str>,
}

pub struct RecordVariableEvent<'a> {
    pub variable_value_id: &'a str,
    pub event_no: i64,
    pub event_kind: &'a str,
    pub value_json: &'a str,
    pub value_content_id: Option<&'a str>,
    pub source_message_version_id: Option<&'a str>,
    pub updated_by_kind: &'a str,
    pub updated_by_ref_id: Option<&'a str>,
    pub config_json: &'a str,
}

pub struct UpsertVariableLockRecord<'a> {
    pub variable_value_id: &'a str,
    pub lock_kind: &'a str,
    pub owner_kind: &'a str,
    pub owner_ref_id: Option<&'a str>,
    pub unlock_policy: &'a str,
    pub config_json: &'a str,
}

pub async fn list_variable_defs(db: &SqlitePool) -> Result<Vec<VariableDefRow>> {
    sqlx::query_as::<_, VariableDefRow>(
        r#"
        SELECT *
        FROM variable_defs
        ORDER BY namespace ASC, scope_type ASC, name ASC, created_at ASC
        "#,
    )
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

pub async fn get_variable_def(db: &SqlitePool, id: &str) -> Result<VariableDefRow> {
    sqlx::query_as::<_, VariableDefRow>("SELECT * FROM variable_defs WHERE id = ? LIMIT 1")
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::NotFound {
            entity: "variable_def",
            id: id.to_string(),
        })
}

pub async fn get_variable_def_by_key(
    db: &SqlitePool,
    var_key: &str,
) -> Result<Option<VariableDefRow>> {
    sqlx::query_as::<_, VariableDefRow>("SELECT * FROM variable_defs WHERE var_key = ? LIMIT 1")
        .bind(var_key)
        .fetch_optional(db)
        .await
        .map_err(Into::into)
}

pub async fn create_variable_def(
    db: &SqlitePool,
    input: &CreateVariableDefRecord<'_>,
) -> Result<VariableDefRow> {
    let id = ids::new_id();
    let now = time::now_ms();
    sqlx::query(
        r#"
        INSERT INTO variable_defs (
            id, var_key, name, value_type, scope_type, namespace,
            is_user_editable, is_plugin_editable,
            ai_can_create, ai_can_update, ai_can_delete,
            ai_can_lock, ai_can_unlock_own_lock, ai_can_unlock_any_lock,
            default_json, config_json, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(input.var_key)
    .bind(input.name)
    .bind(input.value_type)
    .bind(input.scope_type)
    .bind(input.namespace)
    .bind(input.is_user_editable)
    .bind(input.is_plugin_editable)
    .bind(input.ai_can_create)
    .bind(input.ai_can_update)
    .bind(input.ai_can_delete)
    .bind(input.ai_can_lock)
    .bind(input.ai_can_unlock_own_lock)
    .bind(input.ai_can_unlock_any_lock)
    .bind(input.default_json)
    .bind(input.config_json)
    .bind(now)
    .bind(now)
    .execute(db)
    .await?;

    get_variable_def(db, &id).await
}

pub async fn update_variable_def(
    db: &SqlitePool,
    id: &str,
    input: &UpdateVariableDefRecord<'_>,
) -> Result<VariableDefRow> {
    let updated_at = time::now_ms();
    let result = sqlx::query(
        r#"
        UPDATE variable_defs
        SET name = ?,
            namespace = ?,
            is_user_editable = ?,
            is_plugin_editable = ?,
            ai_can_create = ?,
            ai_can_update = ?,
            ai_can_delete = ?,
            ai_can_lock = ?,
            ai_can_unlock_own_lock = ?,
            ai_can_unlock_any_lock = ?,
            default_json = ?,
            config_json = ?,
            updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(input.name)
    .bind(input.namespace)
    .bind(input.is_user_editable)
    .bind(input.is_plugin_editable)
    .bind(input.ai_can_create)
    .bind(input.ai_can_update)
    .bind(input.ai_can_delete)
    .bind(input.ai_can_lock)
    .bind(input.ai_can_unlock_own_lock)
    .bind(input.ai_can_unlock_any_lock)
    .bind(input.default_json)
    .bind(input.config_json)
    .bind(updated_at)
    .bind(id)
    .execute(db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound {
            entity: "variable_def",
            id: id.to_string(),
        });
    }

    get_variable_def(db, id).await
}

pub async fn delete_variable_def(db: &mut Transaction<'_, Sqlite>, id: &str) -> Result<()> {
    let result = sqlx::query("DELETE FROM variable_defs WHERE id = ?")
        .bind(id)
        .execute(db.as_mut())
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound {
            entity: "variable_def",
            id: id.to_string(),
        });
    }
    Ok(())
}

pub async fn delete_variable_values_by_def(
    db: &mut Transaction<'_, Sqlite>,
    variable_def_id: &str,
) -> Result<()> {
    sqlx::query(
        "DELETE FROM variable_events WHERE variable_value_id IN (SELECT id FROM variable_values WHERE variable_def_id = ?)",
    )
    .bind(variable_def_id)
    .execute(db.as_mut())
    .await?;
    sqlx::query(
        "DELETE FROM variable_locks WHERE variable_value_id IN (SELECT id FROM variable_values WHERE variable_def_id = ?)",
    )
    .bind(variable_def_id)
    .execute(db.as_mut())
    .await?;
    sqlx::query("DELETE FROM variable_values WHERE variable_def_id = ?")
        .bind(variable_def_id)
        .execute(db.as_mut())
        .await?;
    Ok(())
}

pub async fn list_variable_values_by_scope(
    db: &SqlitePool,
    scope_type: &str,
    scope_id: &str,
    include_deleted: bool,
) -> Result<Vec<VariableValueRow>> {
    if include_deleted {
        sqlx::query_as::<_, VariableValueRow>(
            r#"
            SELECT *
            FROM variable_values
            WHERE scope_type = ? AND scope_id = ?
            ORDER BY updated_at DESC, created_at DESC
            "#,
        )
        .bind(scope_type)
        .bind(scope_id)
        .fetch_all(db)
        .await
        .map_err(Into::into)
    } else {
        sqlx::query_as::<_, VariableValueRow>(
            r#"
            SELECT *
            FROM variable_values
            WHERE scope_type = ? AND scope_id = ? AND is_deleted = 0
            ORDER BY updated_at DESC, created_at DESC
            "#,
        )
        .bind(scope_type)
        .bind(scope_id)
        .fetch_all(db)
        .await
        .map_err(Into::into)
    }
}

pub async fn get_variable_value(
    db: &SqlitePool,
    variable_def_id: &str,
    scope_type: &str,
    scope_id: &str,
    include_deleted: bool,
) -> Result<Option<VariableValueRow>> {
    let sql = if include_deleted {
        r#"
        SELECT *
        FROM variable_values
        WHERE variable_def_id = ? AND scope_type = ? AND scope_id = ?
        LIMIT 1
        "#
    } else {
        r#"
        SELECT *
        FROM variable_values
        WHERE variable_def_id = ? AND scope_type = ? AND scope_id = ? AND is_deleted = 0
        LIMIT 1
        "#
    };

    sqlx::query_as::<_, VariableValueRow>(sql)
        .bind(variable_def_id)
        .bind(scope_type)
        .bind(scope_id)
        .fetch_optional(db)
        .await
        .map_err(Into::into)
}

pub async fn upsert_variable_value(
    tx: &mut Transaction<'_, Sqlite>,
    input: &UpsertVariableValueRecord<'_>,
) -> Result<VariableValueRow> {
    let existing = get_variable_value_with_executor(
        tx.as_mut(),
        input.variable_def_id,
        input.scope_type,
        input.scope_id,
        true,
    )
    .await?;

    match existing {
        Some(existing) => {
            let updated_at = time::now_ms();
            let next_event = existing.event_no + 1;
            sqlx::query(
                r#"
                UPDATE variable_values
                SET value_json = ?,
                    value_content_id = ?,
                    source_message_version_id = ?,
                    updated_by_kind = ?,
                    updated_by_ref_id = ?,
                    event_no = ?,
                    is_deleted = 0,
                    updated_at = ?
                WHERE id = ?
                "#,
            )
            .bind(input.value_json)
            .bind(input.value_content_id)
            .bind(input.source_message_version_id)
            .bind(input.updated_by_kind)
            .bind(input.updated_by_ref_id)
            .bind(next_event)
            .bind(updated_at)
            .bind(&existing.id)
            .execute(tx.as_mut())
            .await?;

            let updated = get_variable_value_with_executor(
                tx.as_mut(),
                input.variable_def_id,
                input.scope_type,
                input.scope_id,
                true,
            )
            .await?
            .ok_or_else(|| {
                AppError::Other("updated variable value missing after upsert".to_string())
            })?;

            record_variable_event(
                tx,
                &RecordVariableEvent {
                    variable_value_id: &updated.id,
                    event_no: updated.event_no,
                    event_kind: "set",
                    value_json: &updated.value_json,
                    value_content_id: updated.value_content_id.as_deref(),
                    source_message_version_id: updated.source_message_version_id.as_deref(),
                    updated_by_kind: &updated.updated_by_kind,
                    updated_by_ref_id: updated.updated_by_ref_id.as_deref(),
                    config_json: "{}",
                },
            )
            .await?;

            Ok(updated)
        }
        None => {
            let id = ids::new_id();
            let now = time::now_ms();
            sqlx::query(
                r#"
                INSERT INTO variable_values (
                    id, variable_def_id, scope_type, scope_id, value_json, value_content_id,
                    source_message_version_id, updated_by_kind, updated_by_ref_id,
                    event_no, is_deleted, created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 1, 0, ?, ?)
                "#,
            )
            .bind(&id)
            .bind(input.variable_def_id)
            .bind(input.scope_type)
            .bind(input.scope_id)
            .bind(input.value_json)
            .bind(input.value_content_id)
            .bind(input.source_message_version_id)
            .bind(input.updated_by_kind)
            .bind(input.updated_by_ref_id)
            .bind(now)
            .bind(now)
            .execute(tx.as_mut())
            .await?;

            let inserted = get_variable_value_with_executor(
                tx.as_mut(),
                input.variable_def_id,
                input.scope_type,
                input.scope_id,
                true,
            )
            .await?
            .ok_or_else(|| {
                AppError::Other("inserted variable value missing after upsert".to_string())
            })?;

            record_variable_event(
                tx,
                &RecordVariableEvent {
                    variable_value_id: &inserted.id,
                    event_no: inserted.event_no,
                    event_kind: "set",
                    value_json: &inserted.value_json,
                    value_content_id: inserted.value_content_id.as_deref(),
                    source_message_version_id: inserted.source_message_version_id.as_deref(),
                    updated_by_kind: &inserted.updated_by_kind,
                    updated_by_ref_id: inserted.updated_by_ref_id.as_deref(),
                    config_json: "{}",
                },
            )
            .await?;

            Ok(inserted)
        }
    }
}

pub async fn mark_variable_value_deleted(
    tx: &mut Transaction<'_, Sqlite>,
    variable_value_id: &str,
    source_message_version_id: Option<&str>,
    updated_by_kind: &str,
    updated_by_ref_id: Option<&str>,
) -> Result<VariableValueRow> {
    let existing = get_variable_value_by_id_with_executor(tx.as_mut(), variable_value_id).await?;
    let existing = existing.ok_or_else(|| AppError::NotFound {
        entity: "variable_value",
        id: variable_value_id.to_string(),
    })?;

    let updated_at = time::now_ms();
    let next_event = existing.event_no + 1;
    sqlx::query(
        r#"
        UPDATE variable_values
        SET value_json = 'null',
            value_content_id = NULL,
            source_message_version_id = ?,
            updated_by_kind = ?,
            updated_by_ref_id = ?,
            event_no = ?,
            is_deleted = 1,
            updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(source_message_version_id)
    .bind(updated_by_kind)
    .bind(updated_by_ref_id)
    .bind(next_event)
    .bind(updated_at)
    .bind(variable_value_id)
    .execute(tx.as_mut())
    .await?;

    let updated = get_variable_value_by_id_with_executor(tx.as_mut(), variable_value_id)
        .await?
        .ok_or_else(|| {
            AppError::Other("deleted variable value missing after delete mark".to_string())
        })?;

    record_variable_event(
        tx,
        &RecordVariableEvent {
            variable_value_id: &updated.id,
            event_no: updated.event_no,
            event_kind: "delete",
            value_json: &updated.value_json,
            value_content_id: None,
            source_message_version_id: updated.source_message_version_id.as_deref(),
            updated_by_kind: &updated.updated_by_kind,
            updated_by_ref_id: updated.updated_by_ref_id.as_deref(),
            config_json: "{}",
        },
    )
    .await?;

    Ok(updated)
}

pub async fn restore_variable_value_event(
    tx: &mut Transaction<'_, Sqlite>,
    variable_value_id: &str,
    event_id: &str,
    updated_by_kind: &str,
    updated_by_ref_id: Option<&str>,
) -> Result<VariableValueRow> {
    let target = get_variable_event(tx.as_mut(), event_id).await?;
    if target.variable_value_id != variable_value_id {
        return Err(AppError::Validation(
            "variable event does not belong to target variable value".to_string(),
        ));
    }

    let existing = get_variable_value_by_id_with_executor(tx.as_mut(), variable_value_id)
        .await?
        .ok_or_else(|| AppError::NotFound {
            entity: "variable_value",
            id: variable_value_id.to_string(),
        })?;

    let updated_at = time::now_ms();
    let next_event = existing.event_no + 1;
    let is_deleted = target.event_kind == "delete";
    sqlx::query(
        r#"
        UPDATE variable_values
        SET value_json = ?,
            value_content_id = ?,
            source_message_version_id = ?,
            updated_by_kind = ?,
            updated_by_ref_id = ?,
            event_no = ?,
            is_deleted = ?,
            updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(&target.value_json)
    .bind(target.value_content_id.as_deref())
    .bind(target.source_message_version_id.as_deref())
    .bind(updated_by_kind)
    .bind(updated_by_ref_id)
    .bind(next_event)
    .bind(is_deleted)
    .bind(updated_at)
    .bind(variable_value_id)
    .execute(tx.as_mut())
    .await?;

    let updated = get_variable_value_by_id_with_executor(tx.as_mut(), variable_value_id)
        .await?
        .ok_or_else(|| {
            AppError::Other("restored variable value missing after restore".to_string())
        })?;

    record_variable_event(
        tx,
        &RecordVariableEvent {
            variable_value_id: &updated.id,
            event_no: updated.event_no,
            event_kind: "restore",
            value_json: &updated.value_json,
            value_content_id: updated.value_content_id.as_deref(),
            source_message_version_id: updated.source_message_version_id.as_deref(),
            updated_by_kind,
            updated_by_ref_id,
            config_json: "{}",
        },
    )
    .await?;

    Ok(updated)
}

pub async fn list_variable_events(
    db: &SqlitePool,
    variable_value_id: &str,
) -> Result<Vec<VariableEventRow>> {
    sqlx::query_as::<_, VariableEventRow>(
        r#"
        SELECT *
        FROM variable_events
        WHERE variable_value_id = ?
        ORDER BY event_no DESC, created_at DESC
        "#,
    )
    .bind(variable_value_id)
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

pub async fn get_variable_event(
    db: impl sqlx::Executor<'_, Database = Sqlite>,
    id: &str,
) -> Result<VariableEventRow> {
    sqlx::query_as::<_, VariableEventRow>("SELECT * FROM variable_events WHERE id = ? LIMIT 1")
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::NotFound {
            entity: "variable_event",
            id: id.to_string(),
        })
}

pub async fn list_active_variable_locks(
    db: &SqlitePool,
    variable_value_id: &str,
) -> Result<Vec<VariableLockRow>> {
    sqlx::query_as::<_, VariableLockRow>(
        r#"
        SELECT *
        FROM variable_locks
        WHERE variable_value_id = ? AND active = 1
        ORDER BY created_at ASC
        "#,
    )
    .bind(variable_value_id)
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

pub async fn get_variable_lock(db: &SqlitePool, id: &str) -> Result<VariableLockRow> {
    sqlx::query_as::<_, VariableLockRow>("SELECT * FROM variable_locks WHERE id = ? LIMIT 1")
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::NotFound {
            entity: "variable_lock",
            id: id.to_string(),
        })
}

pub async fn upsert_variable_lock(
    tx: &mut Transaction<'_, Sqlite>,
    input: &UpsertVariableLockRecord<'_>,
) -> Result<VariableLockRow> {
    let existing = sqlx::query_as::<_, VariableLockRow>(
        r#"
        SELECT *
        FROM variable_locks
        WHERE variable_value_id = ? AND lock_kind = ? AND active = 1
        LIMIT 1
        "#,
    )
    .bind(input.variable_value_id)
    .bind(input.lock_kind)
    .fetch_optional(tx.as_mut())
    .await?;

    match existing {
        Some(existing) => {
            let updated_at = time::now_ms();
            sqlx::query(
                r#"
                UPDATE variable_locks
                SET owner_kind = ?,
                    owner_ref_id = ?,
                    unlock_policy = ?,
                    config_json = ?,
                    updated_at = ?
                WHERE id = ?
                "#,
            )
            .bind(input.owner_kind)
            .bind(input.owner_ref_id)
            .bind(input.unlock_policy)
            .bind(input.config_json)
            .bind(updated_at)
            .bind(&existing.id)
            .execute(tx.as_mut())
            .await?;
            get_variable_lock_with_executor(tx.as_mut(), &existing.id).await
        }
        None => {
            let id = ids::new_id();
            let now = time::now_ms();
            sqlx::query(
                r#"
                INSERT INTO variable_locks (
                    id, variable_value_id, lock_kind, owner_kind, owner_ref_id,
                    unlock_policy, active, config_json, created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, 1, ?, ?, ?)
                "#,
            )
            .bind(&id)
            .bind(input.variable_value_id)
            .bind(input.lock_kind)
            .bind(input.owner_kind)
            .bind(input.owner_ref_id)
            .bind(input.unlock_policy)
            .bind(input.config_json)
            .bind(now)
            .bind(now)
            .execute(tx.as_mut())
            .await?;
            get_variable_lock_with_executor(tx.as_mut(), &id).await
        }
    }
}

pub async fn deactivate_variable_lock(
    tx: &mut Transaction<'_, Sqlite>,
    id: &str,
) -> Result<VariableLockRow> {
    let updated_at = time::now_ms();
    let result = sqlx::query(
        r#"
        UPDATE variable_locks
        SET active = 0, updated_at = ?
        WHERE id = ? AND active = 1
        "#,
    )
    .bind(updated_at)
    .bind(id)
    .execute(tx.as_mut())
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound {
            entity: "variable_lock",
            id: id.to_string(),
        });
    }

    get_variable_lock_with_executor(tx.as_mut(), id).await
}

async fn record_variable_event(
    tx: &mut Transaction<'_, Sqlite>,
    input: &RecordVariableEvent<'_>,
) -> Result<VariableEventRow> {
    let id = ids::new_id();
    let created_at = time::now_ms();
    sqlx::query(
        r#"
        INSERT INTO variable_events (
            id, variable_value_id, event_no, event_kind, value_json, value_content_id,
            source_message_version_id, updated_by_kind, updated_by_ref_id, created_at, config_json
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(input.variable_value_id)
    .bind(input.event_no)
    .bind(input.event_kind)
    .bind(input.value_json)
    .bind(input.value_content_id)
    .bind(input.source_message_version_id)
    .bind(input.updated_by_kind)
    .bind(input.updated_by_ref_id)
    .bind(created_at)
    .bind(input.config_json)
    .execute(tx.as_mut())
    .await?;

    get_variable_event(tx.as_mut(), &id).await
}

async fn get_variable_value_with_executor<'e, E>(
    executor: E,
    variable_def_id: &str,
    scope_type: &str,
    scope_id: &str,
    include_deleted: bool,
) -> Result<Option<VariableValueRow>>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    let sql = if include_deleted {
        r#"
        SELECT *
        FROM variable_values
        WHERE variable_def_id = ? AND scope_type = ? AND scope_id = ?
        LIMIT 1
        "#
    } else {
        r#"
        SELECT *
        FROM variable_values
        WHERE variable_def_id = ? AND scope_type = ? AND scope_id = ? AND is_deleted = 0
        LIMIT 1
        "#
    };

    sqlx::query_as::<_, VariableValueRow>(sql)
        .bind(variable_def_id)
        .bind(scope_type)
        .bind(scope_id)
        .fetch_optional(executor)
        .await
        .map_err(Into::into)
}

async fn get_variable_value_by_id_with_executor<'e, E>(
    executor: E,
    id: &str,
) -> Result<Option<VariableValueRow>>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    sqlx::query_as::<_, VariableValueRow>("SELECT * FROM variable_values WHERE id = ? LIMIT 1")
        .bind(id)
        .fetch_optional(executor)
        .await
        .map_err(Into::into)
}

async fn get_variable_lock_with_executor<'e, E>(executor: E, id: &str) -> Result<VariableLockRow>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    sqlx::query_as::<_, VariableLockRow>("SELECT * FROM variable_locks WHERE id = ? LIMIT 1")
        .bind(id)
        .fetch_optional(executor)
        .await?
        .ok_or_else(|| AppError::NotFound {
            entity: "variable_lock",
            id: id.to_string(),
        })
}
