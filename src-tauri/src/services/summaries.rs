use std::collections::HashMap;

use serde_json::json;
use sqlx::SqlitePool;

use crate::db::models::{SummaryGroupRow, SummaryUsageRow, SummaryVersionRow};
use crate::db::repos::{
    conversations as conversation_repo, messages as message_repo, summaries as repo,
};
use crate::domain::content::{ContentType, ContentWriteInput, StoredContent};
use crate::domain::messages::{
    GenerationContextItem, MessageRole, MessageVersionView, ProviderChatMessage,
    ProviderChatRequest, ProviderChatResponse, ProviderMessagePart, ProviderMessagePartKind,
};
use crate::domain::summaries::{
    SummaryActivationMode, SummaryGroup, SummaryKind, SummaryScopeType, SummaryTargetKind,
    SummaryUsage, SummaryUsageScope, SummaryVersion, UpsertSummaryUsageInput,
};
use crate::providers::ProviderRegistry;
use crate::services::content;
use crate::services::content_store::ContentStore;
use crate::services::{context_builder, generation, messages, presets};
use crate::support::error::{AppError, Result};
use crate::support::time;

#[derive(Debug, Clone)]
pub(crate) struct ResolvedSummary {
    pub summary_version_id: String,
    pub content: StoredContent,
}

pub async fn list_summary_groups(
    db: &SqlitePool,
    store: &ContentStore,
    conversation_id: &str,
) -> Result<Vec<SummaryGroup>> {
    let rows = repo::list_summary_groups_by_conversation(db, conversation_id).await?;
    let mut groups = Vec::with_capacity(rows.len());
    for row in rows {
        groups.push(map_summary_group(db, store, row).await?);
    }
    Ok(groups)
}

pub async fn generate_node_summary(
    db: &SqlitePool,
    store: &ContentStore,
    providers: &ProviderRegistry,
    node_id: &str,
    generator_preset_id: Option<&str>,
) -> Result<SummaryVersion> {
    let active_version = message_repo::get_active_message_version(db, node_id).await?;
    let message = messages::get_message_version_view(db, store, &active_version.id).await?;
    let conversation_id = message.conversation_id.clone();
    generate_summary_from_messages(
        db,
        store,
        providers,
        &conversation_id,
        ScopeSpec::Node {
            node_id: node_id.to_string(),
        },
        vec![message],
        generator_preset_id,
    )
    .await
}

pub async fn generate_range_summary(
    db: &SqlitePool,
    store: &ContentStore,
    providers: &ProviderRegistry,
    start_node_id: &str,
    end_node_id: &str,
    generator_preset_id: Option<&str>,
) -> Result<SummaryVersion> {
    let start = message_repo::get_message_node(db, start_node_id).await?;
    let end = message_repo::get_message_node(db, end_node_id).await?;
    if start.conversation_id != end.conversation_id {
        return Err(AppError::Validation(
            "start_node_id and end_node_id must belong to the same conversation".to_string(),
        ));
    }

    let visible_messages =
        messages::list_visible_messages(db, store, &start.conversation_id).await?;
    let start_key = start.order_key.clone();
    let end_key = end.order_key.clone();
    let (lower, upper) = if start_key <= end_key {
        (start_key, end_key)
    } else {
        (end_key, start_key)
    };
    let selected = visible_messages
        .into_iter()
        .filter(|item| item.order_key >= lower && item.order_key <= upper)
        .collect::<Vec<_>>();

    if selected.is_empty() {
        return Err(AppError::Validation(
            "no visible messages found in the selected node range".to_string(),
        ));
    }

    generate_summary_from_messages(
        db,
        store,
        providers,
        &start.conversation_id,
        ScopeSpec::NodeRange {
            start_node_id: start_node_id.to_string(),
            end_node_id: end_node_id.to_string(),
        },
        selected,
        generator_preset_id,
    )
    .await
}

pub async fn generate_conversation_summary(
    db: &SqlitePool,
    store: &ContentStore,
    providers: &ProviderRegistry,
    conversation_id: &str,
    generator_preset_id: Option<&str>,
) -> Result<SummaryVersion> {
    let visible_messages = messages::list_visible_messages(db, store, conversation_id).await?;
    if visible_messages.is_empty() {
        return Err(AppError::Validation(
            "cannot summarize an empty conversation".to_string(),
        ));
    }

    generate_summary_from_messages(
        db,
        store,
        providers,
        conversation_id,
        ScopeSpec::Conversation,
        visible_messages,
        generator_preset_id,
    )
    .await
}

