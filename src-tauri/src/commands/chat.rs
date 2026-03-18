use serde::Serialize;
use tauri::{AppHandle, State};

use crate::error::Result;
use crate::services::chat::ChatService;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct SendMessageResponse {
    pub user_msg_id: String,
    pub assistant_msg_id: String,
}

#[derive(Debug, Serialize)]
pub struct RegenerateMessageResponse {
    pub assistant_msg_id: String,
}

#[derive(Debug, Serialize)]
pub struct SaveMessageEditResponse {
    pub message_id: String,
}

#[tauri::command]
pub async fn send_message(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    conv_id: String,
    content: String,
    override_model: Option<String>,
) -> Result<SendMessageResponse> {
    let chat_service = ChatService::new(state.db.clone(), state.providers.clone());
    let (user_msg_id, assistant_msg_id) = chat_service
        .send_message(&conv_id, content, override_model, &app_handle)
        .await?;

    Ok(SendMessageResponse {
        user_msg_id,
        assistant_msg_id,
    })
}

#[tauri::command]
pub async fn regenerate_message(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    conv_id: String,
    message_id: Option<String>,
) -> Result<RegenerateMessageResponse> {
    let chat_service = ChatService::new(state.db.clone(), state.providers.clone());
    let assistant_msg_id = match message_id.as_deref() {
        Some(message_id) => {
            chat_service
                .regenerate_from_message(&conv_id, message_id, &app_handle)
                .await?
        }
        None => chat_service.regenerate(&conv_id, &app_handle).await?,
    };
    Ok(RegenerateMessageResponse { assistant_msg_id })
}

#[tauri::command]
pub async fn save_message_edit(
    state: State<'_, AppState>,
    conv_id: String,
    message_id: String,
    new_content: String,
) -> Result<SaveMessageEditResponse> {
    let chat_service = ChatService::new(state.db.clone(), state.providers.clone());
    let message_id = chat_service
        .save_message_edit(&conv_id, &message_id, new_content)
        .await?;
    Ok(SaveMessageEditResponse { message_id })
}

#[tauri::command]
pub async fn edit_user_message(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    conv_id: String,
    message_id: String,
    new_content: String,
) -> Result<SendMessageResponse> {
    let chat_service = ChatService::new(state.db.clone(), state.providers.clone());
    let (user_msg_id, assistant_msg_id) = chat_service
        .edit_user_message(&conv_id, &message_id, new_content, &app_handle)
        .await?;

    Ok(SendMessageResponse {
        user_msg_id,
        assistant_msg_id,
    })
}
