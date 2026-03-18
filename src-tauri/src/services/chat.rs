use std::sync::Arc;

use chrono::Utc;
use sqlx::SqlitePool;
use tauri::{AppHandle, Emitter};
use tracing::debug;
use uuid::Uuid;

use crate::db::{conversation, message, models::MessageRow};
use crate::error::{AppError, Result};
use crate::providers::ProviderRegistry;
use crate::types::{ChatRequest, Message, MessageContent, ModelParams, Role, StreamEvent};

pub struct ChatService {
    db: SqlitePool,
    providers: Arc<ProviderRegistry>,
}

impl ChatService {
    pub fn new(db: SqlitePool, providers: Arc<ProviderRegistry>) -> Self {
        Self { db, providers }
    }

    pub async fn send_message(
        &self,
        conv_id: &str,
        content: String,
        override_model: Option<String>,
        app_handle: &AppHandle,
    ) -> Result<(String, String)> {
        self.send_message_inner(conv_id, content, override_model, Some(app_handle))
            .await
    }

    pub async fn send_message_no_emit(
        &self,
        conv_id: &str,
        content: String,
        override_model: Option<String>,
    ) -> Result<(String, String)> {
        self.send_message_inner(conv_id, content, override_model, None)
            .await
    }

    async fn send_message_inner(
        &self,
        conv_id: &str,
        content: String,
        override_model: Option<String>,
        app_handle: Option<&AppHandle>,
    ) -> Result<(String, String)> {
        let conv = conversation::get(&self.db, conv_id).await?;
        let model = override_model.unwrap_or_else(|| conv.model_id.clone());

        let user_row = self
            .insert_user_message(
                conv_id,
                content,
                None,
                message::find_last_active_message_id(&self.db, conv_id).await?,
            )
            .await?;

        let history_rows = message::list_active(&self.db, conv_id).await?;
        let assistant_row = self
            .insert_assistant_placeholder(
                conv_id,
                Some(user_row.id.clone()),
                Uuid::now_v7().to_string(),
                1,
                &conv.provider,
                &model,
            )
            .await?;

        self.complete_assistant_message(
            conv_id,
            &conv.provider,
            &model,
            conv.system_prompt.clone(),
            history_rows,
            &assistant_row,
            app_handle,
        )
        .await?;

        Ok((user_row.id, assistant_row.id))
    }

    pub async fn regenerate(&self, conv_id: &str, app_handle: &AppHandle) -> Result<String> {
        self.regenerate_inner(conv_id, None, Some(app_handle)).await
    }

    pub async fn regenerate_no_emit(&self, conv_id: &str) -> Result<String> {
        self.regenerate_inner(conv_id, None, None).await
    }

    pub async fn regenerate_from_message(
        &self,
        conv_id: &str,
        message_id: &str,
        app_handle: &AppHandle,
    ) -> Result<String> {
        let row = message::get(&self.db, message_id).await?;
        if row.conversation_id != conv_id {
            return Err(AppError::Other(
                "message does not belong to the target conversation".to_string(),
            ));
        }

        match Role::parse(&row.role)? {
            Role::User | Role::Assistant => {}
            other => {
                return Err(AppError::Other(format!(
                    "message role {} does not support regenerate",
                    other.as_str()
                )))
            }
        }

        if row.role == Role::User.as_str() {
            self.regenerate_user_reply(conv_id, &row, app_handle).await
        } else {
            self.regenerate_inner(conv_id, Some(message_id), Some(app_handle))
                .await
        }
    }

    async fn regenerate_user_reply(
        &self,
        conv_id: &str,
        user_row: &MessageRow,
        app_handle: &AppHandle,
    ) -> Result<String> {
        let active_child = message::find_active_child(&self.db, &user_row.id).await?;
        let existing_assistant_child = message::find_children(&self.db, &user_row.id)
            .await?
            .into_iter()
            .find(|row| row.role == Role::Assistant.as_str());
        let version_group_id = existing_assistant_child
            .as_ref()
            .map(|row| row.version_group_id.clone())
            .unwrap_or_else(|| Uuid::now_v7().to_string());
        let version_index = message::max_version_index(&self.db, &version_group_id).await? + 1;

        if let Some(active_child) = active_child.as_ref() {
            let mut tx = self.db.begin().await?;
            message::delete_descendants_tx(&mut tx, &active_child.id).await?;
            tx.commit().await?;
        }

        let conv = conversation::get(&self.db, conv_id).await?;
        let history_rows = message::list_path_to_message(&self.db, &user_row.id).await?;
        let assistant_row = MessageRow {
            id: Uuid::now_v7().to_string(),
            conversation_id: conv_id.to_string(),
            parent_message_id: Some(user_row.id.clone()),
            version_group_id,
            version_index,
            is_active: true,
            role: Role::Assistant.as_str().to_string(),
            content: Some(String::new()),
            created_at: Utc::now().timestamp_millis(),
            provider: Some(conv.provider.clone()),
            model_id: Some(conv.model_id.clone()),
            ..Default::default()
        };
        message::insert(&self.db, &assistant_row).await?;

        self.complete_assistant_message(
            conv_id,
            &conv.provider,
            &conv.model_id,
            conv.system_prompt.clone(),
            history_rows,
            &assistant_row,
            Some(app_handle),
        )
        .await?;

        Ok(assistant_row.id)
    }