pub async fn switch_active_summary(
    db: &SqlitePool,
    store: &ContentStore,
    summary_group_id: &str,
    summary_version_id: &str,
) -> Result<SummaryVersion> {
    let _ = repo::get_summary_group(db, summary_group_id).await?;
    let version = repo::get_summary_version(db, summary_version_id).await?;
    if version.summary_group_id != summary_group_id {
        return Err(AppError::Validation(
            "summary_version_id does not belong to summary_group_id".to_string(),
        ));
    }

    let mut tx = db.begin().await?;
    repo::set_active_summary_version(&mut tx, summary_group_id, summary_version_id).await?;
    tx.commit().await?;

    let version = repo::get_summary_version(db, summary_version_id).await?;
    map_summary_version(db, store, version).await
}

pub async fn upsert_summary_usage(
    db: &SqlitePool,
    input: &UpsertSummaryUsageInput,
) -> Result<SummaryUsage> {
    validate_summary_usage_input(db, input).await?;

    let row = repo::upsert_summary_usage(
        db,
        &repo::UpsertSummaryUsageRecord {
            usage_id: input.usage_id.as_deref(),
            summary_group_id: &input.summary_group_id,
            summary_version_id: input.summary_version_id.as_deref(),
            usage_scope: input.usage_scope.as_str(),
            target_kind: input.target_kind.as_str(),
            target_message_version_id: input.target_message_version_id.as_deref(),
            target_start_node_id: input.target_start_node_id.as_deref(),
            target_end_node_id: input.target_end_node_id.as_deref(),
            conversation_id: input.conversation_id.as_deref(),
            activation_mode: input.activation_mode.as_str(),
            replace_from_node_id: input.replace_from_node_id.as_deref(),
            replace_after_message_count: input.replace_after_message_count,
            replace_after_total_bytes: input.replace_after_total_bytes,
            enabled: input.enabled,
            priority: input.priority,
            config_json: &input.config_json.to_string(),
        },
    )
    .await?;

    map_summary_usage(row)
}

pub async fn resolve_viewer_summary(
    db: &SqlitePool,
    store: &ContentStore,
    message_version_id: &str,
) -> Result<Option<SummaryVersion>> {
    let message = messages::get_message_version_view(db, store, message_version_id).await?;
    let mut resolved =
        resolve_summaries_for_messages_internal(db, store, &[message], SummaryUsageScope::Viewer)
            .await?;
    let Some(summary) = resolved.remove(message_version_id) else {
        return Ok(None);
    };
    let row = repo::get_summary_version(db, &summary.summary_version_id).await?;
    map_summary_version(db, store, row).await.map(Some)
}

pub async fn resolve_context_summary(
    db: &SqlitePool,
    store: &ContentStore,
    message_version_id: &str,
) -> Result<Option<SummaryVersion>> {
    let message = messages::get_message_version_view(db, store, message_version_id).await?;
    let mut resolved =
        resolve_summaries_for_messages_internal(db, store, &[message], SummaryUsageScope::Request)
            .await?;
    let Some(summary) = resolved.remove(message_version_id) else {
        return Ok(None);
    };
    let row = repo::get_summary_version(db, &summary.summary_version_id).await?;
    map_summary_version(db, store, row).await.map(Some)
}

pub(crate) async fn resolve_context_summaries_for_messages(
    db: &SqlitePool,
    store: &ContentStore,
    messages: &[MessageVersionView],
) -> Result<HashMap<String, ResolvedSummary>> {
    resolve_summaries_for_messages_internal(db, store, messages, SummaryUsageScope::Request).await
}

