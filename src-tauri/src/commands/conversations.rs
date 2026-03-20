use tauri::{AppHandle, State};

use crate::app::state::AppState;
use crate::commands::incremental;
use crate::domain::common::{ChannelBindingDetail, ResourceBindingDetail};
use crate::domain::common::{ChannelBindingInput, ResourceBindingInput};
use crate::domain::conversations::{
    ConversationDetail, ConversationParticipantDetail, ConversationParticipantInput,
    ConversationSummary, CreateConversationInput, UpdateConversationMetaInput,
};
use crate::support::error::Result;

#[tauri::command]
pub async fn list_conversations(state: State<'_, AppState>) -> Result<Vec<ConversationSummary>> {
    crate::services::conversations::list_conversations(&state.db).await
}

#[tauri::command]
pub async fn get_conversation_detail(
    state: State<'_, AppState>,
    id: String,
) -> Result<ConversationDetail> {
    crate::services::conversations::get_conversation_detail(&state.db, &id).await
}

#[tauri::command]
pub async fn create_conversation(
    app: AppHandle,
    state: State<'_, AppState>,
    input: CreateConversationInput,
) -> Result<ConversationDetail> {
    let detail = crate::services::conversations::create_conversation(&state.db, &input).await?;
    incremental::emit_upsert(
        &app,
        "global",
        None,
        "conversation",
        Some(&detail.summary.id),
        &detail,
    )?;
    Ok(detail)
}

#[tauri::command]
pub async fn rename_conversation(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    title: String,
) -> Result<ConversationDetail> {
    let detail =
        crate::services::conversations::rename_conversation(&state.db, &id, &title).await?;
    incremental::emit_upsert(
        &app,
        "global",
        None,
        "conversation",
        Some(&detail.summary.id),
        &detail,
    )?;
    Ok(detail)
}

#[tauri::command]
pub async fn update_conversation_meta(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    input: UpdateConversationMetaInput,
) -> Result<ConversationDetail> {
    let detail =
        crate::services::conversations::update_conversation_meta(&state.db, &id, &input).await?;
    incremental::emit_upsert(
        &app,
        "global",
        None,
        "conversation",
        Some(&detail.summary.id),
        &detail,
    )?;
    Ok(detail)
}

#[tauri::command]
pub async fn delete_conversation(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<()> {
    crate::services::conversations::delete_conversation(&state.db, &id).await?;
    incremental::emit_delete(&app, "global", None, "conversation", &id)?;
    Ok(())
}

#[tauri::command]
pub async fn replace_conversation_participants(
    app: AppHandle,
    state: State<'_, AppState>,
    conversation_id: String,
    items: Vec<ConversationParticipantInput>,
) -> Result<Vec<ConversationParticipantDetail>> {
    crate::services::conversations::replace_participants(&state.db, &conversation_id, &items)
        .await?;
    let detail =
        crate::services::conversations::get_conversation_detail(&state.db, &conversation_id)
            .await?;
    incremental::emit_replace(
        &app,
        "conversation",
        Some(&conversation_id),
        "conversation_participants",
        &detail.participants,
    )?;
    Ok(detail.participants)
}

#[tauri::command]
pub async fn replace_conversation_presets(
    app: AppHandle,
    state: State<'_, AppState>,
    conversation_id: String,
    items: Vec<ResourceBindingInput>,
) -> Result<Vec<ResourceBindingDetail>> {
    crate::services::conversations::replace_presets(&state.db, &conversation_id, &items).await?;
    let detail =
        crate::services::conversations::get_conversation_detail(&state.db, &conversation_id)
            .await?;
    incremental::emit_replace(
        &app,
        "conversation",
        Some(&conversation_id),
        "conversation_preset_bindings",
        &detail.preset_bindings,
    )?;
    Ok(detail.preset_bindings)
}

#[tauri::command]
pub async fn replace_conversation_lorebooks(
    app: AppHandle,
    state: State<'_, AppState>,
    conversation_id: String,
    items: Vec<ResourceBindingInput>,
) -> Result<Vec<ResourceBindingDetail>> {
    crate::services::conversations::replace_lorebooks(&state.db, &conversation_id, &items).await?;
    let detail =
        crate::services::conversations::get_conversation_detail(&state.db, &conversation_id)
            .await?;
    incremental::emit_replace(
        &app,
        "conversation",
        Some(&conversation_id),
        "conversation_lorebook_bindings",
        &detail.lorebook_bindings,
    )?;
    Ok(detail.lorebook_bindings)
}

#[tauri::command]
pub async fn replace_conversation_user_profiles(
    app: AppHandle,
    state: State<'_, AppState>,
    conversation_id: String,
    items: Vec<ResourceBindingInput>,
) -> Result<Vec<ResourceBindingDetail>> {
    crate::services::conversations::replace_user_profiles(&state.db, &conversation_id, &items)
        .await?;
    let detail =
        crate::services::conversations::get_conversation_detail(&state.db, &conversation_id)
            .await?;
    incremental::emit_replace(
        &app,
        "conversation",
        Some(&conversation_id),
        "conversation_user_profile_bindings",
        &detail.user_profile_bindings,
    )?;
    Ok(detail.user_profile_bindings)
}

#[tauri::command]
pub async fn replace_conversation_channels(
    app: AppHandle,
    state: State<'_, AppState>,
    conversation_id: String,
    items: Vec<ChannelBindingInput>,
) -> Result<Vec<ChannelBindingDetail>> {
    crate::services::conversations::replace_channels(&state.db, &conversation_id, &items).await?;
    let detail =
        crate::services::conversations::get_conversation_detail(&state.db, &conversation_id)
            .await?;
    incremental::emit_replace(
        &app,
        "conversation",
        Some(&conversation_id),
        "conversation_channel_bindings",
        &detail.channel_bindings,
    )?;
    Ok(detail.channel_bindings)
}
