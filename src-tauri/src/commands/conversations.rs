//! 会话管理命令。
//!
//! 这里暴露的是会话资源的 CRUD 命令，实际业务校验和绑定检查都下沉到
//! `conversation_service`，命令层只负责参数转发与默认值处理。

use tauri::State;

use crate::{
    error::AppError,
    models::{
        Conversation, ConversationSummary, CreateConversationInput, UpdateConversationInput,
    },
    services::conversation_service,
    state::AppState,
};

/// 列出会话列表。
pub async fn list_conversations_impl(
    state: &AppState,
    archived: Option<bool>,
) -> Result<Vec<ConversationSummary>, AppError> {
    conversation_service::list(&state.db, archived.unwrap_or(false)).await
}

/// Tauri 命令：列出会话列表。
#[tauri::command]
pub async fn list_conversations(
    state: State<'_, AppState>,
    archived: Option<bool>,
) -> Result<Vec<ConversationSummary>, AppError> {
    list_conversations_impl(state.inner(), archived).await
}

/// 获取单个会话详情。
pub async fn get_conversation_impl(
    state: &AppState,
    id: String,
) -> Result<Conversation, AppError> {
    conversation_service::get(&state.db, &id).await
}

/// Tauri 命令：获取单个会话详情。
#[tauri::command]
pub async fn get_conversation(
    state: State<'_, AppState>,
    id: String,
) -> Result<Conversation, AppError> {
    get_conversation_impl(state.inner(), id).await
}

/// 创建会话。
pub async fn create_conversation_impl(
    state: &AppState,
    input: Option<CreateConversationInput>,
) -> Result<Conversation, AppError> {
    conversation_service::create(&state.db, input.unwrap_or_default()).await
}

/// Tauri 命令：创建会话。
#[tauri::command]
pub async fn create_conversation(
    state: State<'_, AppState>,
    input: Option<CreateConversationInput>,
) -> Result<Conversation, AppError> {
    create_conversation_impl(state.inner(), input).await
}

/// 更新会话。
pub async fn update_conversation_impl(
    state: &AppState,
    id: String,
    input: UpdateConversationInput,
) -> Result<Conversation, AppError> {
    conversation_service::update(&state.db, &id, input).await
}

/// Tauri 命令：更新会话。
#[tauri::command]
pub async fn update_conversation(
    state: State<'_, AppState>,
    id: String,
    input: UpdateConversationInput,
) -> Result<Conversation, AppError> {
    update_conversation_impl(state.inner(), id, input).await
}

/// 删除会话。
pub async fn delete_conversation_impl(state: &AppState, id: String) -> Result<(), AppError> {
    conversation_service::delete(&state.db, &id).await
}

/// Tauri 命令：删除会话。
#[tauri::command]
pub async fn delete_conversation(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), AppError> {
    delete_conversation_impl(state.inner(), id).await
}