async fn generate_summary_from_messages(
    db: &SqlitePool,
    store: &ContentStore,
    providers: &ProviderRegistry,
    conversation_id: &str,
    scope: ScopeSpec,
    source_messages: Vec<MessageVersionView>,
    generator_preset_id: Option<&str>,
) -> Result<SummaryVersion> {
    let participant = select_summary_participant(db, conversation_id).await?;
    let group =
        upsert_summary_group_for_scope(db, conversation_id, &scope, generator_preset_id).await?;
    let effective_generator_preset_id =
        generator_preset_id.or(group.default_generator_preset_id.as_deref());
    let generator_preset = match effective_generator_preset_id {
        Some(id) => Some(presets::get_preset_detail(db, store, id).await?),
        None => None,
    };

    let (api_channel, api_channel_model, _, _) = context_builder::resolve_active_channel_model(
        db,
        conversation_id,
        &participant.id,
        None,
        None,
    )
    .await?;

    let context_items = build_summary_generation_context_items(
        db,
        store,
        &source_messages,
        generator_preset.as_ref(),
    )
    .await?;
    let request_parameters_json = api_channel_model.default_parameters_json.clone();
    let request = build_summary_provider_request(
        api_channel.clone(),
        api_channel_model.clone(),
        request_parameters_json.clone(),
        &context_items,
    );

    let request_payload_content =
        store_payload_content(db, store, &serde_json::to_value(&request)?).await?;
    let run = message_repo::create_generation_run(
        db,
        &message_repo::CreateGenerationRunRecord {
            conversation_id,
            trigger_node_id: source_messages.first().map(|item| item.node_id.as_str()),
            trigger_message_version_id: source_messages
                .first()
                .map(|item| item.version_id.as_str()),
            responder_participant_id: Some(&participant.id),
            api_channel_id: Some(&api_channel.id),
            api_channel_model_id: Some(&api_channel_model.id),
            preset_id: effective_generator_preset_id,
            preset_source_scope: Some("summary"),
            lorebook_id: None,
            lorebook_source_scope: None,
            user_profile_id: None,
            user_profile_source_scope: None,
            api_channel_source_scope: Some("conversation"),
            api_channel_model_source_scope: Some("conversation"),
            run_type: "summary",
            request_parameters_json: &request_parameters_json.to_string(),
            request_payload_content_id: Some(&request_payload_content.content_id),
        },
    )
    .await?;

    generation::record_generation_context(db, &run.id, &context_items).await?;
    let provider = providers.get(&api_channel.channel_type)?;

    let response = match provider.chat(request).await {
        Ok(response) => response,
        Err(err) => {
            generation::persist_generation_failure(
                db,
                store,
                &crate::domain::messages::PersistGenerationFailureInput {
                    generation_run_id: run.id,
                    error_text: err.to_string(),
                    response_payload_json: None,
                },
            )
            .await?;
            return Err(err);
        }
    };

    persist_summary_success(
        db,
        store,
        &group,
        effective_generator_preset_id,
        &source_messages,
        &run.id,
        response,
    )
    .await
}

async fn persist_summary_success(
    db: &SqlitePool,
    store: &ContentStore,
    group: &SummaryGroupRow,
    generator_preset_id: Option<&str>,
    source_messages: &[MessageVersionView],
    generation_run_id: &str,
    response: ProviderChatResponse,
) -> Result<SummaryVersion> {
    let response_payload_content = match &response.raw_response_json {
        Some(payload) => Some(store_payload_content(db, store, payload).await?),
        None => None,
    };

    let _ = message_repo::finish_generation_run_success(
        db,
        generation_run_id,
        &message_repo::FinishGenerationRunSuccessRecord {
            response_payload_content_id: response_payload_content
                .as_ref()
                .map(|item| item.content_id.as_str()),
        },
    )
    .await?;

    let summary_content_input = extract_summary_content_input(&response)?;
    let summary_content = content::create_content(db, store, &summary_content_input).await?;
    let next_version_index = repo::list_summary_versions(db, &group.id)
        .await?
        .into_iter()
        .map(|item| item.version_index)
        .max()
        .unwrap_or(0)
        + 1;

    let now = time::now_ms();
    let mut tx = db.begin().await?;
    let version = repo::create_summary_version(
        &mut tx,
        &repo::CreateSummaryVersionRecord {
            summary_group_id: &group.id,
            version_index: next_version_index,
            is_active: false,
            content_id: &summary_content.content_id,
            generator_type: "model",
            generator_preset_id,
            workflow_run_id: None,
            generation_run_id: Some(generation_run_id),
            config_json: &json!({}).to_string(),
            created_at: now,
        },
    )
    .await?;
    repo::set_active_summary_version(&mut tx, &group.id, &version.id).await?;

    let sources = source_messages
        .iter()
        .enumerate()
        .map(|(idx, item)| repo::SummarySourceRecord {
            source_kind: "message_version",
            source_message_version_id: Some(item.version_id.as_str()),
            source_start_node_id: None,
            source_end_node_id: None,
            source_summary_version_id: None,
            sort_order: idx as i64,
        })
        .collect::<Vec<_>>();
    repo::replace_summary_sources(&mut tx, &group.id, &version.id, &sources).await?;
    tx.commit().await?;

    let version = repo::get_summary_version(db, &version.id).await?;
    map_summary_version(db, store, version).await
}

