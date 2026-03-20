use tauri::{AppHandle, State};

use crate::app::state::AppState;
use crate::commands::incremental;
use crate::domain::common::ChannelBindingDetail;
use crate::domain::presets::{
    CreatePresetEntryInput, CreatePresetInput, PresetChannelBindingInput, PresetDetail,
    PresetEntryDetail, PresetSummary, UpdatePresetEntryInput, UpdatePresetInput,
};
use crate::support::error::Result;

#[tauri::command]
pub async fn list_presets(state: State<'_, AppState>) -> Result<Vec<PresetSummary>> {
    crate::services::presets::list_presets(&state.db).await
}

#[tauri::command]
pub async fn get_preset_detail(state: State<'_, AppState>, id: String) -> Result<PresetDetail> {
    crate::services::presets::get_preset_detail(&state.db, &state.content_store, &id).await
}

#[tauri::command]
pub async fn create_preset(
    app: AppHandle,
    state: State<'_, AppState>,
    input: CreatePresetInput,
) -> Result<PresetDetail> {
    let preset =
        crate::services::presets::create_preset(&state.db, &state.content_store, &input).await?;
    incremental::emit_upsert(
        &app,
        "global",
        None,
        "preset",
        Some(&preset.preset.id),
        &preset,
    )?;
    Ok(preset)
}

#[tauri::command]
pub async fn update_preset(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    input: UpdatePresetInput,
) -> Result<PresetDetail> {
    let preset =
        crate::services::presets::update_preset(&state.db, &state.content_store, &id, &input)
            .await?;
    incremental::emit_upsert(
        &app,
        "global",
        None,
        "preset",
        Some(&preset.preset.id),
        &preset,
    )?;
    Ok(preset)
}

#[tauri::command]
pub async fn delete_preset(app: AppHandle, state: State<'_, AppState>, id: String) -> Result<()> {
    crate::services::presets::delete_preset(&state.db, &id).await?;
    incremental::emit_delete(&app, "global", None, "preset", &id)?;
    Ok(())
}

#[tauri::command]
pub async fn create_preset_entry(
    app: AppHandle,
    state: State<'_, AppState>,
    input: CreatePresetEntryInput,
) -> Result<PresetEntryDetail> {
    let entry =
        crate::services::presets::create_entry(&state.db, &state.content_store, &input).await?;
    incremental::emit_upsert(
        &app,
        "preset",
        Some(&entry.preset_id),
        "preset_entry",
        Some(&entry.id),
        &entry,
    )?;
    Ok(entry)
}

#[tauri::command]
pub async fn update_preset_entry(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    input: UpdatePresetEntryInput,
) -> Result<PresetEntryDetail> {
    let entry =
        crate::services::presets::update_entry(&state.db, &state.content_store, &id, &input)
            .await?;
    incremental::emit_upsert(
        &app,
        "preset",
        Some(&entry.preset_id),
        "preset_entry",
        Some(&entry.id),
        &entry,
    )?;
    Ok(entry)
}

#[tauri::command]
pub async fn delete_preset_entry(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<()> {
    crate::services::presets::delete_entry(&state.db, &id).await?;
    incremental::emit_delete(&app, "preset_entry", None, "preset_entry", &id)?;
    Ok(())
}

#[tauri::command]
pub async fn reorder_preset_entries(
    app: AppHandle,
    state: State<'_, AppState>,
    preset_id: String,
    entry_ids: Vec<String>,
) -> Result<Vec<PresetEntryDetail>> {
    crate::services::presets::reorder_entries(&state.db, &preset_id, &entry_ids).await?;
    let detail =
        crate::services::presets::get_preset_detail(&state.db, &state.content_store, &preset_id)
            .await?;
    incremental::emit_replace(
        &app,
        "preset",
        Some(&preset_id),
        "preset_entries",
        &detail.entries,
    )?;
    Ok(detail.entries)
}

#[tauri::command]
pub async fn bind_preset_channel(
    app: AppHandle,
    state: State<'_, AppState>,
    preset_id: String,
    input: PresetChannelBindingInput,
) -> Result<ChannelBindingDetail> {
    let binding = crate::services::presets::bind_channel(&state.db, &preset_id, &input).await?;
    incremental::emit_upsert(
        &app,
        "preset",
        Some(&preset_id),
        "preset_channel_binding",
        Some(&binding.id),
        &binding,
    )?;
    Ok(binding)
}

#[tauri::command]
pub async fn unbind_preset_channel(
    app: AppHandle,
    state: State<'_, AppState>,
    preset_id: String,
    channel_id: String,
    channel_model_id: Option<String>,
) -> Result<Vec<ChannelBindingDetail>> {
    crate::services::presets::unbind_channel(
        &state.db,
        &preset_id,
        &channel_id,
        channel_model_id.as_deref(),
    )
    .await?;
    let detail =
        crate::services::presets::get_preset_detail(&state.db, &state.content_store, &preset_id)
            .await?;
    incremental::emit_replace(
        &app,
        "preset",
        Some(&preset_id),
        "preset_channel_bindings",
        &detail.channel_bindings,
    )?;
    Ok(detail.channel_bindings)
}
