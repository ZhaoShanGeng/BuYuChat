//! 消息与生成控制命令。
//!
//! 该文件承接消息查询、版本管理与 AI 生成相关的所有 Tauri 命令。
//! 由于生成过程需要通过 Tauri Channel 持续回推事件，这里把 `Channel<GenerationEvent>`
//! 直接作为命令参数暴露给前端。

use tauri::{State, ipc::Channel};

use crate::{
    error::AppError,
    models::{
        DeleteVersionResult, GenerationEvent, MessageNode, RerollInput, RerollResult,
        SendMessageInput, SendMessageResponse, VersionContent,
    },
    services::message_service,
    state::AppState,
};

/// 查询会话下的消息列表。
pub async fn list_messages_impl(
    state: &AppState,
    id: String,
    before_order_key: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<MessageNode>, AppError> {
    message_service::list_messages(state, &id, before_order_key, limit).await
}

/// Tauri 命令：查询会话下的消息列表。
#[tauri::command]
pub async fn list_messages(
    state: State<'_, AppState>,
    id: String,
    before_order_key: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<MessageNode>, AppError> {
    list_messages_impl(state.inner(), id, before_order_key, limit).await
}

/// 按需加载某个版本的完整内容。
pub async fn get_version_content_impl(
    state: &AppState,
    version_id: String,
) -> Result<VersionContent, AppError> {
    message_service::get_version_content(state, &version_id).await
}

/// Tauri 命令：按需加载某个版本的完整内容。
#[tauri::command]
pub async fn get_version_content(
    state: State<'_, AppState>,
    version_id: String,
) -> Result<VersionContent, AppError> {
    get_version_content_impl(state.inner(), version_id).await
}

/// 切换某个楼层的 active version。
pub async fn set_active_version_impl(
    state: &AppState,
    id: String,
    node_id: String,
    version_id: String,
) -> Result<(), AppError> {
    message_service::set_active_version(state, &id, &node_id, &version_id).await
}

/// Tauri 命令：切换某个楼层的 active version。
#[tauri::command]
pub async fn set_active_version(
    state: State<'_, AppState>,
    id: String,
    node_id: String,
    version_id: String,
) -> Result<(), AppError> {
    set_active_version_impl(state.inner(), id, node_id, version_id).await
}

/// 删除指定版本。
pub async fn delete_version_impl(
    state: &AppState,
    id: String,
    node_id: String,
    version_id: String,
) -> Result<DeleteVersionResult, AppError> {
    message_service::delete_version(state, &id, &node_id, &version_id).await
}

/// Tauri 命令：删除指定版本。
#[tauri::command]
pub async fn delete_version(
    state: State<'_, AppState>,
    id: String,
    node_id: String,
    version_id: String,
) -> Result<DeleteVersionResult, AppError> {
    delete_version_impl(state.inner(), id, node_id, version_id).await
}

/// 发送消息并触发后台生成。
pub async fn send_message_impl(
    state: &AppState,
    id: String,
    input: SendMessageInput,
    event_channel: Option<Channel<GenerationEvent>>,
) -> Result<SendMessageResponse, AppError> {
    message_service::send_message(state, &id, input, event_channel).await
}

/// Tauri 命令：发送消息并触发后台生成。
#[tauri::command]
pub async fn send_message(
    state: State<'_, AppState>,
    id: String,
    input: SendMessageInput,
    event_channel: Channel<GenerationEvent>,
) -> Result<SendMessageResponse, AppError> {
    send_message_impl(state.inner(), id, input, Some(event_channel)).await
}

/// 对指定楼层执行 reroll。
pub async fn reroll_impl(
    state: &AppState,
    id: String,
    node_id: String,
    input: Option<RerollInput>,
    event_channel: Option<Channel<GenerationEvent>>,
) -> Result<RerollResult, AppError> {
    message_service::reroll(
        state,
        &id,
        &node_id,
        input.unwrap_or_default(),
        event_channel,
    )
    .await
}

/// Tauri 命令：对指定楼层执行 reroll。
#[tauri::command]
pub async fn reroll(
    state: State<'_, AppState>,
    id: String,
    node_id: String,
    input: Option<RerollInput>,
    event_channel: Channel<GenerationEvent>,
) -> Result<RerollResult, AppError> {
    reroll_impl(state.inner(), id, node_id, input, Some(event_channel)).await
}

/// 取消某个 generating version。
pub async fn cancel_generation_impl(
    state: &AppState,
    version_id: String,
) -> Result<(), AppError> {
    message_service::cancel_generation(state, &version_id).await
}

/// Tauri 命令：取消某个 generating version。
#[tauri::command]
pub async fn cancel_generation(
    state: State<'_, AppState>,
    version_id: String,
) -> Result<(), AppError> {
    cancel_generation_impl(state.inner(), version_id).await
}