async fn build_summary_generation_context_items(
    db: &SqlitePool,
    store: &ContentStore,
    source_messages: &[MessageVersionView],
    generator_preset: Option<&crate::domain::presets::PresetDetail>,
) -> Result<Vec<GenerationContextItem>> {
    let mut sequence_no = 0_i64;
    let mut items = Vec::new();

    if let Some(generator_preset) = generator_preset {
        for entry in generator_preset.entries.iter().filter(|item| item.enabled) {
            items.push(GenerationContextItem {
                sequence_no,
                send_role: entry.role,
                rendered_content: entry.primary_content.clone(),
                source_kind: "preset_entry".to_string(),
                source_message_node_id: None,
                source_message_version_id: None,
                source_summary_version_id: None,
                source_preset_entry_id: Some(entry.id.clone()),
                source_lorebook_entry_id: None,
                source_user_profile_id: None,
                source_agent_id: None,
                source_agent_greeting_id: None,
                source_tool_invocation_id: None,
                source_rag_ref_id: None,
                source_mcp_event_id: None,
                source_plugin_id: None,
                included_in_request: true,
                config_json: json!({}),
            });
            sequence_no += 1;
        }
    }

    items.push(GenerationContextItem {
        sequence_no,
        send_role: MessageRole::System,
        rendered_content: create_inline_instruction_content(
            db,
            store,
            "[Summary Engine]\nGenerate a concise factual summary of the provided conversation materials. Do not invent information. Return plain text only.",
        )
        .await?,
        source_kind: "summary_instruction".to_string(),
        source_message_node_id: None,
        source_message_version_id: None,
        source_summary_version_id: None,
        source_preset_entry_id: None,
        source_lorebook_entry_id: None,
        source_user_profile_id: None,
        source_agent_id: None,
        source_agent_greeting_id: None,
        source_tool_invocation_id: None,
        source_rag_ref_id: None,
        source_mcp_event_id: None,
        source_plugin_id: None,
        included_in_request: true,
        config_json: json!({}),
    });
    sequence_no += 1;

    for message in source_messages {
        items.push(GenerationContextItem {
            sequence_no,
            send_role: message.role,
            rendered_content: message.primary_content.clone(),
            source_kind: "message_version".to_string(),
            source_message_node_id: Some(message.node_id.clone()),
            source_message_version_id: Some(message.version_id.clone()),
            source_summary_version_id: None,
            source_preset_entry_id: None,
            source_lorebook_entry_id: None,
            source_user_profile_id: None,
            source_agent_id: None,
            source_agent_greeting_id: None,
            source_tool_invocation_id: None,
            source_rag_ref_id: None,
            source_mcp_event_id: None,
            source_plugin_id: None,
            included_in_request: true,
            config_json: json!({}),
        });
        sequence_no += 1;
    }

    items.push(GenerationContextItem {
        sequence_no,
        send_role: MessageRole::User,
        rendered_content: create_inline_instruction_content(
            db,
            store,
            "Summarize the materials above in concise Chinese. Keep stable facts, important constraints, and unresolved goals. Avoid markdown lists unless necessary.",
        )
        .await?,
        source_kind: "summary_instruction".to_string(),
        source_message_node_id: None,
        source_message_version_id: None,
        source_summary_version_id: None,
        source_preset_entry_id: None,
        source_lorebook_entry_id: None,
        source_user_profile_id: None,
        source_agent_id: None,
        source_agent_greeting_id: None,
        source_tool_invocation_id: None,
        source_rag_ref_id: None,
        source_mcp_event_id: None,
        source_plugin_id: None,
        included_in_request: true,
        config_json: json!({}),
    });

    Ok(items)
}

fn build_summary_provider_request(
    api_channel: crate::domain::api_channels::ApiChannel,
    api_channel_model: crate::domain::api_channels::ApiChannelModel,
    request_parameters_json: serde_json::Value,
    items: &[GenerationContextItem],
) -> ProviderChatRequest {
    ProviderChatRequest {
        api_channel,
        api_channel_model,
        request_parameters_json,
        messages: items
            .iter()
            .map(|item| ProviderChatMessage {
                role: item.send_role,
                name: None,
                parts: vec![ProviderMessagePart {
                    kind: match item.rendered_content.content_type {
                        ContentType::Text | ContentType::Markdown | ContentType::Html => {
                            ProviderMessagePartKind::Text
                        }
                        ContentType::Json => ProviderMessagePartKind::JsonPayload,
                        ContentType::Image => ProviderMessagePartKind::ImageRef,
                        ContentType::Audio => ProviderMessagePartKind::AudioRef,
                        ContentType::Video => ProviderMessagePartKind::VideoRef,
                        ContentType::File | ContentType::Binary => ProviderMessagePartKind::FileRef,
                        ContentType::ToolRequest => ProviderMessagePartKind::ToolRequest,
                        ContentType::ToolResponse => ProviderMessagePartKind::ToolResponse,
                        ContentType::RagExcerpt => ProviderMessagePartKind::RagExcerpt,
                        ContentType::McpPayload => ProviderMessagePartKind::McpPayload,
                        ContentType::PluginPayload | ContentType::PluginState => {
                            ProviderMessagePartKind::PluginPayload
                        }
                        ContentType::ReasoningTrace => ProviderMessagePartKind::ReasoningTrace,
                        ContentType::ProviderSignature => {
                            ProviderMessagePartKind::ProviderSignature
                        }
                    },
                    text: item
                        .rendered_content
                        .text_content
                        .clone()
                        .or_else(|| item.rendered_content.preview_text.clone()),
                    content: Some(item.rendered_content.clone()),
                    metadata_json: item.config_json.clone(),
                }],
                metadata_json: item.config_json.clone(),
            })
            .collect(),
    }
}