    async fn regenerate_inner(
        &self,
        conv_id: &str,
        message_id: Option<&str>,
        app_handle: Option<&AppHandle>,
    ) -> Result<String> {
        let conv = conversation::get(&self.db, conv_id).await?;
        let target_assistant = match message_id {
            Some(message_id) => {
                let row = message::get(&self.db, message_id).await?;
                if row.role != Role::Assistant.as_str() {
                    return Err(AppError::Other(
                        "only assistant messages can be regenerated".to_string(),
                    ));
                }
                row
            }
            None => message::find_last_active_assistant(&self.db, conv_id).await?,
        };

        let next_version_index =
            message::max_version_index(&self.db, &target_assistant.version_group_id).await? + 1;
        let history_rows = match target_assistant.parent_message_id.as_deref() {
            Some(parent_message_id) => {
                message::list_path_to_message(&self.db, parent_message_id).await?
            }
            None => Vec::new(),
        };

        let assistant_row = {
            let mut tx = self.db.begin().await?;
            let row = MessageRow {
                id: Uuid::now_v7().to_string(),
                conversation_id: conv_id.to_string(),
                parent_message_id: target_assistant.parent_message_id.clone(),
                version_group_id: target_assistant.version_group_id.clone(),
                version_index: next_version_index,
                is_active: true,
                role: Role::Assistant.as_str().to_string(),
                content: Some(String::new()),
                provider: Some(conv.provider.clone()),
                model_id: Some(conv.model_id.clone()),
                created_at: Utc::now().timestamp_millis(),
                ..Default::default()
            };
            insert_message_tx(&mut tx, &row).await?;
            activate_version_group_tx(&mut tx, &target_assistant.version_group_id, &row.id).await?;
            tx.commit().await?;
            row
        };

        self.complete_assistant_message(
            conv_id,
            &conv.provider,
            &conv.model_id,
            conv.system_prompt.clone(),
            history_rows,
            &assistant_row,
            app_handle,
        )
        .await?;

        Ok(assistant_row.id)
    }

    pub async fn save_message_edit(
        &self,
        conv_id: &str,
        message_id: &str,
        new_content: String,
    ) -> Result<String> {
        let target_message = message::get(&self.db, message_id).await?;
        if target_message.conversation_id != conv_id {
            return Err(AppError::Other(
                "message does not belong to the target conversation".to_string(),
            ));
        }
        let next_version_index =
            message::max_version_index(&self.db, &target_message.version_group_id).await? + 1;

        let new_row = {
            let mut tx = self.db.begin().await?;

            let row = MessageRow {
                id: Uuid::now_v7().to_string(),
                conversation_id: conv_id.to_string(),
                parent_message_id: target_message.parent_message_id.clone(),
                version_group_id: target_message.version_group_id.clone(),
                version_index: next_version_index,
                is_active: true,
                role: target_message.role.clone(),
                content: Some(new_content),
                content_parts: None,
                tool_calls: None,
                tool_call_id: target_message.tool_call_id.clone(),
                citations_json: None,
                tokens_used: None,
                provider: target_message.provider.clone(),
                model_id: target_message.model_id.clone(),
                created_at: Utc::now().timestamp_millis(),
            };
            insert_message_tx(&mut tx, &row).await?;
            activate_version_group_tx(&mut tx, &target_message.version_group_id, &row.id).await?;
            tx.commit().await?;
            row
        };

        conversation::touch(&self.db, conv_id).await?;
        Ok(new_row.id)
    }

    pub async fn edit_user_message(
        &self,
        conv_id: &str,
        message_id: &str,
        new_content: String,
        app_handle: &AppHandle,
    ) -> Result<(String, String)> {
        let user_msg_id = self
            .save_message_edit(conv_id, message_id, new_content)
            .await?;
        let assistant_msg_id = self
            .regenerate_from_parent(conv_id, &user_msg_id, app_handle)
            .await?;
        Ok((user_msg_id, assistant_msg_id))
    }

