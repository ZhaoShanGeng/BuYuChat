use sqlx::SqlitePool;

use crate::db::models::{VariableDefRow, VariableEventRow, VariableLockRow, VariableValueRow};
use crate::db::repos::variables as repo;
use crate::domain::content::StoredContent;
use crate::domain::variables::{
    CreateVariableDefInput, CreateVariableLockInput, DeleteVariableValueInput,
    ReleaseVariableLockInput, SetVariableValueInput, UpdateVariableDefInput, VariableActionKind,
    VariableDef, VariableEvent, VariableEventKind, VariableLock, VariableScopeType,
    VariableUnlockPolicy, VariableValue, VariableValueType,
};
use crate::services::content as content_service;
use crate::services::content_store::ContentStore;
use crate::support::error::{AppError, Result};

pub async fn list_variable_defs(db: &SqlitePool) -> Result<Vec<VariableDef>> {
    repo::list_variable_defs(db)
        .await?
        .into_iter()
        .map(map_variable_def)
        .collect()
}

pub async fn get_variable_def(db: &SqlitePool, id: &str) -> Result<VariableDef> {
    map_variable_def(repo::get_variable_def(db, id).await?)
}

pub async fn get_variable_def_by_key(
    db: &SqlitePool,
    var_key: &str,
) -> Result<Option<VariableDef>> {
    repo::get_variable_def_by_key(db, var_key)
        .await?
        .map(map_variable_def)
        .transpose()
}

pub async fn create_variable_def(
    db: &SqlitePool,
    input: &CreateVariableDefInput,
) -> Result<VariableDef> {
    map_variable_def(
        repo::create_variable_def(
            db,
            &repo::CreateVariableDefRecord {
                var_key: &input.var_key,
                name: &input.name,
                value_type: input.value_type.as_str(),
                scope_type: input.scope_type.as_str(),
                namespace: &input.namespace,
                is_user_editable: input.is_user_editable,
                is_plugin_editable: input.is_plugin_editable,
                ai_can_create: input.ai_can_create,
                ai_can_update: input.ai_can_update,
                ai_can_delete: input.ai_can_delete,
                ai_can_lock: input.ai_can_lock,
                ai_can_unlock_own_lock: input.ai_can_unlock_own_lock,
                ai_can_unlock_any_lock: input.ai_can_unlock_any_lock,
                default_json: &input.default_json.to_string(),
                config_json: &input.config_json.to_string(),
            },
        )
        .await?,
    )
}

pub async fn update_variable_def(
    db: &SqlitePool,
    id: &str,
    input: &UpdateVariableDefInput,
) -> Result<VariableDef> {
    map_variable_def(
        repo::update_variable_def(
            db,
            id,
            &repo::UpdateVariableDefRecord {
                name: &input.name,
                namespace: &input.namespace,
                is_user_editable: input.is_user_editable,
                is_plugin_editable: input.is_plugin_editable,
                ai_can_create: input.ai_can_create,
                ai_can_update: input.ai_can_update,
                ai_can_delete: input.ai_can_delete,
                ai_can_lock: input.ai_can_lock,
                ai_can_unlock_own_lock: input.ai_can_unlock_own_lock,
                ai_can_unlock_any_lock: input.ai_can_unlock_any_lock,
                default_json: &input.default_json.to_string(),
                config_json: &input.config_json.to_string(),
            },
        )
        .await?,
    )
}

