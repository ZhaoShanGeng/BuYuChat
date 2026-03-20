use serde_json::json;
use sqlx::SqlitePool;

use crate::db::repos::messages as message_repo;
use crate::domain::content::{ContentType, ContentWriteInput, StoredContent};
use crate::domain::messages::{
    AddAttachmentInput, BuildGenerationContextInput, BuiltGenerationContext, ContextPolicy,
    GenerateReplyInput, GenerationStreamEvent, GenerationStreamEventKind, MessageRole,
    MessageVersionView, PersistGenerationFailureInput, PersistGenerationSuccessInput,
    ProviderChatEvent, ProviderChatRequest, ProviderChatResponse, ProviderMessagePart,
    ProviderMessagePartKind, RegenerateReplyInput, ViewerPolicy,
};
use crate::providers::ProviderRegistry;
use crate::services::content;
use crate::services::content_store::ContentStore;
use crate::services::{context_builder, messages};
use crate::support::error::{AppError, Result};

pub type GenerationEventCallback<'a> = dyn FnMut(GenerationStreamEvent) -> Result<()> + Send + 'a;

pub async fn generate_reply(
    db: &SqlitePool,
    store: &ContentStore,
    providers: &ProviderRegistry,
    input: &GenerateReplyInput,
) -> Result<MessageVersionView> {
    let context = context_builder::build_generation_context(
        db,
        store,
        &BuildGenerationContextInput {
            conversation_id: input.conversation_id.clone(),
            responder_participant_id: input.responder_participant_id.clone(),
            trigger_message_version_id: input.trigger_message_version_id.clone(),
            override_api_channel_id: input.override_api_channel_id.clone(),
            override_api_channel_model_id: input.override_api_channel_model_id.clone(),
            request_parameters_json: input.request_parameters_json.clone(),
        },
    )
    .await?;

    let run_type = if input.trigger_message_version_id.is_some() {
        "regenerate"
    } else {
        "reply"
    };

    generate_reply_from_context(
        db,
        store,
        providers,
        &context,
        run_type,
        input.create_hidden_message,
        json!({}),
    )
    .await
}

pub async fn generate_reply_streaming(
    db: &SqlitePool,
    store: &ContentStore,
    providers: &ProviderRegistry,
    input: &GenerateReplyInput,
    stream_id: &str,
    on_event: &mut GenerationEventCallback<'_>,
) -> Result<MessageVersionView> {
    let context = context_builder::build_generation_context(
        db,
        store,
        &BuildGenerationContextInput {
            conversation_id: input.conversation_id.clone(),
            responder_participant_id: input.responder_participant_id.clone(),
            trigger_message_version_id: input.trigger_message_version_id.clone(),
            override_api_channel_id: input.override_api_channel_id.clone(),
            override_api_channel_model_id: input.override_api_channel_model_id.clone(),
            request_parameters_json: input.request_parameters_json.clone(),
        },
    )
    .await?;

    let run_type = if input.trigger_message_version_id.is_some() {
        "regenerate"
    } else {
        "reply"
    };

    generate_reply_from_context_streaming(
        db,
        store,
        providers,
        &context,
        run_type,
        input.create_hidden_message,
        json!({}),
        stream_id,
        on_event,
    )
    .await
}

