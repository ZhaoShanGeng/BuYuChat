use tauri::{AppHandle, Emitter, State};

use crate::app::state::AppState;
use crate::commands::incremental;
use crate::domain::content::StoredContent;
use crate::domain::messages::{
    AddAttachmentInput, CreateMessageInput, EditMessageVersionInput, GenerateReplyInput,
    GenerateReplyStreamInput, MessageContentRefView, MessageVersionView, RegenerateReplyInput,
    RegenerateReplyStreamInput,
};
use crate::support::error::{AppError, Result};

const GENERATION_STREAM_EVENT: &str = "generation_stream_event";

#[tauri::command]
pub async fn list_visible_messages(
    state: State<'_, AppState>,
    conversation_id: String,
) -> Result<Vec<MessageVersionView>> {
    crate::services::messages::list_visible_messages(
        &state.db,
        &state.content_store,
        &conversation_id,
    )
    .await
}

#[tauri::command]
pub async fn list_message_versions(
    state: State<'_, AppState>,
    node_id: String,
) -> Result<Vec<MessageVersionView>> {
    crate::services::messages::list_message_versions(&state.db, &state.content_store, &node_id)
        .await
}

#[tauri::command]
pub async fn get_message_body(
    state: State<'_, AppState>,
    version_id: String,
) -> Result<StoredContent> {
    crate::services::messages::get_message_body(&state.db, &state.content_store, &version_id).await
}

#[tauri::command]
pub async fn create_user_message(
    app: AppHandle,
    state: State<'_, AppState>,
    input: CreateMessageInput,
) -> Result<MessageVersionView> {
    let message =
        crate::services::messages::create_user_message(&state.db, &state.content_store, &input)
            .await?;
    incremental::emit_upsert(
        &app,
        "conversation",
        Some(&message.conversation_id),
        "message_version",
        Some(&message.version_id),
        &message,
    )?;
    Ok(message)
}

#[tauri::command]
pub async fn create_system_message(
    app: AppHandle,
    state: State<'_, AppState>,
    input: CreateMessageInput,
) -> Result<MessageVersionView> {
    let message =
        crate::services::messages::create_system_message(&state.db, &state.content_store, &input)
            .await?;
    incremental::emit_upsert(
        &app,
        "conversation",
        Some(&message.conversation_id),
        "message_version",
        Some(&message.version_id),
        &message,
    )?;
    Ok(message)
}

#[tauri::command]
pub async fn edit_message_version(
    app: AppHandle,
    state: State<'_, AppState>,
    input: EditMessageVersionInput,
) -> Result<MessageVersionView> {
    let message =
        crate::services::messages::edit_message_version(&state.db, &state.content_store, &input)
            .await?;
    let versions = crate::services::messages::list_message_versions(
        &state.db,
        &state.content_store,
        &input.node_id,
    )
    .await?;
    incremental::emit_upsert(
        &app,
        "conversation",
        Some(&message.conversation_id),
        "message_version",
        Some(&message.version_id),
        &message,
    )?;
    incremental::emit_replace(
        &app,
        "message_node",
        Some(&input.node_id),
        "message_versions",
        &versions,
    )?;
    Ok(message)
}

#[tauri::command]
pub async fn switch_message_version(
    app: AppHandle,
    state: State<'_, AppState>,
    node_id: String,
    version_id: String,
) -> Result<MessageVersionView> {
    let message = crate::services::messages::switch_message_version(
        &state.db,
        &state.content_store,
        &node_id,
        &version_id,
    )
    .await?;
    let versions =
        crate::services::messages::list_message_versions(&state.db, &state.content_store, &node_id)
            .await?;
    incremental::emit_upsert(
        &app,
        "conversation",
        Some(&message.conversation_id),
        "message_version",
        Some(&message.version_id),
        &message,
    )?;
    incremental::emit_replace(
        &app,
        "message_node",
        Some(&node_id),
        "message_versions",
        &versions,
    )?;
    Ok(message)
}

