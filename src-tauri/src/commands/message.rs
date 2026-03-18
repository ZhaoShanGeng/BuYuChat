use std::collections::HashMap;

use serde::Serialize;
use tauri::State;

use crate::db::conversation;
use crate::db::message;
use crate::db::models::MessageRow;
use crate::error::Result;
use crate::services::versioning::VersioningService;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct MessageBundleResponse {
    pub active_messages: Vec<MessageRow>,
    pub versions_by_group: HashMap<String, Vec<MessageRow>>,
}

#[tauri::command]
pub async fn list_messages(state: State<'_, AppState>, conv_id: String) -> Result<Vec<MessageRow>> {
    message::list_active(&state.db, &conv_id).await
}

#[tauri::command]
pub async fn list_message_bundle(
    state: State<'_, AppState>,
    conv_id: String,
) -> Result<MessageBundleResponse> {
    let active_messages = message::list_active(&state.db, &conv_id).await?;
    let version_group_ids = active_messages
        .iter()
        .map(|row| row.version_group_id.clone())
        .collect::<Vec<_>>();
    let all_versions = message::list_versions_for_groups(&state.db, &version_group_ids).await?;

    let mut versions_by_group: HashMap<String, Vec<MessageRow>> = HashMap::new();
    for row in all_versions {
        versions_by_group
            .entry(row.version_group_id.clone())
            .or_default()
            .push(row);
    }

    Ok(MessageBundleResponse {
        active_messages,
        versions_by_group,
    })
}

#[tauri::command]
pub async fn get_message_versions(
    state: State<'_, AppState>,
    version_group_id: String,
) -> Result<Vec<MessageRow>> {
    message::list_versions(&state.db, &version_group_id).await
}

#[tauri::command]
pub async fn switch_message_version(
    state: State<'_, AppState>,
    version_group_id: String,
    target_index: i64,
) -> Result<MessageRow> {
    VersioningService::new(state.db.clone())
        .switch_version(&version_group_id, target_index)
        .await
}

#[tauri::command]
pub async fn delete_message(
    state: State<'_, AppState>,
    conv_id: String,
    message_id: String,
) -> Result<()> {
    let row = message::get(&state.db, &message_id).await?;
    if row.conversation_id != conv_id {
        return Err(crate::error::AppError::Other(
            "message does not belong to the target conversation".to_string(),
        ));
    }

    message::delete_message_only(&state.db, &message_id).await?;
    conversation::touch(&state.db, &conv_id).await
}
