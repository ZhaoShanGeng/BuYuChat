use std::collections::HashMap;

use tauri::State;
use uuid::Uuid;

use crate::db::conversation;
use crate::db::message;
use crate::db::models::{ConversationRow, MessageRow};
use crate::error::Result;
use crate::state::AppState;
use crate::types::PageResponse;

#[tauri::command]
pub async fn list_conversations(
    state: State<'_, AppState>,
    page: u32,
    per_page: u32,
) -> Result<PageResponse<ConversationRow>> {
    conversation::list(&state.db, page, per_page).await
}

#[tauri::command]
pub async fn get_conversation(state: State<'_, AppState>, id: String) -> Result<ConversationRow> {
    conversation::get(&state.db, &id).await
}

#[tauri::command]
pub async fn create_conversation(
    state: State<'_, AppState>,
    model_id: String,
    provider: String,
    assistant_id: Option<String>,
) -> Result<ConversationRow> {
    conversation::create(&state.db, &model_id, &provider, assistant_id.as_deref()).await
}

#[tauri::command]
pub async fn update_conversation_title(
    state: State<'_, AppState>,
    id: String,
    title: String,
) -> Result<()> {
    conversation::update_title(&state.db, &id, &title).await
}

#[tauri::command]
pub async fn update_conversation_model(
    state: State<'_, AppState>,
    id: String,
    model_id: String,
    provider: String,
) -> Result<()> {
    conversation::update_model(&state.db, &id, &model_id, &provider).await
}

#[tauri::command]
pub async fn update_conversation_system_prompt(
    state: State<'_, AppState>,
    id: String,
    system_prompt: Option<String>,
) -> Result<()> {
    conversation::update_system_prompt(&state.db, &id, system_prompt.as_deref()).await
}

#[tauri::command]
pub async fn delete_conversation(state: State<'_, AppState>, id: String) -> Result<()> {
    conversation::delete(&state.db, &id).await
}

#[tauri::command]
pub async fn clear_conversation_messages(state: State<'_, AppState>, id: String) -> Result<()> {
    conversation::clear_messages(&state.db, &id).await
}

#[tauri::command]
pub async fn fork_conversation_from_message(
    state: State<'_, AppState>,
    conv_id: String,
    message_id: String,
) -> Result<ConversationRow> {
    fork_conversation_from_message_inner(&state.db, &conv_id, &message_id).await
}

pub async fn fork_conversation_from_message_inner(
    db: &sqlx::SqlitePool,
    conv_id: &str,
    message_id: &str,
) -> Result<ConversationRow> {
    let source = conversation::get(db, conv_id).await?;
    let path = message::list_path_to_message(db, message_id).await?;
    if path.is_empty() {
        return Err(crate::error::AppError::NotFound {
            entity: "message",
            id: message_id.to_string(),
        });
    }
    if path.iter().any(|row| row.conversation_id != conv_id) {
        return Err(crate::error::AppError::Other(
            "message does not belong to the target conversation".to_string(),
        ));
    }

    let forked = conversation::create_with_fields(
        db,
        &format!("{} · 分支", source.title),
        &source.model_id,
        &source.provider,
        source.assistant_id.as_deref(),
        source.system_prompt.as_deref(),
    )
    .await?;

    let mut message_map: HashMap<String, String> = HashMap::new();
    let mut version_group_map: HashMap<String, String> = HashMap::new();

    for row in &path {
        let versions = message::list_versions(db, &row.version_group_id).await?;
        let new_version_group_id = version_group_map
            .entry(row.version_group_id.clone())
            .or_insert_with(|| Uuid::now_v7().to_string())
            .clone();

        for version in versions {
            let new_id = Uuid::now_v7().to_string();
            let new_parent_id = version
                .parent_message_id
                .as_ref()
                .and_then(|parent_id| message_map.get(parent_id))
                .cloned();

            let cloned = MessageRow {
                id: new_id.clone(),
                conversation_id: forked.id.clone(),
                parent_message_id: new_parent_id,
                version_group_id: new_version_group_id.clone(),
                version_index: version.version_index,
                is_active: version.is_active,
                role: version.role.clone(),
                content: version.content.clone(),
                content_parts: version.content_parts.clone(),
                tool_calls: version.tool_calls.clone(),
                tool_call_id: version.tool_call_id.clone(),
                citations_json: version.citations_json.clone(),
                tokens_used: version.tokens_used,
                provider: version.provider.clone(),
                model_id: version.model_id.clone(),
                created_at: version.created_at,
            };
            message::insert(db, &cloned).await?;
            message_map.insert(version.id.clone(), new_id);
        }
    }

    conversation::touch(db, &forked.id).await?;
    conversation::get(db, &forked.id).await
}