#[tauri::command]
pub async fn delete_message_version(
    app: AppHandle,
    state: State<'_, AppState>,
    node_id: String,
    version_id: String,
) -> Result<()> {
    let existing_versions =
        crate::services::messages::list_message_versions(&state.db, &state.content_store, &node_id)
            .await?;
    let conversation_id = existing_versions
        .first()
        .map(|item| item.conversation_id.clone());
    crate::services::messages::delete_message_version(&state.db, &node_id, &version_id).await?;
    incremental::emit_delete(
        &app,
        "message_node",
        Some(&node_id),
        "message_version",
        &version_id,
    )?;
    match crate::services::messages::list_message_versions(
        &state.db,
        &state.content_store,
        &node_id,
    )
    .await
    {
        Ok(versions) => {
            incremental::emit_replace(
                &app,
                "message_node",
                Some(&node_id),
                "message_versions",
                &versions,
            )?;
        }
        Err(AppError::NotFound {
            entity: "message_node",
            ..
        }) => {
            incremental::emit_delete(
                &app,
                "conversation",
                conversation_id.as_deref(),
                "message_node",
                &node_id,
            )?;
        }
        Err(err) => return Err(err),
    }
    Ok(())
}

#[tauri::command]
pub async fn delete_message_node(
    app: AppHandle,
    state: State<'_, AppState>,
    node_id: String,
) -> Result<()> {
    let existing_versions =
        crate::services::messages::list_message_versions(&state.db, &state.content_store, &node_id)
            .await?;
    let conversation_id = existing_versions
        .first()
        .map(|item| item.conversation_id.clone());
    crate::services::messages::delete_message_node(&state.db, &node_id).await?;
    incremental::emit_delete(
        &app,
        "conversation",
        conversation_id.as_deref(),
        "message_node",
        &node_id,
    )?;
    Ok(())
}

#[tauri::command]
pub async fn append_message_attachment(
    app: AppHandle,
    state: State<'_, AppState>,
    input: AddAttachmentInput,
) -> Result<MessageContentRefView> {
    let attachment =
        crate::services::messages::append_attachment(&state.db, &state.content_store, &input)
            .await?;
    incremental::emit_upsert(
        &app,
        "message_version",
        Some(&input.message_version_id),
        "message_content_ref",
        Some(&attachment.ref_id),
        &attachment,
    )?;
    Ok(attachment)
}

#[tauri::command]
pub async fn generate_reply(
    app: AppHandle,
    state: State<'_, AppState>,
    input: GenerateReplyInput,
) -> Result<MessageVersionView> {
    let message = crate::services::generation::generate_reply(
        &state.db,
        &state.content_store,
        &state.provider_registry,
        &input,
    )
    .await?;
    incremental::emit_upsert(
        &app,
        "conversation",
        Some(&message.conversation_id),
        "message_version",
        Some(&message.version_id),
        &message,
    )?;
    Ok(message)
}

#[tauri::command]
pub async fn regenerate_reply(
    app: AppHandle,
    state: State<'_, AppState>,
    input: RegenerateReplyInput,
) -> Result<MessageVersionView> {
    let message = crate::services::generation::regenerate_reply(
        &state.db,
        &state.content_store,
        &state.provider_registry,
        &input,
    )
    .await?;
    incremental::emit_upsert(
        &app,
        "conversation",
        Some(&message.conversation_id),
        "message_version",
        Some(&message.version_id),
        &message,
    )?;
    Ok(message)
}

#[tauri::command]
pub async fn generate_reply_stream(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    input: GenerateReplyStreamInput,
) -> Result<MessageVersionView> {
    let message = crate::services::generation::generate_reply_streaming(
        &state.db,
        &state.content_store,
        &state.provider_registry,
        &input.request,
        &input.stream_id,
        &mut |event| {
            app.emit(GENERATION_STREAM_EVENT, event)
                .map_err(|err| AppError::Other(format!("failed to emit stream event: {err}")))
        },
    )
    .await?;
    incremental::emit_upsert(
        &app,
        "conversation",
        Some(&message.conversation_id),
        "message_version",
        Some(&message.version_id),
        &message,
    )?;
    Ok(message)
}

#[tauri::command]
pub async fn regenerate_reply_stream(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    input: RegenerateReplyStreamInput,
) -> Result<MessageVersionView> {
    let message = crate::services::generation::regenerate_reply_streaming(
        &state.db,
        &state.content_store,
        &state.provider_registry,
        &input.request,
        &input.stream_id,
        &mut |event| {
            app.emit(GENERATION_STREAM_EVENT, event)
                .map_err(|err| AppError::Other(format!("failed to emit stream event: {err}")))
        },
    )
    .await?;
    incremental::emit_upsert(
        &app,
        "conversation",
        Some(&message.conversation_id),
        "message_version",
        Some(&message.version_id),
        &message,
    )?;
    Ok(message)
}
