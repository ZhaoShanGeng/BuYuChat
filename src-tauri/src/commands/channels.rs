//! 渠道管理相关的 Tauri 命令。

use tauri::State;

use crate::{
    error::AppError,
    models::{Channel, ChannelTestResult, CreateChannelInput, UpdateChannelInput},
    services::channel_service,
    state::AppState,
};

/// 列出渠道，默认包含已禁用项。
pub async fn list_channels_impl(
    state: &AppState,
    include_disabled: Option<bool>,
) -> Result<Vec<Channel>, AppError> {
    channel_service::list(&state.db, include_disabled.unwrap_or(true)).await
}

/// Tauri 命令：列出渠道。
#[tauri::command]
pub async fn list_channels(
    state: State<'_, AppState>,
    include_disabled: Option<bool>,
) -> Result<Vec<Channel>, AppError> {
    list_channels_impl(state.inner(), include_disabled).await
}

/// 按 ID 获取单个渠道。
pub async fn get_channel_impl(state: &AppState, id: String) -> Result<Channel, AppError> {
    channel_service::get(&state.db, &id).await
}

/// Tauri 命令：按 ID 获取单个渠道。
#[tauri::command]
pub async fn get_channel(state: State<'_, AppState>, id: String) -> Result<Channel, AppError> {
    get_channel_impl(state.inner(), id).await
}

/// 创建渠道。
pub async fn create_channel_impl(
    state: &AppState,
    input: CreateChannelInput,
) -> Result<Channel, AppError> {
    channel_service::create(&state.db, input).await
}

/// Tauri 命令：创建渠道。
#[tauri::command]
pub async fn create_channel(
    state: State<'_, AppState>,
    input: CreateChannelInput,
) -> Result<Channel, AppError> {
    create_channel_impl(state.inner(), input).await
}

/// 更新渠道。
pub async fn update_channel_impl(
    state: &AppState,
    id: String,
    input: UpdateChannelInput,
) -> Result<Channel, AppError> {
    channel_service::update(&state.db, &id, input).await
}

/// Tauri 命令：更新渠道。
#[tauri::command]
pub async fn update_channel(
    state: State<'_, AppState>,
    id: String,
    input: UpdateChannelInput,
) -> Result<Channel, AppError> {
    update_channel_impl(state.inner(), id, input).await
}

/// 删除渠道。
pub async fn delete_channel_impl(state: &AppState, id: String) -> Result<(), AppError> {
    channel_service::delete(&state.db, &id).await
}

/// Tauri 命令：删除渠道。
#[tauri::command]
pub async fn delete_channel(state: State<'_, AppState>, id: String) -> Result<(), AppError> {
    delete_channel_impl(state.inner(), id).await
}

/// 测试渠道连通性。
pub async fn test_channel_impl(
    state: &AppState,
    id: String,
) -> Result<ChannelTestResult, AppError> {
    channel_service::test_channel(&state.db, &state.http_client, &id).await
}

/// Tauri 命令：测试渠道连通性。
#[tauri::command]
pub async fn test_channel(
    state: State<'_, AppState>,
    id: String,
) -> Result<ChannelTestResult, AppError> {
    test_channel_impl(state.inner(), id).await
}