pub async fn generate_reply_from_context(
    db: &SqlitePool,
    store: &ContentStore,
    providers: &ProviderRegistry,
    context: &BuiltGenerationContext,
    run_type: &str,
    create_hidden_message: bool,
    message_config_json: serde_json::Value,
) -> Result<MessageVersionView> {
    let request = build_provider_request(&context)?;
    let request_payload_content =
        store_payload_content(db, store, &serde_json::to_value(&request)?).await?;
    let run = message_repo::create_generation_run(
        db,
        &message_repo::CreateGenerationRunRecord {
            conversation_id: &context.conversation_id,
            trigger_node_id: context.trigger_node_id.as_deref(),
            trigger_message_version_id: context.trigger_message_version_id.as_deref(),
            responder_participant_id: Some(&context.responder_participant_id),
            api_channel_id: Some(&context.api_channel.id),
            api_channel_model_id: Some(&context.api_channel_model.id),
            preset_id: context.preset_id.as_deref(),
            preset_source_scope: context.preset_source_scope.as_deref(),
            lorebook_id: context.lorebook_id.as_deref(),
            lorebook_source_scope: context.lorebook_source_scope.as_deref(),
            user_profile_id: context.user_profile_id.as_deref(),
            user_profile_source_scope: context.user_profile_source_scope.as_deref(),
            api_channel_source_scope: context.api_channel_source_scope.as_deref(),
            api_channel_model_source_scope: context.api_channel_model_source_scope.as_deref(),
            run_type,
            request_parameters_json: &context.request_parameters_json.to_string(),
            request_payload_content_id: Some(&request_payload_content.content_id),
        },
    )
    .await?;

    record_generation_context(db, &run.id, &context.items).await?;
    let provider = providers.get(&context.api_channel.channel_type)?;

    match provider.chat(request).await {
        Ok(response) => {
            persist_generation_success(
                db,
                store,
                &context,
                &PersistGenerationSuccessInput {
                    generation_run_id: run.id,
                    conversation_id: context.conversation_id.clone(),
                    responder_participant_id: context.responder_participant_id.clone(),
                    reply_to_node_id: context.trigger_node_id.clone(),
                    assistant_response: response,
                    context_policy: ContextPolicy::Full,
                    viewer_policy: if create_hidden_message {
                        ViewerPolicy::Hidden
                    } else {
                        ViewerPolicy::Full
                    },
                    config_json: message_config_json,
                },
            )
            .await
        }
        Err(err) => {
            persist_generation_failure(
                db,
                store,
                &PersistGenerationFailureInput {
                    generation_run_id: run.id,
                    error_text: err.to_string(),
                    response_payload_json: None,
                },
            )
            .await?;
            Err(err)
        }
    }
}

pub async fn regenerate_reply(
    db: &SqlitePool,
    store: &ContentStore,
    providers: &ProviderRegistry,
    input: &RegenerateReplyInput,
) -> Result<MessageVersionView> {
    generate_reply(
        db,
        store,
        providers,
        &GenerateReplyInput {
            conversation_id: input.conversation_id.clone(),
            responder_participant_id: input.responder_participant_id.clone(),
            trigger_message_version_id: Some(input.trigger_message_version_id.clone()),
            override_api_channel_id: input.override_api_channel_id.clone(),
            override_api_channel_model_id: input.override_api_channel_model_id.clone(),
            request_parameters_json: input.request_parameters_json.clone(),
            create_hidden_message: false,
        },
    )
    .await
}

pub async fn regenerate_reply_streaming(
    db: &SqlitePool,
    store: &ContentStore,
    providers: &ProviderRegistry,
    input: &RegenerateReplyInput,
    stream_id: &str,
    on_event: &mut GenerationEventCallback<'_>,
) -> Result<MessageVersionView> {
    generate_reply_streaming(
        db,
        store,
        providers,
        &GenerateReplyInput {
            conversation_id: input.conversation_id.clone(),
            responder_participant_id: input.responder_participant_id.clone(),
            trigger_message_version_id: Some(input.trigger_message_version_id.clone()),
            override_api_channel_id: input.override_api_channel_id.clone(),
            override_api_channel_model_id: input.override_api_channel_model_id.clone(),
            request_parameters_json: input.request_parameters_json.clone(),
            create_hidden_message: false,
        },
        stream_id,
        on_event,
    )
    .await
}