pub async fn delete_variable_def(db: &SqlitePool, id: &str) -> Result<()> {
    let mut tx = db.begin().await?;
    repo::delete_variable_values_by_def(&mut tx, id).await?;
    repo::delete_variable_def(&mut tx, id).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn list_values_by_scope(
    db: &SqlitePool,
    store: &ContentStore,
    scope_type: VariableScopeType,
    scope_id: &str,
    include_deleted: bool,
) -> Result<Vec<VariableValue>> {
    let rows =
        repo::list_variable_values_by_scope(db, scope_type.as_str(), scope_id, include_deleted)
            .await?;
    let mut values = Vec::with_capacity(rows.len());
    for row in rows {
        values.push(map_variable_value(db, store, row, false).await?);
    }
    Ok(values)
}

pub async fn get_value(
    db: &SqlitePool,
    store: &ContentStore,
    variable_def_id: &str,
    scope_type: VariableScopeType,
    scope_id: &str,
    include_deleted: bool,
) -> Result<Option<VariableValue>> {
    match repo::get_variable_value(
        db,
        variable_def_id,
        scope_type.as_str(),
        scope_id,
        include_deleted,
    )
    .await?
    {
        Some(row) => Ok(Some(map_variable_value(db, store, row, true).await?)),
        None => Ok(None),
    }
}

pub async fn set_value(
    db: &SqlitePool,
    store: &ContentStore,
    input: &SetVariableValueInput,
) -> Result<VariableValue> {
    let variable_def = map_variable_def(repo::get_variable_def(db, &input.variable_def_id).await?)?;
    validate_scope_matches(&variable_def, input.scope_type)?;
    validate_value_input(variable_def.value_type, input)?;

    let existing = repo::get_variable_value(
        db,
        &input.variable_def_id,
        input.scope_type.as_str(),
        &input.scope_id,
        true,
    )
    .await?;

    let permission_action = match existing.as_ref() {
        Some(row) if !row.is_deleted => VariableActionKind::Update,
        _ => VariableActionKind::Create,
    };
    enforce_action_permission(
        &variable_def,
        permission_action,
        &input.updated_by_kind,
        input.updated_by_ref_id.as_deref(),
        None,
    )?;

    if let Some(existing_row) = existing.as_ref() {
        enforce_locks(
            db,
            &variable_def,
            existing_row,
            VariableActionKind::Update,
            &input.updated_by_kind,
            input.updated_by_ref_id.as_deref(),
        )
        .await?;
    }

    let stored_content = match &input.value_content {
        Some(content_input) => {
            Some(content_service::create_content(db, store, content_input).await?)
        }
        None => None,
    };

    let mut tx = db.begin().await?;
    let row = repo::upsert_variable_value(
        &mut tx,
        &repo::UpsertVariableValueRecord {
            variable_def_id: &input.variable_def_id,
            scope_type: input.scope_type.as_str(),
            scope_id: &input.scope_id,
            value_json: &input.value_json.to_string(),
            value_content_id: stored_content.as_ref().map(|item| item.content_id.as_str()),
            source_message_version_id: input.source_message_version_id.as_deref(),
            updated_by_kind: &input.updated_by_kind,
            updated_by_ref_id: input.updated_by_ref_id.as_deref(),
        },
    )
    .await?;
    tx.commit().await?;

    map_variable_value_with_content(row, stored_content)
}

pub async fn delete_value(
    db: &SqlitePool,
    store: &ContentStore,
    input: &DeleteVariableValueInput,
) -> Result<VariableValue> {
    let variable_def = map_variable_def(repo::get_variable_def(db, &input.variable_def_id).await?)?;
    validate_scope_matches(&variable_def, input.scope_type)?;
    enforce_action_permission(
        &variable_def,
        VariableActionKind::Delete,
        &input.updated_by_kind,
        input.updated_by_ref_id.as_deref(),
        None,
    )?;

    let value = repo::get_variable_value(
        db,
        &input.variable_def_id,
        input.scope_type.as_str(),
        &input.scope_id,
        true,
    )
    .await?
    .ok_or_else(|| AppError::NotFound {
        entity: "variable_value",
        id: format!(
            "{}:{}:{}",
            input.variable_def_id,
            input.scope_type.as_str(),
            input.scope_id
        ),
    })?;

    enforce_locks(
        db,
        &variable_def,
        &value,
        VariableActionKind::Delete,
        &input.updated_by_kind,
        input.updated_by_ref_id.as_deref(),
    )
    .await?;

    let mut tx = db.begin().await?;
    let deleted = repo::mark_variable_value_deleted(
        &mut tx,
        &value.id,
        input.source_message_version_id.as_deref(),
        &input.updated_by_kind,
        input.updated_by_ref_id.as_deref(),
    )
    .await?;
    tx.commit().await?;

    map_variable_value(db, store, deleted, true).await
}

pub async fn list_events(
    db: &SqlitePool,
    store: &ContentStore,
    variable_value_id: &str,
) -> Result<Vec<VariableEvent>> {
    let rows = repo::list_variable_events(db, variable_value_id).await?;
    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        items.push(map_variable_event(db, store, row, true).await?);
    }
    Ok(items)
}

