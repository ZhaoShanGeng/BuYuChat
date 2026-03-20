use tauri::{AppHandle, State};

use crate::app::state::AppState;
use crate::commands::incremental;
use crate::domain::summaries::{
    SummaryGroup, SummaryUsage, SummaryVersion, UpsertSummaryUsageInput,
};
use crate::support::error::Result;

#[tauri::command]
pub async fn list_summary_groups(
    state: State<'_, AppState>,
    conversation_id: String,
) -> Result<Vec<SummaryGroup>> {
    crate::services::summaries::list_summary_groups(
        &state.db,
        &state.content_store,
        &conversation_id,
    )
    .await
}

#[tauri::command]
pub async fn generate_node_summary(
    app: AppHandle,
    state: State<'_, AppState>,
    node_id: String,
    generator_preset_id: Option<String>,
) -> Result<SummaryVersion> {
    let version = crate::services::summaries::generate_node_summary(
        &state.db,
        &state.content_store,
        &state.provider_registry,
        &node_id,
        generator_preset_id.as_deref(),
    )
    .await?;
    incremental::emit_upsert(
        &app,
        "summary_group",
        Some(&version.summary_group_id),
        "summary_version",
        Some(&version.id),
        &version,
    )?;
    Ok(version)
}

#[tauri::command]
pub async fn generate_range_summary(
    app: AppHandle,
    state: State<'_, AppState>,
    start_node_id: String,
    end_node_id: String,
    generator_preset_id: Option<String>,
) -> Result<SummaryVersion> {
    let version = crate::services::summaries::generate_range_summary(
        &state.db,
        &state.content_store,
        &state.provider_registry,
        &start_node_id,
        &end_node_id,
        generator_preset_id.as_deref(),
    )
    .await?;
    incremental::emit_upsert(
        &app,
        "summary_group",
        Some(&version.summary_group_id),
        "summary_version",
        Some(&version.id),
        &version,
    )?;
    Ok(version)
}

#[tauri::command]
pub async fn generate_conversation_summary(
    app: AppHandle,
    state: State<'_, AppState>,
    conversation_id: String,
    generator_preset_id: Option<String>,
) -> Result<SummaryVersion> {
    let version = crate::services::summaries::generate_conversation_summary(
        &state.db,
        &state.content_store,
        &state.provider_registry,
        &conversation_id,
        generator_preset_id.as_deref(),
    )
    .await?;
    incremental::emit_upsert(
        &app,
        "conversation",
        Some(&conversation_id),
        "summary_version",
        Some(&version.id),
        &version,
    )?;
    Ok(version)
}

#[tauri::command]
pub async fn switch_summary_version(
    app: AppHandle,
    state: State<'_, AppState>,
    summary_group_id: String,
    summary_version_id: String,
) -> Result<SummaryVersion> {
    let version = crate::services::summaries::switch_active_summary(
        &state.db,
        &state.content_store,
        &summary_group_id,
        &summary_version_id,
    )
    .await?;
    incremental::emit_upsert(
        &app,
        "summary_group",
        Some(&summary_group_id),
        "summary_version",
        Some(&version.id),
        &version,
    )?;
    Ok(version)
}

#[tauri::command]
pub async fn upsert_summary_usage(
    app: AppHandle,
    state: State<'_, AppState>,
    input: UpsertSummaryUsageInput,
) -> Result<SummaryUsage> {
    let usage = crate::services::summaries::upsert_summary_usage(&state.db, &input).await?;
    incremental::emit_upsert(
        &app,
        if usage.conversation_id.is_some() {
            "conversation"
        } else {
            "summary_group"
        },
        usage.conversation_id.as_deref(),
        "summary_usage",
        Some(&usage.id),
        &usage,
    )?;
    Ok(usage)
}
