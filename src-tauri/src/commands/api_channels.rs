use tauri::{AppHandle, State};

use crate::app::state::AppState;
use crate::commands::incremental;
use crate::domain::api_channels::{
    ApiChannel, ApiChannelModel, ApiChannelTestResponse, CreateApiChannelInput,
    UpdateApiChannelInput, UpsertApiChannelModelInput,
};
use crate::support::error::Result;

#[tauri::command]
pub async fn list_api_channels(state: State<'_, AppState>) -> Result<Vec<ApiChannel>> {
    crate::services::api_channels::list_channels(&state.db).await
}

#[tauri::command]
pub async fn get_api_channel(state: State<'_, AppState>, id: String) -> Result<ApiChannel> {
    crate::services::api_channels::get_channel(&state.db, &id).await
}

#[tauri::command]
pub async fn create_api_channel(
    app: AppHandle,
    state: State<'_, AppState>,
    input: CreateApiChannelInput,
) -> Result<ApiChannel> {
    let channel = crate::services::api_channels::create_channel(&state.db, &input).await?;
    incremental::emit_upsert(
        &app,
        "global",
        None,
        "api_channel",
        Some(&channel.id),
        &channel,
    )?;
    Ok(channel)
}

#[tauri::command]
pub async fn update_api_channel(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    input: UpdateApiChannelInput,
) -> Result<ApiChannel> {
    let channel = crate::services::api_channels::update_channel(&state.db, &id, &input).await?;
    incremental::emit_upsert(
        &app,
        "global",
        None,
        "api_channel",
        Some(&channel.id),
        &channel,
    )?;
    Ok(channel)
}

#[tauri::command]
pub async fn delete_api_channel(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<()> {
    crate::services::api_channels::delete_channel(&state.db, &id).await?;
    incremental::emit_delete(&app, "global", None, "api_channel", &id)?;
    Ok(())
}

#[tauri::command]
pub async fn list_api_channel_models(
    state: State<'_, AppState>,
    channel_id: String,
) -> Result<Vec<ApiChannelModel>> {
    crate::services::api_channels::list_channel_models(&state.db, &channel_id).await
}

#[tauri::command]
pub async fn fetch_api_channel_remote_models(
    state: State<'_, AppState>,
    channel_id: String,
) -> Result<Vec<ApiChannelModel>> {
    crate::services::api_channels::fetch_remote_channel_models(
        &state.db,
        state.provider_registry.as_ref(),
        &channel_id,
    )
    .await
}

#[tauri::command]
pub async fn upsert_api_channel_model(
    app: AppHandle,
    state: State<'_, AppState>,
    input: UpsertApiChannelModelInput,
) -> Result<ApiChannelModel> {
    let model = crate::services::api_channels::upsert_channel_model(&state.db, &input).await?;
    incremental::emit_upsert(
        &app,
        "api_channel",
        Some(&model.channel_id),
        "api_channel_model",
        Some(&model.id),
        &model,
    )?;
    Ok(model)
}

#[tauri::command]
pub async fn delete_api_channel_model(
    app: AppHandle,
    state: State<'_, AppState>,
    channel_id: String,
    model_id: String,
) -> Result<()> {
    crate::services::api_channels::delete_channel_model(&state.db, &channel_id, &model_id).await?;
    incremental::emit_delete(
        &app,
        "api_channel",
        Some(&channel_id),
        "api_channel_model",
        &model_id,
    )?;
    Ok(())
}

#[tauri::command]
pub async fn refresh_api_channel_models(
    app: AppHandle,
    state: State<'_, AppState>,
    channel_id: String,
) -> Result<Vec<ApiChannelModel>> {
    let models = crate::services::api_channels::refresh_channel_models(
        &state.db,
        state.provider_registry.as_ref(),
        &channel_id,
    )
    .await?;

    for model in &models {
        incremental::emit_upsert(
            &app,
            "api_channel",
            Some(&model.channel_id),
            "api_channel_model",
            Some(&model.id),
            model,
        )?;
    }

    Ok(models)
}

#[tauri::command]
pub async fn test_api_channel_message(
    state: State<'_, AppState>,
    channel_id: String,
    model_id: String,
) -> Result<ApiChannelTestResponse> {
    crate::services::api_channels::test_channel_message(
        &state.db,
        state.provider_registry.as_ref(),
        &channel_id,
        &model_id,
    )
    .await
}
