//! 模型管理相关的 Tauri 命令。

use tauri::State;

use crate::{
    error::AppError,
    models::{ChannelModel, CreateModelInput, RemoteModelInfo, UpdateModelInput},
    services::model_service,
    state::AppState,
};

/// 列出指定渠道下的模型列表。
pub async fn list_models_impl(
    state: &AppState,
    channel_id: String,
) -> Result<Vec<ChannelModel>, AppError> {
    model_service::list(&state.db, &channel_id).await
}

/// Tauri 命令：列出指定渠道下的模型列表。
#[tauri::command]
pub async fn list_models(
    state: State<'_, AppState>,
    channel_id: String,
) -> Result<Vec<ChannelModel>, AppError> {
    list_models_impl(state.inner(), channel_id).await
}

/// 在指定渠道下创建模型。
pub async fn create_model_impl(
    state: &AppState,
    channel_id: String,
    input: CreateModelInput,
) -> Result<ChannelModel, AppError> {
    model_service::create(&state.db, &channel_id, input).await
}

/// Tauri 命令：在指定渠道下创建模型。
#[tauri::command]
pub async fn create_model(
    state: State<'_, AppState>,
    channel_id: String,
    input: CreateModelInput,
) -> Result<ChannelModel, AppError> {
    create_model_impl(state.inner(), channel_id, input).await
}

/// 更新指定渠道下的模型。
pub async fn update_model_impl(
    state: &AppState,
    channel_id: String,
    id: String,
    input: UpdateModelInput,
) -> Result<ChannelModel, AppError> {
    model_service::update(&state.db, &channel_id, &id, input).await
}

/// Tauri 命令：更新指定渠道下的模型。
#[tauri::command]
pub async fn update_model(
    state: State<'_, AppState>,
    channel_id: String,
    id: String,
    input: UpdateModelInput,
) -> Result<ChannelModel, AppError> {
    update_model_impl(state.inner(), channel_id, id, input).await
}

/// 删除指定渠道下的模型。
pub async fn delete_model_impl(
    state: &AppState,
    channel_id: String,
    id: String,
) -> Result<(), AppError> {
    model_service::delete(&state.db, &channel_id, &id).await
}

/// Tauri 命令：删除指定渠道下的模型。
#[tauri::command]
pub async fn delete_model(
    state: State<'_, AppState>,
    channel_id: String,
    id: String,
) -> Result<(), AppError> {
    delete_model_impl(state.inner(), channel_id, id).await
}

/// 从远程渠道拉取可用模型列表。
pub async fn fetch_remote_models_impl(
    state: &AppState,
    channel_id: String,
) -> Result<Vec<RemoteModelInfo>, AppError> {
    model_service::fetch_remote_models(&state.db, &state.http_client, &channel_id).await
}

/// Tauri 命令：从远程渠道拉取可用模型列表。
#[tauri::command]
pub async fn fetch_remote_models(
    state: State<'_, AppState>,
    channel_id: String,
) -> Result<Vec<RemoteModelInfo>, AppError> {
    fetch_remote_models_impl(state.inner(), channel_id).await
}