    async fn regenerate_from_parent(
        &self,
        conv_id: &str,
        parent_message_id: &str,
        app_handle: &AppHandle,
    ) -> Result<String> {
        let conv = conversation::get(&self.db, conv_id).await?;
        let history_rows = message::list_path_to_message(&self.db, parent_message_id).await?;
        let existing_assistant_child = message::find_children(&self.db, parent_message_id)
            .await?
            .into_iter()
            .find(|row| row.role == Role::Assistant.as_str());
        let version_group_id = existing_assistant_child
            .as_ref()
            .map(|row| row.version_group_id.clone())
            .unwrap_or_else(|| Uuid::now_v7().to_string());
        let version_index = message::max_version_index(&self.db, &version_group_id).await? + 1;

        let assistant_row = {
            let mut tx = self.db.begin().await?;
            if let Some(active_child) =
                message::find_active_child(&self.db, parent_message_id).await?
            {
                message::delete_descendants_tx(&mut tx, &active_child.id).await?;
            }
            let row = MessageRow {
                id: Uuid::now_v7().to_string(),
                conversation_id: conv_id.to_string(),
                parent_message_id: Some(parent_message_id.to_string()),
                version_group_id: version_group_id.clone(),
                version_index,
                is_active: true,
                role: Role::Assistant.as_str().to_string(),
                content: Some(String::new()),
                provider: Some(conv.provider.clone()),
                model_id: Some(conv.model_id.clone()),
                created_at: Utc::now().timestamp_millis(),
                ..Default::default()
            };
            insert_message_tx(&mut tx, &row).await?;
            activate_version_group_tx(&mut tx, &version_group_id, &row.id).await?;
            tx.commit().await?;
            row
        };

        self.complete_assistant_message(
            conv_id,
            &conv.provider,
            &conv.model_id,
            conv.system_prompt.clone(),
            history_rows,
            &assistant_row,
            Some(app_handle),
        )
        .await?;

        Ok(assistant_row.id)
    }

    async fn complete_assistant_message(
        &self,
        conv_id: &str,
        provider_name: &str,
        model: &str,
        system_prompt: Option<String>,
        history_rows: Vec<MessageRow>,
        assistant_row: &MessageRow,
        app_handle: Option<&AppHandle>,
    ) -> Result<()> {
        let request_messages = history_rows
            .iter()
            .map(map_row_to_message)
            .collect::<Result<Vec<_>>>()?;

        let provider = self.providers.get(provider_name).await?;
        debug!(
            conv_id = %conv_id,
            provider = %provider_name,
            model = %model,
            history_messages = history_rows.len(),
            system_prompt_present = system_prompt.as_ref().map(|value| !value.trim().is_empty()).unwrap_or(false),
            system_prompt_len = system_prompt.as_ref().map(|value| value.len()).unwrap_or(0),
            "completing assistant message"
        );
        let response = provider
            .chat(&ChatRequest {
                model: model.to_string(),
                messages: request_messages,
                system_prompt,
                params: ModelParams::default(),
                tools: None,
                stream: false,
            })
            .await?;

        let tool_calls_json = response
            .tool_calls
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        let total_tokens = response
            .usage
            .as_ref()
            .map(|usage| usage.total_tokens as i64);

        message::update_assistant_result(
            &self.db,
            &assistant_row.id,
            &response.content,
            tool_calls_json.as_deref(),
            None,
            total_tokens,
        )
        .await?;
        conversation::touch(&self.db, conv_id).await?;

        if !response.content.is_empty() {
            emit_stream_event(
                app_handle,
                conv_id,
                StreamEvent::Delta {
                    text: response.content.clone(),
                },
            )?;
        }
        emit_stream_event(
            app_handle,
            conv_id,
            StreamEvent::Done {
                usage: response.usage,
                finish_reason: response.finish_reason.unwrap_or_else(|| "stop".to_string()),
            },
        )?;

        Ok(())
    }

    async fn insert_user_message(
        &self,
        conv_id: &str,
        content: String,
        version_group_id: Option<String>,
        parent_message_id: Option<String>,
    ) -> Result<MessageRow> {
        let row = new_user_row_with_group(conv_id, content, parent_message_id, version_group_id);
        message::insert(&self.db, &row).await?;
        Ok(row)
    }