pub(crate) async fn resolve_summaries_for_messages_internal(
    db: &SqlitePool,
    store: &ContentStore,
    messages: &[MessageVersionView],
    usage_scope: SummaryUsageScope,
) -> Result<HashMap<String, ResolvedSummary>> {
    if messages.is_empty() {
        return Ok(HashMap::new());
    }

    let conversation_id = &messages[0].conversation_id;
    let usages = repo::list_summary_usages(db, conversation_id).await?;
    if usages.is_empty() {
        return Ok(HashMap::new());
    }

    let mut order_index = HashMap::<String, usize>::new();
    let mut total_bytes = 0_i64;
    for (idx, message) in messages.iter().enumerate() {
        order_index.insert(message.node_id.clone(), idx);
        total_bytes += message.primary_content.size_bytes as i64;
    }

    let mut version_cache = HashMap::<String, SummaryVersionRow>::new();
    let mut content_cache = HashMap::<String, StoredContent>::new();
    let mut resolved = HashMap::<String, ResolvedSummary>::new();

    for message in messages {
        let Some(usage) = pick_best_usage(
            &usages,
            message,
            usage_scope,
            messages.len() as i64,
            total_bytes,
            &order_index,
        ) else {
            continue;
        };

        let summary_version = if let Some(version_id) = &usage.summary_version_id {
            load_summary_version_cached(db, &mut version_cache, version_id).await?
        } else {
            load_active_summary_version_cached(db, &mut version_cache, &usage.summary_group_id)
                .await?
        };

        let Some(summary_version) = summary_version else {
            continue;
        };

        let content = if let Some(content) = content_cache.get(&summary_version.content_id) {
            content.clone()
        } else {
            let content =
                content::get_content(db, store, &summary_version.content_id, true).await?;
            content_cache.insert(summary_version.content_id.clone(), content.clone());
            content
        };

        resolved.insert(
            message.version_id.clone(),
            ResolvedSummary {
                summary_version_id: summary_version.id.clone(),
                content,
            },
        );
    }

    Ok(resolved)
}

async fn map_summary_group(
    db: &SqlitePool,
    store: &ContentStore,
    row: SummaryGroupRow,
) -> Result<SummaryGroup> {
    let active_version = match repo::get_active_summary_version(db, &row.id).await? {
        Some(version) => Some(map_summary_version(db, store, version).await?),
        None => None,
    };

    Ok(SummaryGroup {
        id: row.id,
        conversation_id: row.conversation_id,
        scope_type: SummaryScopeType::parse(&row.scope_type)?,
        scope_message_version_id: row.scope_message_version_id,
        scope_start_node_id: row.scope_start_node_id,
        scope_end_node_id: row.scope_end_node_id,
        scope_summary_group_id: row.scope_summary_group_id,
        summary_kind: SummaryKind::parse(&row.summary_kind)?,
        default_generator_preset_id: row.default_generator_preset_id,
        enabled: row.enabled,
        created_at: row.created_at,
        updated_at: row.updated_at,
        active_version,
    })
}

async fn map_summary_version(
    db: &SqlitePool,
    store: &ContentStore,
    row: SummaryVersionRow,
) -> Result<SummaryVersion> {
    Ok(SummaryVersion {
        id: row.id,
        summary_group_id: row.summary_group_id,
        version_index: row.version_index,
        is_active: row.is_active,
        content: content::get_content(db, store, &row.content_id, true).await?,
        generator_type: row.generator_type,
        generator_preset_id: row.generator_preset_id,
        workflow_run_id: row.workflow_run_id,
        generation_run_id: row.generation_run_id,
        config_json: parse_json(&row.config_json, "summary_versions.config_json")?,
        created_at: row.created_at,
    })
}