pub async fn restore_value_event(
    db: &SqlitePool,
    store: &ContentStore,
    variable_value_id: &str,
    event_id: &str,
    updated_by_kind: &str,
    updated_by_ref_id: Option<&str>,
) -> Result<VariableValue> {
    let target = repo::get_variable_event(db, event_id).await?;
    if target.variable_value_id != variable_value_id {
        return Err(AppError::Validation(
            "variable event does not belong to target variable value".to_string(),
        ));
    }

    let value_row = get_variable_value_by_id(db, variable_value_id).await?;
    let variable_def =
        map_variable_def(repo::get_variable_def(db, &value_row.variable_def_id).await?)?;
    enforce_action_permission(
        &variable_def,
        VariableActionKind::Update,
        updated_by_kind,
        updated_by_ref_id,
        None,
    )?;
    enforce_locks(
        db,
        &variable_def,
        &value_row,
        VariableActionKind::Update,
        updated_by_kind,
        updated_by_ref_id,
    )
    .await?;

    let mut tx = db.begin().await?;
    let restored = repo::restore_variable_value_event(
        &mut tx,
        variable_value_id,
        event_id,
        updated_by_kind,
        updated_by_ref_id,
    )
    .await?;
    tx.commit().await?;

    map_variable_value(db, store, restored, true).await
}

pub async fn list_locks(db: &SqlitePool, variable_value_id: &str) -> Result<Vec<VariableLock>> {
    repo::list_active_variable_locks(db, variable_value_id)
        .await?
        .into_iter()
        .map(map_variable_lock)
        .collect()
}

pub async fn create_lock(db: &SqlitePool, input: &CreateVariableLockInput) -> Result<VariableLock> {
    let variable_def = map_variable_def(repo::get_variable_def(db, &input.variable_def_id).await?)?;
    validate_scope_matches(&variable_def, input.scope_type)?;
    enforce_action_permission(
        &variable_def,
        VariableActionKind::Lock,
        &input.owner_kind,
        input.owner_ref_id.as_deref(),
        None,
    )?;

    let value = repo::get_variable_value(
        db,
        &input.variable_def_id,
        input.scope_type.as_str(),
        &input.scope_id,
        true,
    )
    .await?
    .ok_or_else(|| AppError::NotFound {
        entity: "variable_value",
        id: format!(
            "{}:{}:{}",
            input.variable_def_id,
            input.scope_type.as_str(),
            input.scope_id
        ),
    })?;

    let mut tx = db.begin().await?;
    let lock = repo::upsert_variable_lock(
        &mut tx,
        &repo::UpsertVariableLockRecord {
            variable_value_id: &value.id,
            lock_kind: input.lock_kind.as_str(),
            owner_kind: &input.owner_kind,
            owner_ref_id: input.owner_ref_id.as_deref(),
            unlock_policy: input.unlock_policy.as_str(),
            config_json: &input.config_json.to_string(),
        },
    )
    .await?;
    tx.commit().await?;

    map_variable_lock(lock)
}

pub async fn release_lock(
    db: &SqlitePool,
    input: &ReleaseVariableLockInput,
) -> Result<VariableLock> {
    let lock = repo::get_variable_lock(db, &input.variable_lock_id).await?;
    let value_row = get_variable_value_by_id(db, &lock.variable_value_id).await?;
    let variable_def =
        map_variable_def(repo::get_variable_def(db, &value_row.variable_def_id).await?)?;
    enforce_action_permission(
        &variable_def,
        VariableActionKind::Unlock,
        &input.released_by_kind,
        input.released_by_ref_id.as_deref(),
        Some(&lock),
    )?;

    let mut tx = db.begin().await?;
    let released = repo::deactivate_variable_lock(&mut tx, &lock.id).await?;
    tx.commit().await?;
    map_variable_lock(released)
}

