use serde::Serialize;
use tauri::State;

use crate::db::models::{CustomChannelRow, ProviderConfigRow};
use crate::error::Result;
use crate::services::provider::ProviderService;
use crate::state::AppState;
use crate::types::ModelInfo;

#[derive(Debug, Serialize)]
pub struct TestProviderConnectionResponse {
    pub ok: bool,
    pub message: String,
}

#[tauri::command]
pub async fn list_provider_configs(state: State<'_, AppState>) -> Result<Vec<ProviderConfigRow>> {
    provider_service(&state).list_configs().await
}

#[tauri::command]
pub async fn list_custom_channels(state: State<'_, AppState>) -> Result<Vec<CustomChannelRow>> {
    provider_service(&state).list_custom_channels().await
}

#[tauri::command]
pub async fn get_provider_api_key(
    state: State<'_, AppState>,
    provider: String,
) -> Result<Option<String>> {
    provider_service(&state).get_api_key(&provider).await
}

#[tauri::command]
pub async fn get_custom_channel_api_key(
    state: State<'_, AppState>,
    id: String,
) -> Result<Option<String>> {
    provider_service(&state)
        .get_custom_channel_api_key(&id)
        .await
}

#[tauri::command]
pub async fn save_provider_config(
    state: State<'_, AppState>,
    provider: String,
    api_key: Option<String>,
    base_url: Option<String>,
) -> Result<()> {
    provider_service(&state)
        .save_config(&provider, api_key.as_deref(), base_url.as_deref())
        .await
}

#[tauri::command]
pub async fn create_custom_channel(
    state: State<'_, AppState>,
    name: String,
    channel_type: String,
    base_url: String,
    models_path: String,
    chat_path: String,
    stream_path: String,
    api_key: Option<String>,
) -> Result<CustomChannelRow> {
    provider_service(&state)
        .create_custom_channel(
            &name,
            &channel_type,
            &base_url,
            &models_path,
            &chat_path,
            &stream_path,
            api_key.as_deref(),
        )
        .await
}

#[tauri::command]
pub async fn update_custom_channel(
    state: State<'_, AppState>,
    id: String,
    name: String,
    channel_type: String,
    base_url: String,
    models_path: String,
    chat_path: String,
    stream_path: String,
    api_key: Option<String>,
) -> Result<CustomChannelRow> {
    provider_service(&state)
        .update_custom_channel(
            &id,
            &name,
            &channel_type,
            &base_url,
            &models_path,
            &chat_path,
            &stream_path,
            api_key.as_deref(),
        )
        .await
}

#[tauri::command]
pub async fn delete_custom_channel(state: State<'_, AppState>, id: String) -> Result<()> {
    provider_service(&state).delete_custom_channel(&id).await
}

#[tauri::command]
pub async fn test_provider_connection(
    state: State<'_, AppState>,
    provider: String,
) -> Result<TestProviderConnectionResponse> {
    provider_service(&state).test_connection(&provider).await?;
    Ok(TestProviderConnectionResponse {
        ok: true,
        message: "connection successful".to_string(),
    })
}

#[tauri::command]
pub async fn list_models(state: State<'_, AppState>, provider: String) -> Result<Vec<ModelInfo>> {
    provider_service(&state).list_models(&provider).await
}

#[tauri::command]
pub async fn refresh_custom_channel_models(
    state: State<'_, AppState>,
    id: String,
) -> Result<Vec<ModelInfo>> {
    provider_service(&state)
        .refresh_custom_channel_models(&id)
        .await
}

#[tauri::command]
pub async fn save_custom_channel_models(
    state: State<'_, AppState>,
    id: String,
    models: Vec<ModelInfo>,
) -> Result<()> {
    provider_service(&state)
        .save_custom_channel_models(&id, &models)
        .await
}

#[tauri::command]
pub fn save_api_key(state: State<'_, AppState>, key_id: String, value: String) -> Result<()> {
    provider_service(&state).save_api_key(&key_id, &value)
}

#[tauri::command]
pub fn delete_api_key(state: State<'_, AppState>, key_id: String) -> Result<()> {
    provider_service(&state).delete_api_key(&key_id)
}

fn provider_service(state: &State<'_, AppState>) -> ProviderService {
    ProviderService::new(
        state.db.clone(),
        state.providers.clone(),
        state.keyring.clone(),
    )
}