fn map_summary_usage(row: SummaryUsageRow) -> Result<SummaryUsage> {
    Ok(SummaryUsage {
        id: row.id,
        summary_group_id: row.summary_group_id,
        summary_version_id: row.summary_version_id,
        usage_scope: SummaryUsageScope::parse(&row.usage_scope)?,
        target_kind: SummaryTargetKind::parse(&row.target_kind)?,
        target_message_version_id: row.target_message_version_id,
        target_start_node_id: row.target_start_node_id,
        target_end_node_id: row.target_end_node_id,
        conversation_id: row.conversation_id,
        activation_mode: SummaryActivationMode::parse(&row.activation_mode)?,
        replace_from_node_id: row.replace_from_node_id,
        replace_after_message_count: row.replace_after_message_count,
        replace_after_total_bytes: row.replace_after_total_bytes,
        enabled: row.enabled,
        priority: row.priority,
        config_json: parse_json(&row.config_json, "summary_usages.config_json")?,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

fn pick_best_usage<'a>(
    usages: &'a [SummaryUsageRow],
    message: &MessageVersionView,
    usage_scope: SummaryUsageScope,
    total_messages: i64,
    total_bytes: i64,
    order_index: &HashMap<String, usize>,
) -> Option<&'a SummaryUsageRow> {
    let mut candidates = usages
        .iter()
        .filter(|usage| usage.enabled)
        .filter(|usage| {
            SummaryUsageScope::parse(&usage.usage_scope)
                .map(|scope| match usage_scope {
                    SummaryUsageScope::Viewer => scope.matches_viewer(),
                    SummaryUsageScope::Request => scope.matches_request(),
                    SummaryUsageScope::Both => true,
                })
                .unwrap_or(false)
        })
        .filter(|usage| usage_targets_message(usage, message, order_index))
        .filter(|usage| {
            usage_activation_passes(usage, message, total_messages, total_bytes, order_index)
        })
        .collect::<Vec<_>>();

    candidates.sort_by(|lhs, rhs| {
        rhs.priority
            .cmp(&lhs.priority)
            .then(rhs.updated_at.cmp(&lhs.updated_at))
            .then(rhs.created_at.cmp(&lhs.created_at))
    });
    candidates.into_iter().next()
}

fn usage_targets_message(
    usage: &SummaryUsageRow,
    message: &MessageVersionView,
    order_index: &HashMap<String, usize>,
) -> bool {
    match SummaryTargetKind::parse(&usage.target_kind) {
        Ok(SummaryTargetKind::MessageVersion) => usage
            .target_message_version_id
            .as_deref()
            .is_some_and(|id| id == message.version_id),
        Ok(SummaryTargetKind::NodeRange) => {
            let Some(start_id) = usage.target_start_node_id.as_ref() else {
                return false;
            };
            let Some(end_id) = usage.target_end_node_id.as_ref() else {
                return false;
            };
            let Some(&current) = order_index.get(&message.node_id) else {
                return false;
            };
            let Some(&start) = order_index.get(start_id) else {
                return false;
            };
            let Some(&end) = order_index.get(end_id) else {
                return false;
            };
            let (lower, upper) = if start <= end {
                (start, end)
            } else {
                (end, start)
            };
            current >= lower && current <= upper
        }
        Ok(SummaryTargetKind::Conversation) => usage
            .conversation_id
            .as_deref()
            .is_some_and(|id| id == message.conversation_id),
        Err(_) => false,
    }
}

fn usage_activation_passes(
    usage: &SummaryUsageRow,
    message: &MessageVersionView,
    total_messages: i64,
    total_bytes: i64,
    order_index: &HashMap<String, usize>,
) -> bool {
    match SummaryActivationMode::parse(&usage.activation_mode) {
        Ok(SummaryActivationMode::Manual) => true,
        Ok(SummaryActivationMode::Auto) => {
            if let Some(replace_from_node_id) = &usage.replace_from_node_id {
                let Some(&message_index) = order_index.get(&message.node_id) else {
                    return false;
                };
                let Some(&replace_index) = order_index.get(replace_from_node_id) else {
                    return false;
                };
                if message_index < replace_index {
                    return false;
                }
            }

            if let Some(min_count) = usage.replace_after_message_count {
                if total_messages < min_count {
                    return false;
                }
            }

            if let Some(min_bytes) = usage.replace_after_total_bytes {
                if total_bytes < min_bytes {
                    return false;
                }
            }

            true
        }
        Err(_) => false,
    }
}

async fn load_summary_version_cached(
    db: &SqlitePool,
    cache: &mut HashMap<String, SummaryVersionRow>,
    summary_version_id: &str,
) -> Result<Option<SummaryVersionRow>> {
    if let Some(cached) = cache.get(summary_version_id) {
        return Ok(Some(cached.clone()));
    }
    let row = repo::get_summary_version(db, summary_version_id).await?;
    cache.insert(summary_version_id.to_string(), row.clone());
    Ok(Some(row))
}

async fn load_active_summary_version_cached(
    db: &SqlitePool,
    cache: &mut HashMap<String, SummaryVersionRow>,
    summary_group_id: &str,
) -> Result<Option<SummaryVersionRow>> {
    if let Some(cached) = cache
        .values()
        .find(|item| item.summary_group_id == summary_group_id)
    {
        return Ok(Some(cached.clone()));
    }

    let row = repo::get_active_summary_version(db, summary_group_id).await?;
    if let Some(row) = row {
        cache.insert(row.id.clone(), row.clone());
        Ok(Some(row))
    } else {
        Ok(None)
    }
}

async fn select_summary_participant(
    db: &SqlitePool,
    conversation_id: &str,
) -> Result<crate::db::models::ConversationParticipantRow> {
    let participants =
        conversation_repo::list_conversation_participants(db, conversation_id).await?;
    participants
        .iter()
        .find(|item| item.enabled && item.agent_id.is_some())
        .cloned()
        .or_else(|| participants.into_iter().find(|item| item.enabled))
        .ok_or_else(|| {
            AppError::Validation(
                "conversation must have at least one enabled participant to generate a summary"
                    .to_string(),
            )
        })
}

async fn upsert_summary_group_for_scope(
    db: &SqlitePool,
    conversation_id: &str,
    scope: &ScopeSpec,
    generator_preset_id: Option<&str>,
) -> Result<SummaryGroupRow> {
    let (
        scope_type,
        scope_message_version_id,
        scope_start_node_id,
        scope_end_node_id,
        scope_summary_group_id,
    ) = scope.to_parts();
    if let Some(existing) = repo::find_summary_group_by_scope(
        db,
        conversation_id,
        scope_type,
        scope_message_version_id,
        scope_start_node_id,
        scope_end_node_id,
        scope_summary_group_id,
        SummaryKind::Brief.as_str(),
    )
    .await?
    {
        if generator_preset_id.is_some()
            && existing.default_generator_preset_id.as_deref() != generator_preset_id
        {
            return repo::update_summary_group(
                db,
                &existing.id,
                &repo::UpdateSummaryGroupRecord {
                    default_generator_preset_id: generator_preset_id,
                    enabled: true,
                },
            )
            .await;
        }
        return Ok(existing);
    }

    repo::create_summary_group(
        db,
        &repo::CreateSummaryGroupRecord {
            conversation_id,
            scope_type,
            scope_message_version_id,
            scope_start_node_id,
            scope_end_node_id,
            scope_summary_group_id,
            summary_kind: SummaryKind::Brief.as_str(),
            default_generator_preset_id: generator_preset_id,
            enabled: true,
        },
    )
    .await
}

async fn validate_summary_usage_input(
    db: &SqlitePool,
    input: &UpsertSummaryUsageInput,
) -> Result<()> {
    let group = repo::get_summary_group(db, &input.summary_group_id).await?;

    if let Some(summary_version_id) = &input.summary_version_id {
        let version = repo::get_summary_version(db, summary_version_id).await?;
        if version.summary_group_id != input.summary_group_id {
            return Err(AppError::Validation(
                "summary_version_id does not belong to summary_group_id".to_string(),
            ));
        }
    }

    let derived_conversation_id = match input.target_kind {
        SummaryTargetKind::MessageVersion => {
            let message_version_id =
                input.target_message_version_id.as_deref().ok_or_else(|| {
                    AppError::Validation(
                        "target_message_version_id is required for target_kind=message_version"
                            .to_string(),
                    )
                })?;
            let version = message_repo::get_message_version(db, message_version_id).await?;
            let node = message_repo::get_message_node(db, &version.node_id).await?;
            Some(node.conversation_id)
        }
        SummaryTargetKind::NodeRange => {
            let start_node_id = input.target_start_node_id.as_deref().ok_or_else(|| {
                AppError::Validation(
                    "target_start_node_id is required for target_kind=node_range".to_string(),
                )
            })?;
            let end_node_id = input.target_end_node_id.as_deref().ok_or_else(|| {
                AppError::Validation(
                    "target_end_node_id is required for target_kind=node_range".to_string(),
                )
            })?;
            let start = message_repo::get_message_node(db, start_node_id).await?;
            let end = message_repo::get_message_node(db, end_node_id).await?;
            if start.conversation_id != end.conversation_id {
                return Err(AppError::Validation(
                    "target_start_node_id and target_end_node_id must belong to the same conversation"
                        .to_string(),
                ));
            }
            Some(start.conversation_id)
        }
        SummaryTargetKind::Conversation => {
            let conversation_id = input.conversation_id.as_deref().ok_or_else(|| {
                AppError::Validation(
                    "conversation_id is required for target_kind=conversation".to_string(),
                )
            })?;
            let _ = conversation_repo::get_conversation(db, conversation_id).await?;
            Some(conversation_id.to_string())
        }
    };

    if let Some(conversation_id) = &input.conversation_id {
        if let Some(derived) = &derived_conversation_id {
            if conversation_id != derived {
                return Err(AppError::Validation(
                    "conversation_id does not match summary usage target conversation".to_string(),
                ));
            }
        }
    }

    if group.conversation_id
        != derived_conversation_id
            .clone()
            .unwrap_or_else(|| group.conversation_id.clone())
    {
        return Err(AppError::Validation(
            "summary_group_id does not belong to the target conversation".to_string(),
        ));
    }

    if let Some(replace_from_node_id) = &input.replace_from_node_id {
        let node = message_repo::get_message_node(db, replace_from_node_id).await?;
        if node.conversation_id != group.conversation_id {
            return Err(AppError::Validation(
                "replace_from_node_id does not belong to the summary conversation".to_string(),
            ));
        }
    }

    Ok(())
}

fn extract_summary_content_input(response: &ProviderChatResponse) -> Result<ContentWriteInput> {
    let part = response
        .parts
        .iter()
        .find(|item| {
            item.text
                .as_ref()
                .is_some_and(|text| !text.trim().is_empty())
                || item
                    .content
                    .as_ref()
                    .and_then(|content| content.text_content.as_ref())
                    .is_some_and(|text| !text.trim().is_empty())
        })
        .ok_or_else(|| {
            AppError::Validation(
                "provider response contained no usable summary content".to_string(),
            )
        })?;

    if let Some(content) = &part.content {
        return Ok(ContentWriteInput {
            content_type: ContentType::Text,
            mime_type: Some("text/plain".to_string()),
            text_content: content
                .text_content
                .clone()
                .or_else(|| content.preview_text.clone()),
            source_file_path: None,
            primary_storage_uri: None,
            size_bytes_hint: Some(content.size_bytes),
            preview_text: content.preview_text.clone(),
            config_json: part.metadata_json.clone(),
        });
    }

    Ok(ContentWriteInput {
        content_type: ContentType::Text,
        mime_type: Some("text/plain".to_string()),
        text_content: part.text.clone(),
        source_file_path: None,
        primary_storage_uri: None,
        size_bytes_hint: None,
        preview_text: None,
        config_json: part.metadata_json.clone(),
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

async fn create_inline_instruction_content(
    db: &SqlitePool,
    store: &ContentStore,
    text: &str,
) -> Result<StoredContent> {
    content::create_content(
        db,
        store,
        &ContentWriteInput {
            content_type: ContentType::Text,
            mime_type: Some("text/plain".to_string()),
            text_content: Some(text.to_string()),
            source_file_path: None,
            primary_storage_uri: None,
            size_bytes_hint: Some(text.len() as u64),
            preview_text: Some(text.chars().take(1024).collect()),
            config_json: json!({}),
        },
    )
    .await
}

fn parse_json(raw: &str, field: &'static str) -> Result<serde_json::Value> {
    serde_json::from_str(raw)
        .map_err(|err| AppError::Validation(format!("failed to parse {field} as json: {err}")))
}

enum ScopeSpec {
    Node {
        node_id: String,
    },
    NodeRange {
        start_node_id: String,
        end_node_id: String,
    },
    Conversation,
}

impl ScopeSpec {
    fn to_parts(&self) -> (&str, Option<&str>, Option<&str>, Option<&str>, Option<&str>) {
        match self {
            Self::Node { node_id } => ("node", None, Some(node_id), Some(node_id), None),
            Self::NodeRange {
                start_node_id,
                end_node_id,
            } => (
                "node_range",
                None,
                Some(start_node_id),
                Some(end_node_id),
                None,
            ),
            Self::Conversation => ("conversation", None, None, None, None),
        }
    }
}
