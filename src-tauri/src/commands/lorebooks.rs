use tauri::{AppHandle, State};

use crate::app::state::AppState;
use crate::commands::incremental;
use crate::domain::lorebooks::{
    CreateLorebookEntryInput, CreateLorebookInput, LorebookDetail, LorebookEntryDetail,
    LorebookMatchInput, LorebookSummary, MatchedLorebookEntry, UpdateLorebookEntryInput,
    UpdateLorebookInput,
};
use crate::support::error::Result;

#[tauri::command]
pub async fn list_lorebooks(state: State<'_, AppState>) -> Result<Vec<LorebookSummary>> {
    crate::services::lorebooks::list_lorebooks(&state.db).await
}

#[tauri::command]
pub async fn get_lorebook_detail(state: State<'_, AppState>, id: String) -> Result<LorebookDetail> {
    crate::services::lorebooks::get_lorebook_detail(&state.db, &state.content_store, &id).await
}

#[tauri::command]
pub async fn create_lorebook(
    app: AppHandle,
    state: State<'_, AppState>,
    input: CreateLorebookInput,
) -> Result<LorebookDetail> {
    let lorebook =
        crate::services::lorebooks::create_lorebook(&state.db, &state.content_store, &input)
            .await?;
    incremental::emit_upsert(
        &app,
        "global",
        None,
        "lorebook",
        Some(&lorebook.lorebook.id),
        &lorebook,
    )?;
    Ok(lorebook)
}

#[tauri::command]
pub async fn update_lorebook(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    input: UpdateLorebookInput,
) -> Result<LorebookDetail> {
    let lorebook =
        crate::services::lorebooks::update_lorebook(&state.db, &state.content_store, &id, &input)
            .await?;
    incremental::emit_upsert(
        &app,
        "global",
        None,
        "lorebook",
        Some(&lorebook.lorebook.id),
        &lorebook,
    )?;
    Ok(lorebook)
}

#[tauri::command]
pub async fn delete_lorebook(app: AppHandle, state: State<'_, AppState>, id: String) -> Result<()> {
    crate::services::lorebooks::delete_lorebook(&state.db, &id).await?;
    incremental::emit_delete(&app, "global", None, "lorebook", &id)?;
    Ok(())
}

#[tauri::command]
pub async fn create_lorebook_entry(
    app: AppHandle,
    state: State<'_, AppState>,
    input: CreateLorebookEntryInput,
) -> Result<LorebookEntryDetail> {
    let entry =
        crate::services::lorebooks::create_entry(&state.db, &state.content_store, &input).await?;
    incremental::emit_upsert(
        &app,
        "lorebook",
        Some(&entry.lorebook_id),
        "lorebook_entry",
        Some(&entry.id),
        &entry,
    )?;
    Ok(entry)
}

#[tauri::command]
pub async fn update_lorebook_entry(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    input: UpdateLorebookEntryInput,
) -> Result<LorebookEntryDetail> {
    let entry =
        crate::services::lorebooks::update_entry(&state.db, &state.content_store, &id, &input)
            .await?;
    incremental::emit_upsert(
        &app,
        "lorebook",
        Some(&entry.lorebook_id),
        "lorebook_entry",
        Some(&entry.id),
        &entry,
    )?;
    Ok(entry)
}

#[tauri::command]
pub async fn delete_lorebook_entry(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<()> {
    crate::services::lorebooks::delete_entry(&state.db, &id).await?;
    incremental::emit_delete(&app, "lorebook_entry", None, "lorebook_entry", &id)?;
    Ok(())
}

#[tauri::command]
pub async fn replace_lorebook_entry_keys(
    app: AppHandle,
    state: State<'_, AppState>,
    entry_id: String,
    keys: Vec<String>,
) -> Result<Vec<String>> {
    crate::services::lorebooks::replace_keys(&state.db, &entry_id, &keys).await?;
    incremental::emit_replace(
        &app,
        "lorebook_entry",
        Some(&entry_id),
        "lorebook_entry_keys",
        &keys,
    )?;
    Ok(keys)
}

#[tauri::command]
pub async fn match_lorebook_entries(
    state: State<'_, AppState>,
    input: LorebookMatchInput,
) -> Result<Vec<MatchedLorebookEntry>> {
    crate::services::lorebooks::match_entries(&state.db, &state.content_store, &input).await
}