fn map_variable_def(row: VariableDefRow) -> Result<VariableDef> {
    Ok(VariableDef {
        id: row.id,
        var_key: row.var_key,
        name: row.name,
        value_type: VariableValueType::parse(&row.value_type)?,
        scope_type: VariableScopeType::parse(&row.scope_type)?,
        namespace: row.namespace,
        is_user_editable: row.is_user_editable,
        is_plugin_editable: row.is_plugin_editable,
        ai_can_create: row.ai_can_create,
        ai_can_update: row.ai_can_update,
        ai_can_delete: row.ai_can_delete,
        ai_can_lock: row.ai_can_lock,
        ai_can_unlock_own_lock: row.ai_can_unlock_own_lock,
        ai_can_unlock_any_lock: row.ai_can_unlock_any_lock,
        default_json: parse_json(&row.default_json, "variable_defs.default_json")?,
        config_json: parse_json(&row.config_json, "variable_defs.config_json")?,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

async fn map_variable_value(
    db: &SqlitePool,
    store: &ContentStore,
    row: VariableValueRow,
    include_body: bool,
) -> Result<VariableValue> {
    let value_content = match row.value_content_id.as_deref() {
        Some(content_id) => {
            Some(content_service::get_content(db, store, content_id, include_body).await?)
        }
        None => None,
    };
    map_variable_value_with_content(row, value_content)
}

fn map_variable_value_with_content(
    row: VariableValueRow,
    value_content: Option<StoredContent>,
) -> Result<VariableValue> {
    Ok(VariableValue {
        id: row.id,
        variable_def_id: row.variable_def_id,
        scope_type: VariableScopeType::parse(&row.scope_type)?,
        scope_id: row.scope_id,
        value_json: parse_json(&row.value_json, "variable_values.value_json")?,
        value_content,
        source_message_version_id: row.source_message_version_id,
        updated_by_kind: row.updated_by_kind,
        updated_by_ref_id: row.updated_by_ref_id,
        event_no: row.event_no,
        is_deleted: row.is_deleted,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

async fn map_variable_event(
    db: &SqlitePool,
    store: &ContentStore,
    row: VariableEventRow,
    include_body: bool,
) -> Result<VariableEvent> {
    let value_content = match row.value_content_id.as_deref() {
        Some(content_id) => {
            Some(content_service::get_content(db, store, content_id, include_body).await?)
        }
        None => None,
    };
    Ok(VariableEvent {
        id: row.id,
        variable_value_id: row.variable_value_id,
        event_no: row.event_no,
        event_kind: VariableEventKind::parse(&row.event_kind)?,
        value_json: parse_json(&row.value_json, "variable_events.value_json")?,
        value_content,
        source_message_version_id: row.source_message_version_id,
        updated_by_kind: row.updated_by_kind,
        updated_by_ref_id: row.updated_by_ref_id,
        created_at: row.created_at,
    })
}

fn map_variable_lock(row: VariableLockRow) -> Result<VariableLock> {
    Ok(VariableLock {
        id: row.id,
        variable_value_id: row.variable_value_id,
        lock_kind: crate::domain::variables::VariableLockKind::parse(&row.lock_kind)?,
        owner_kind: row.owner_kind,
        owner_ref_id: row.owner_ref_id,
        unlock_policy: VariableUnlockPolicy::parse(&row.unlock_policy)?,
        active: row.active,
        config_json: parse_json(&row.config_json, "variable_locks.config_json")?,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

fn parse_json(raw: &str, field: &'static str) -> Result<serde_json::Value> {
    serde_json::from_str(raw)
        .map_err(|err| AppError::Validation(format!("failed to parse {field} as json: {err}")))
}

fn validate_scope_matches(variable_def: &VariableDef, scope_type: VariableScopeType) -> Result<()> {
    if variable_def.scope_type.as_str() != scope_type.as_str() {
        return Err(AppError::Validation(format!(
            "variable '{}' only supports scope_type '{}'",
            variable_def.var_key,
            variable_def.scope_type.as_str()
        )));
    }
    Ok(())
}

fn validate_value_input(
    value_type: VariableValueType,
    input: &SetVariableValueInput,
) -> Result<()> {
    match value_type {
        VariableValueType::String => {
            if !matches!(input.value_json, serde_json::Value::String(_)) {
                return Err(AppError::Validation(
                    "string variable requires value_json to be a string".to_string(),
                ));
            }
            if input.value_content.is_some() {
                return Err(AppError::Validation(
                    "string variable does not accept value_content".to_string(),
                ));
            }
        }
        VariableValueType::Number => {
            if !input.value_json.is_number() {
                return Err(AppError::Validation(
                    "number variable requires value_json to be a number".to_string(),
                ));
            }
            if input.value_content.is_some() {
                return Err(AppError::Validation(
                    "number variable does not accept value_content".to_string(),
                ));
            }
        }
        VariableValueType::Boolean => {
            if !matches!(input.value_json, serde_json::Value::Bool(_)) {
                return Err(AppError::Validation(
                    "boolean variable requires value_json to be a boolean".to_string(),
                ));
            }
            if input.value_content.is_some() {
                return Err(AppError::Validation(
                    "boolean variable does not accept value_content".to_string(),
                ));
            }
        }
        VariableValueType::Json => {
            if input.value_content.is_some() {
                return Err(AppError::Validation(
                    "json variable does not accept value_content".to_string(),
                ));
            }
        }
        VariableValueType::ContentRef => {
            if input.value_content.is_none() {
                return Err(AppError::Validation(
                    "content_ref variable requires value_content".to_string(),
                ));
            }
        }
    }
    Ok(())
}

fn enforce_action_permission(
    variable_def: &VariableDef,
    action: VariableActionKind,
    actor_kind: &str,
    actor_ref_id: Option<&str>,
    lock: Option<&VariableLockRow>,
) -> Result<()> {
    match actor_kind {
        "user" => {
            if !variable_def.is_user_editable {
                return Err(AppError::Validation(format!(
                    "user is not allowed to {:?} variable '{}'",
                    action, variable_def.var_key
                )));
            }
        }
        "plugin" => {
            if !variable_def.is_plugin_editable {
                return Err(AppError::Validation(format!(
                    "plugin is not allowed to {:?} variable '{}'",
                    action, variable_def.var_key
                )));
            }
        }
        "ai" => match action {
            VariableActionKind::Create if !variable_def.ai_can_create => {
                return Err(AppError::Validation(format!(
                    "ai is not allowed to create variable '{}'",
                    variable_def.var_key
                )))
            }
            VariableActionKind::Update if !variable_def.ai_can_update => {
                return Err(AppError::Validation(format!(
                    "ai is not allowed to update variable '{}'",
                    variable_def.var_key
                )))
            }
            VariableActionKind::Delete if !variable_def.ai_can_delete => {
                return Err(AppError::Validation(format!(
                    "ai is not allowed to delete variable '{}'",
                    variable_def.var_key
                )))
            }
            VariableActionKind::Lock if !variable_def.ai_can_lock => {
                return Err(AppError::Validation(format!(
                    "ai is not allowed to lock variable '{}'",
                    variable_def.var_key
                )))
            }
            VariableActionKind::Unlock => {
                let is_own_lock = lock.map(|item| {
                    item.owner_kind == actor_kind && item.owner_ref_id.as_deref() == actor_ref_id
                });
                match is_own_lock {
                    Some(true) if variable_def.ai_can_unlock_own_lock => {}
                    Some(false) if variable_def.ai_can_unlock_any_lock => {}
                    Some(true) => {
                        return Err(AppError::Validation(format!(
                            "ai is not allowed to unlock its own lock on variable '{}'",
                            variable_def.var_key
                        )))
                    }
                    Some(false) | None => {
                        return Err(AppError::Validation(format!(
                            "ai is not allowed to unlock this variable lock on '{}'",
                            variable_def.var_key
                        )))
                    }
                }
            }
            _ => {}
        },
        "workflow" | "system" => {}
        _ => {}
    }

    Ok(())
}

async fn enforce_locks(
    db: &SqlitePool,
    variable_def: &VariableDef,
    value_row: &VariableValueRow,
    action: VariableActionKind,
    actor_kind: &str,
    actor_ref_id: Option<&str>,
) -> Result<()> {
    let locks = repo::list_active_variable_locks(db, &value_row.id).await?;
    for lock in locks {
        let lock_kind = crate::domain::variables::VariableLockKind::parse(&lock.lock_kind)?;
        if !lock_kind.blocks(action) {
            continue;
        }

        if actor_kind == "user" || actor_kind == "workflow" || actor_kind == "system" {
            continue;
        }
        let _ = actor_ref_id;
        let _ = VariableUnlockPolicy::parse(&lock.unlock_policy)?;

        return Err(AppError::Validation(format!(
            "variable '{}' is locked for {:?}",
            variable_def.var_key, action
        )));
    }

    Ok(())
}

async fn get_variable_value_by_id(db: &SqlitePool, id: &str) -> Result<VariableValueRow> {
    let values =
        sqlx::query_as::<_, VariableValueRow>("SELECT * FROM variable_values WHERE id = ? LIMIT 1")
            .bind(id)
            .fetch_optional(db)
            .await?;
    values.ok_or_else(|| AppError::NotFound {
        entity: "variable_value",
        id: id.to_string(),
    })
}
