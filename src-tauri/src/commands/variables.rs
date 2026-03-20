use tauri::{AppHandle, State};

use crate::app::state::AppState;
use crate::commands::incremental;
use crate::domain::variables::{
    CreateVariableDefInput, CreateVariableLockInput, DeleteVariableValueInput,
    ReleaseVariableLockInput, SetVariableValueInput, UpdateVariableDefInput, VariableDef,
    VariableEvent, VariableLock, VariableScopeType, VariableValue,
};
use crate::support::error::Result;

#[tauri::command]
pub async fn list_variable_defs(state: State<'_, AppState>) -> Result<Vec<VariableDef>> {
    crate::services::variables::list_variable_defs(&state.db).await
}

#[tauri::command]
pub async fn get_variable_def(state: State<'_, AppState>, id: String) -> Result<VariableDef> {
    crate::services::variables::get_variable_def(&state.db, &id).await
}

#[tauri::command]
pub async fn create_variable_def(
    app: AppHandle,
    state: State<'_, AppState>,
    input: CreateVariableDefInput,
) -> Result<VariableDef> {
    let variable_def = crate::services::variables::create_variable_def(&state.db, &input).await?;
    incremental::emit_upsert(
        &app,
        "global",
        None,
        "variable_def",
        Some(&variable_def.id),
        &variable_def,
    )?;
    Ok(variable_def)
}

#[tauri::command]
pub async fn update_variable_def(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    input: UpdateVariableDefInput,
) -> Result<VariableDef> {
    let variable_def =
        crate::services::variables::update_variable_def(&state.db, &id, &input).await?;
    incremental::emit_upsert(
        &app,
        "global",
        None,
        "variable_def",
        Some(&variable_def.id),
        &variable_def,
    )?;
    Ok(variable_def)
}

#[tauri::command]
pub async fn delete_variable_def(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<()> {
    crate::services::variables::delete_variable_def(&state.db, &id).await?;
    incremental::emit_delete(&app, "global", None, "variable_def", &id)?;
    Ok(())
}

#[tauri::command]
pub async fn list_variable_values(
    state: State<'_, AppState>,
    scope_type: VariableScopeType,
    scope_id: String,
    include_deleted: Option<bool>,
) -> Result<Vec<VariableValue>> {
    crate::services::variables::list_values_by_scope(
        &state.db,
        &state.content_store,
        scope_type,
        &scope_id,
        include_deleted.unwrap_or(false),
    )
    .await
}

#[tauri::command]
pub async fn get_variable_value(
    state: State<'_, AppState>,
    variable_def_id: String,
    scope_type: VariableScopeType,
    scope_id: String,
    include_deleted: Option<bool>,
) -> Result<Option<VariableValue>> {
    crate::services::variables::get_value(
        &state.db,
        &state.content_store,
        &variable_def_id,
        scope_type,
        &scope_id,
        include_deleted.unwrap_or(false),
    )
    .await
}

#[tauri::command]
pub async fn set_variable_value(
    app: AppHandle,
    state: State<'_, AppState>,
    input: SetVariableValueInput,
) -> Result<VariableValue> {
    let value =
        crate::services::variables::set_value(&state.db, &state.content_store, &input).await?;
    incremental::emit_upsert(
        &app,
        input.scope_type.as_str(),
        Some(&input.scope_id),
        "variable_value",
        Some(&value.id),
        &value,
    )?;
    Ok(value)
}

#[tauri::command]
pub async fn delete_variable_value(
    app: AppHandle,
    state: State<'_, AppState>,
    input: DeleteVariableValueInput,
) -> Result<VariableValue> {
    let value =
        crate::services::variables::delete_value(&state.db, &state.content_store, &input).await?;
    incremental::emit_upsert(
        &app,
        input.scope_type.as_str(),
        Some(&input.scope_id),
        "variable_value",
        Some(&value.id),
        &value,
    )?;
    Ok(value)
}

#[tauri::command]
pub async fn list_variable_events(
    state: State<'_, AppState>,
    variable_value_id: String,
) -> Result<Vec<VariableEvent>> {
    crate::services::variables::list_events(&state.db, &state.content_store, &variable_value_id)
        .await
}

#[tauri::command]
pub async fn restore_variable_event(
    app: AppHandle,
    state: State<'_, AppState>,
    variable_value_id: String,
    event_id: String,
    updated_by_kind: String,
    updated_by_ref_id: Option<String>,
) -> Result<VariableValue> {
    let value = crate::services::variables::restore_value_event(
        &state.db,
        &state.content_store,
        &variable_value_id,
        &event_id,
        &updated_by_kind,
        updated_by_ref_id.as_deref(),
    )
    .await?;
    incremental::emit_upsert(
        &app,
        value.scope_type.as_str(),
        Some(&value.scope_id),
        "variable_value",
        Some(&value.id),
        &value,
    )?;
    Ok(value)
}

#[tauri::command]
pub async fn list_variable_locks(
    state: State<'_, AppState>,
    variable_value_id: String,
) -> Result<Vec<VariableLock>> {
    crate::services::variables::list_locks(&state.db, &variable_value_id).await
}

#[tauri::command]
pub async fn create_variable_lock(
    app: AppHandle,
    state: State<'_, AppState>,
    input: CreateVariableLockInput,
) -> Result<VariableLock> {
    let lock = crate::services::variables::create_lock(&state.db, &input).await?;
    incremental::emit_upsert(
        &app,
        input.scope_type.as_str(),
        Some(&input.scope_id),
        "variable_lock",
        Some(&lock.id),
        &lock,
    )?;
    Ok(lock)
}

#[tauri::command]
pub async fn release_variable_lock(
    app: AppHandle,
    state: State<'_, AppState>,
    input: ReleaseVariableLockInput,
) -> Result<VariableLock> {
    let lock = crate::services::variables::release_lock(&state.db, &input).await?;
    incremental::emit_delete(
        &app,
        "variable_value",
        Some(&lock.variable_value_id),
        "variable_lock",
        &lock.id,
    )?;
    Ok(lock)
}