pub fn build_provider_request(input: &BuiltGenerationContext) -> Result<ProviderChatRequest> {
    let messages = input
        .items
        .iter()
        .map(|item| {
            Ok(crate::domain::messages::ProviderChatMessage {
                role: item.send_role,
                name: None,
                parts: vec![content_to_part(&item.rendered_content)?],
                metadata_json: item.config_json.clone(),
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(ProviderChatRequest {
        api_channel: input.api_channel.clone(),
        api_channel_model: input.api_channel_model.clone(),
        request_parameters_json: input.request_parameters_json.clone(),
        messages,
    })
}

pub async fn record_generation_context(
    db: &SqlitePool,
    run_id: &str,
    items: &[crate::domain::messages::GenerationContextItem],
) -> Result<()> {
    let owned = items
        .iter()
        .map(|item| OwnedContextItem {
            sequence_no: item.sequence_no,
            send_role: item.send_role.as_str().to_string(),
            rendered_content_id: item.rendered_content.content_id.clone(),
            source_kind: item.source_kind.clone(),
            source_message_node_id: item.source_message_node_id.clone(),
            source_message_version_id: item.source_message_version_id.clone(),
            source_summary_version_id: item.source_summary_version_id.clone(),
            source_preset_entry_id: item.source_preset_entry_id.clone(),
            source_lorebook_entry_id: item.source_lorebook_entry_id.clone(),
            source_user_profile_id: item.source_user_profile_id.clone(),
            source_agent_id: item.source_agent_id.clone(),
            source_agent_greeting_id: item.source_agent_greeting_id.clone(),
            source_tool_invocation_id: item.source_tool_invocation_id.clone(),
            source_rag_ref_id: item.source_rag_ref_id.clone(),
            source_mcp_event_id: item.source_mcp_event_id.clone(),
            source_plugin_id: item.source_plugin_id.clone(),
            included_in_request: item.included_in_request,
            config_json: item.config_json.to_string(),
        })
        .collect::<Vec<_>>();

    let records = owned
        .iter()
        .map(|item| message_repo::GenerationRunContextItemRecord {
            sequence_no: item.sequence_no,
            send_role: &item.send_role,
            rendered_content_id: &item.rendered_content_id,
            source_kind: &item.source_kind,
            source_message_node_id: item.source_message_node_id.as_deref(),
            source_message_version_id: item.source_message_version_id.as_deref(),
            source_summary_version_id: item.source_summary_version_id.as_deref(),
            source_preset_entry_id: item.source_preset_entry_id.as_deref(),
            source_lorebook_entry_id: item.source_lorebook_entry_id.as_deref(),
            source_user_profile_id: item.source_user_profile_id.as_deref(),
            source_agent_id: item.source_agent_id.as_deref(),
            source_agent_greeting_id: item.source_agent_greeting_id.as_deref(),
            source_tool_invocation_id: item.source_tool_invocation_id.as_deref(),
            source_rag_ref_id: item.source_rag_ref_id.as_deref(),
            source_mcp_event_id: item.source_mcp_event_id.as_deref(),
            source_plugin_id: item.source_plugin_id.as_deref(),
            included_in_request: item.included_in_request,
            config_json: &item.config_json,
        })
        .collect::<Vec<_>>();

    let mut tx = db.begin().await?;
    message_repo::replace_generation_run_context_items(&mut tx, run_id, &records).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn persist_generation_success(
    db: &SqlitePool,
    store: &ContentStore,
    context: &BuiltGenerationContext,
    input: &PersistGenerationSuccessInput,
) -> Result<MessageVersionView> {
    let response_payload_content = match &input.assistant_response.raw_response_json {
        Some(payload) => Some(store_payload_content(db, store, payload).await?),
        None => None,
    };

    let _ = message_repo::finish_generation_run_success(
        db,
        &input.generation_run_id,
        &message_repo::FinishGenerationRunSuccessRecord {
            response_payload_content_id: response_payload_content
                .as_ref()
                .map(|item| item.content_id.as_str()),
        },
    )
    .await?;

    let (primary_content, refs) = split_response_parts(&input.assistant_response)?;
    let created = messages::create_assistant_message(
        db,
        store,
        &crate::domain::messages::CreateMessageInput {
            conversation_id: input.conversation_id.clone(),
            author_participant_id: input.responder_participant_id.clone(),
            role: MessageRole::Assistant,
            reply_to_node_id: input.reply_to_node_id.clone(),
            order_after_node_id: input.reply_to_node_id.clone(),
            primary_content,
            context_policy: input.context_policy,
            viewer_policy: input.viewer_policy,
            config_json: input.config_json.clone(),
        },
    )
    .await?;

    let mut tx = db.begin().await?;
    message_repo::update_message_version_generation_metadata(
        &mut tx,
        &created.version_id,
        Some(&context.api_channel.id),
        Some(&context.api_channel_model.id),
        Some(&input.generation_run_id),
        input.assistant_response.prompt_tokens,
        input.assistant_response.completion_tokens,
        input.assistant_response.total_tokens,
        input.assistant_response.finish_reason.as_deref(),
    )
    .await?;
    tx.commit().await?;

    for (sort_order, (ref_role, content_input)) in refs.into_iter().enumerate() {
        let _ = messages::append_attachment(
            db,
            store,
            &AddAttachmentInput {
                message_version_id: created.version_id.clone(),
                plugin_id: None,
                ref_role,
                sort_order: sort_order as i64,
                content: content_input,
                config_json: json!({}),
            },
        )
        .await?;
    }

    messages::get_message_version_view(db, store, &created.version_id).await
}

pub async fn persist_generation_failure(
    db: &SqlitePool,
    store: &ContentStore,
    input: &PersistGenerationFailureInput,
) -> Result<()> {
    let response_payload_content = match &input.response_payload_json {
        Some(payload) => Some(store_payload_content(db, store, payload).await?),
        None => None,
    };

    let _ = message_repo::finish_generation_run_failure(
        db,
        &input.generation_run_id,
        &message_repo::FinishGenerationRunFailureRecord {
            error_text: &input.error_text,
            response_payload_content_id: response_payload_content
                .as_ref()
                .map(|item| item.content_id.as_str()),
        },
    )
    .await?;
    Ok(())
}

async fn generate_reply_from_context_streaming(
    db: &SqlitePool,
    store: &ContentStore,
    providers: &ProviderRegistry,
    context: &BuiltGenerationContext,
    run_type: &str,
    create_hidden_message: bool,
    message_config_json: serde_json::Value,
    stream_id: &str,
    on_event: &mut GenerationEventCallback<'_>,
) -> Result<MessageVersionView> {
    let request = build_provider_request(context)?;
    let request_payload_content =
        store_payload_content(db, store, &serde_json::to_value(&request)?).await?;
    let run = message_repo::create_generation_run(
        db,
        &message_repo::CreateGenerationRunRecord {
            conversation_id: &context.conversation_id,
            trigger_node_id: context.trigger_node_id.as_deref(),
            trigger_message_version_id: context.trigger_message_version_id.as_deref(),
            responder_participant_id: Some(&context.responder_participant_id),
            api_channel_id: Some(&context.api_channel.id),
            api_channel_model_id: Some(&context.api_channel_model.id),
            preset_id: context.preset_id.as_deref(),
            preset_source_scope: context.preset_source_scope.as_deref(),
            lorebook_id: context.lorebook_id.as_deref(),
            lorebook_source_scope: context.lorebook_source_scope.as_deref(),
            user_profile_id: context.user_profile_id.as_deref(),
            user_profile_source_scope: context.user_profile_source_scope.as_deref(),
            api_channel_source_scope: context.api_channel_source_scope.as_deref(),
            api_channel_model_source_scope: context.api_channel_model_source_scope.as_deref(),
            run_type,
            request_parameters_json: &context.request_parameters_json.to_string(),
            request_payload_content_id: Some(&request_payload_content.content_id),
        },
    )
    .await?;

    record_generation_context(db, &run.id, &context.items).await?;
    let provider = providers.get(&context.api_channel.channel_type)?;
    let mut accumulated_text = String::new();
    let mut streamed_finish_reason = None;
    let mut streamed_prompt_tokens = None;
    let mut streamed_completion_tokens = None;
    let mut streamed_total_tokens = None;

    on_event(GenerationStreamEvent {
        stream_id: stream_id.to_string(),
        kind: GenerationStreamEventKind::Started,
        delta_text: None,
        accumulated_text: Some(String::new()),
        message_version_id: None,
        finish_reason: None,
        prompt_tokens: None,
        completion_tokens: None,
        total_tokens: None,
        error_text: None,
    })?;

    match provider
        .chat_stream(request, &mut |event| {
            match event {
                ProviderChatEvent::Delta { parts, .. } => {
                    let delta_text = extract_text_from_parts(&parts);
                    if !delta_text.is_empty() {
                        accumulated_text.push_str(&delta_text);
                        on_event(GenerationStreamEvent {
                            stream_id: stream_id.to_string(),
                            kind: GenerationStreamEventKind::Delta,
                            delta_text: Some(delta_text),
                            accumulated_text: Some(accumulated_text.clone()),
                            message_version_id: None,
                            finish_reason: None,
                            prompt_tokens: None,
                            completion_tokens: None,
                            total_tokens: None,
                            error_text: None,
                        })?;
                    }
                }
                ProviderChatEvent::Finished {
                    finish_reason,
                    prompt_tokens,
                    completion_tokens,
                    total_tokens,
                    ..
                } => {
                    streamed_finish_reason = finish_reason;
                    streamed_prompt_tokens = prompt_tokens;
                    streamed_completion_tokens = completion_tokens;
                    streamed_total_tokens = total_tokens;
                }
            }
            Ok(())
        })
        .await
    {
        Ok(response) => {
            let message = persist_generation_success(
                db,
                store,
                context,
                &PersistGenerationSuccessInput {
                    generation_run_id: run.id,
                    conversation_id: context.conversation_id.clone(),
                    responder_participant_id: context.responder_participant_id.clone(),
                    reply_to_node_id: context.trigger_node_id.clone(),
                    assistant_response: response.clone(),
                    context_policy: ContextPolicy::Full,
                    viewer_policy: if create_hidden_message {
                        ViewerPolicy::Hidden
                    } else {
                        ViewerPolicy::Full
                    },
                    config_json: message_config_json,
                },
            )
            .await?;

            on_event(GenerationStreamEvent {
                stream_id: stream_id.to_string(),
                kind: GenerationStreamEventKind::Completed,
                delta_text: None,
                accumulated_text: Some(accumulated_text),
                message_version_id: Some(message.version_id.clone()),
                finish_reason: response.finish_reason.or(streamed_finish_reason),
                prompt_tokens: response.prompt_tokens.or(streamed_prompt_tokens),
                completion_tokens: response.completion_tokens.or(streamed_completion_tokens),
                total_tokens: response.total_tokens.or(streamed_total_tokens),
                error_text: None,
            })?;

            Ok(message)
        }
        Err(err) => {
            persist_generation_failure(
                db,
                store,
                &PersistGenerationFailureInput {
                    generation_run_id: run.id,
                    error_text: err.to_string(),
                    response_payload_json: None,
                },
            )
            .await?;
            on_event(GenerationStreamEvent {
                stream_id: stream_id.to_string(),
                kind: GenerationStreamEventKind::Failed,
                delta_text: None,
                accumulated_text: Some(accumulated_text),
                message_version_id: None,
                finish_reason: None,
                prompt_tokens: None,
                completion_tokens: None,
                total_tokens: None,
                error_text: Some(err.to_string()),
            })?;
            Err(err)
        }
    }
}

fn content_to_part(content: &StoredContent) -> Result<ProviderMessagePart> {
    let kind = match content.content_type {
        ContentType::Text | ContentType::Markdown | ContentType::Html => {
            ProviderMessagePartKind::Text
        }
        ContentType::Image => ProviderMessagePartKind::ImageRef,
        ContentType::Audio => ProviderMessagePartKind::AudioRef,
        ContentType::Video => ProviderMessagePartKind::VideoRef,
        ContentType::File | ContentType::Binary => ProviderMessagePartKind::FileRef,
        ContentType::Json => ProviderMessagePartKind::JsonPayload,
        ContentType::ToolRequest => ProviderMessagePartKind::ToolRequest,
        ContentType::ToolResponse => ProviderMessagePartKind::ToolResponse,
        ContentType::RagExcerpt => ProviderMessagePartKind::RagExcerpt,
        ContentType::McpPayload => ProviderMessagePartKind::McpPayload,
        ContentType::PluginPayload | ContentType::PluginState => {
            ProviderMessagePartKind::PluginPayload
        }
        ContentType::ReasoningTrace => ProviderMessagePartKind::ReasoningTrace,
        ContentType::ProviderSignature => ProviderMessagePartKind::ProviderSignature,
    };

    Ok(ProviderMessagePart {
        kind,
        text: content.text_content.clone(),
        content: Some(content.clone()),
        metadata_json: content.config_json.clone(),
    })
}

async fn store_payload_content(
    db: &SqlitePool,
    store: &ContentStore,
    payload: &serde_json::Value,
) -> Result<StoredContent> {
    content::create_content(
        db,
        store,
        &ContentWriteInput {
            content_type: ContentType::Json,
            mime_type: Some("application/json".to_string()),
            text_content: Some(payload.to_string()),
            source_file_path: None,
            primary_storage_uri: None,
            size_bytes_hint: None,
            preview_text: None,
            config_json: json!({}),
        },
    )
    .await
}

fn split_response_parts(
    response: &ProviderChatResponse,
) -> Result<(ContentWriteInput, Vec<(String, ContentWriteInput)>)> {
    let primary = response
        .parts
        .first()
        .and_then(part_to_content_write_input)
        .ok_or_else(|| {
            AppError::Validation("provider response contained no usable content".to_string())
        })?;

    let refs = response
        .parts
        .iter()
        .skip(1)
        .filter_map(|part| {
            part_to_content_write_input(part).map(|input| {
                let ref_role = match part.kind {
                    ProviderMessagePartKind::ReasoningTrace => "reasoning_trace",
                    ProviderMessagePartKind::ProviderSignature => "provider_signature",
                    ProviderMessagePartKind::ToolRequest => "tool_request",
                    ProviderMessagePartKind::ToolResponse => "tool_response",
                    _ => "attachment",
                };
                (ref_role.to_string(), input)
            })
        })
        .collect::<Vec<_>>();

    Ok((primary, refs))
}

fn extract_text_from_parts(parts: &[ProviderMessagePart]) -> String {
    let mut text = String::new();
    for part in parts {
        if let Some(delta) = part.text.clone().or_else(|| {
            part.content
                .as_ref()
                .and_then(|content| content.text_content.clone())
        }) {
            text.push_str(&delta);
        }
    }
    text
}

fn part_to_content_write_input(part: &ProviderMessagePart) -> Option<ContentWriteInput> {
    if let Some(content) = &part.content {
        return Some(ContentWriteInput {
            content_type: content.content_type,
            mime_type: content.mime_type.clone(),
            text_content: content
                .text_content
                .clone()
                .or_else(|| content.preview_text.clone()),
            source_file_path: None,
            primary_storage_uri: content.primary_storage_uri.clone(),
            size_bytes_hint: Some(content.size_bytes),
            preview_text: content.preview_text.clone(),
            config_json: content.config_json.clone(),
        });
    }

    let content_type = match part.kind {
        ProviderMessagePartKind::Text => ContentType::Text,
        ProviderMessagePartKind::ImageRef => ContentType::Image,
        ProviderMessagePartKind::AudioRef => ContentType::Audio,
        ProviderMessagePartKind::VideoRef => ContentType::Video,
        ProviderMessagePartKind::FileRef => ContentType::File,
        ProviderMessagePartKind::JsonPayload => ContentType::Json,
        ProviderMessagePartKind::ToolRequest => ContentType::ToolRequest,
        ProviderMessagePartKind::ToolResponse => ContentType::ToolResponse,
        ProviderMessagePartKind::RagExcerpt => ContentType::RagExcerpt,
        ProviderMessagePartKind::McpPayload => ContentType::McpPayload,
        ProviderMessagePartKind::PluginPayload => ContentType::PluginPayload,
        ProviderMessagePartKind::ReasoningTrace => ContentType::ReasoningTrace,
        ProviderMessagePartKind::ProviderSignature => ContentType::ProviderSignature,
    };

    Some(ContentWriteInput {
        content_type,
        mime_type: match content_type {
            ContentType::Json => Some("application/json".to_string()),
            _ => Some("text/plain".to_string()),
        },
        text_content: part.text.clone(),
        source_file_path: None,
        primary_storage_uri: None,
        size_bytes_hint: None,
        preview_text: None,
        config_json: part.metadata_json.clone(),
    })
}

struct OwnedContextItem {
    sequence_no: i64,
    send_role: String,
    rendered_content_id: String,
    source_kind: String,
    source_message_node_id: Option<String>,
    source_message_version_id: Option<String>,
    source_summary_version_id: Option<String>,
    source_preset_entry_id: Option<String>,
    source_lorebook_entry_id: Option<String>,
    source_user_profile_id: Option<String>,
    source_agent_id: Option<String>,
    source_agent_greeting_id: Option<String>,
    source_tool_invocation_id: Option<String>,
    source_rag_ref_id: Option<String>,
    source_mcp_event_id: Option<String>,
    source_plugin_id: Option<String>,
    included_in_request: bool,
    config_json: String,
}