    async fn insert_assistant_placeholder(
        &self,
        conv_id: &str,
        parent_message_id: Option<String>,
        version_group_id: String,
        version_index: i64,
        provider: &str,
        model_id: &str,
    ) -> Result<MessageRow> {
        let row = MessageRow {
            id: Uuid::now_v7().to_string(),
            conversation_id: conv_id.to_string(),
            parent_message_id,
            version_group_id,
            version_index,
            is_active: true,
            role: Role::Assistant.as_str().to_string(),
            content: Some(String::new()),
            provider: Some(provider.to_string()),
            model_id: Some(model_id.to_string()),
            created_at: Utc::now().timestamp_millis(),
            ..Default::default()
        };
        message::insert(&self.db, &row).await?;
        Ok(row)
    }
}

fn map_row_to_message(row: &MessageRow) -> Result<Message> {
    Ok(Message {
        role: Role::parse(&row.role)?,
        content: MessageContent::Text(row.content.clone().unwrap_or_default()),
        tool_calls: None,
        tool_call_id: row.tool_call_id.clone(),
        tool_result: None,
    })
}

fn new_user_row_with_group(
    conv_id: &str,
    content: String,
    parent_message_id: Option<String>,
    version_group_id: Option<String>,
) -> MessageRow {
    new_user_row_with_group_and_index(conv_id, content, parent_message_id, version_group_id, 1)
}

fn new_user_row_with_group_and_index(
    conv_id: &str,
    content: String,
    parent_message_id: Option<String>,
    version_group_id: Option<String>,
    version_index: i64,
) -> MessageRow {
    MessageRow {
        id: Uuid::now_v7().to_string(),
        conversation_id: conv_id.to_string(),
        parent_message_id,
        version_group_id: version_group_id.unwrap_or_else(|| Uuid::now_v7().to_string()),
        version_index,
        is_active: true,
        role: Role::User.as_str().to_string(),
        content: Some(content),
        created_at: Utc::now().timestamp_millis(),
        ..Default::default()
    }
}

async fn insert_message_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    row: &MessageRow,
) -> Result<()> {
    let parent_turn_id = match row.parent_message_id.as_deref() {
        Some(parent_message_id) => Some(
            sqlx::query_scalar::<_, String>("SELECT turn_id FROM turn_versions WHERE id = ?")
                .bind(parent_message_id)
                .fetch_optional(&mut **tx)
                .await?
                .ok_or_else(|| AppError::NotFound {
                    entity: "message",
                    id: parent_message_id.to_string(),
                })?,
        ),
        None => None,
    };

    let turn_exists =
        sqlx::query_scalar::<_, String>("SELECT id FROM conversation_turns WHERE id = ?")
            .bind(&row.version_group_id)
            .fetch_optional(&mut **tx)
            .await?;

    if turn_exists.is_none() {
        sqlx::query(
            r#"
            INSERT INTO conversation_turns (
                id, conversation_id, parent_turn_id, role, active_version_id,
                deleted_at, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, NULL, ?, ?)
            "#,
        )
        .bind(&row.version_group_id)
        .bind(&row.conversation_id)
        .bind(parent_turn_id)
        .bind(&row.role)
        .bind(if row.is_active {
            Some(row.id.as_str())
        } else {
            None
        })
        .bind(row.created_at)
        .bind(row.created_at)
        .execute(&mut **tx)
        .await?;
    }

    sqlx::query(
        r#"
        INSERT INTO turn_versions (
            id, turn_id, version_index, content, content_parts, tool_calls,
            tool_call_id, citations_json, tokens_used, provider, model_id, created_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&row.id)
    .bind(&row.version_group_id)
    .bind(row.version_index)
    .bind(&row.content)
    .bind(&row.content_parts)
    .bind(&row.tool_calls)
    .bind(&row.tool_call_id)
    .bind(&row.citations_json)
    .bind(row.tokens_used)
    .bind(&row.provider)
    .bind(&row.model_id)
    .bind(row.created_at)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn activate_version_group_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    version_group_id: &str,
    message_id: &str,
) -> Result<()> {
    let affected = sqlx::query(
        "UPDATE conversation_turns SET active_version_id = ?, updated_at = ? WHERE id = ?",
    )
    .bind(message_id)
    .bind(Utc::now().timestamp_millis())
    .bind(version_group_id)
    .execute(&mut **tx)
    .await?
    .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "message",
            id: message_id.to_string(),
        });
    }

    Ok(())
}

fn emit_stream_event(
    app_handle: Option<&AppHandle>,
    conv_id: &str,
    event: StreamEvent,
) -> Result<()> {
    if let Some(app_handle) = app_handle {
        app_handle.emit(&format!("chat:stream:{conv_id}"), event)?;
    }
    Ok(())
}
