use sqlx::SqlitePool;

use crate::db::repos::{agents as agent_repo, conversations as conversation_repo};
use crate::domain::agents::AgentDetail;
use crate::domain::api_channels::{ApiChannel, ApiChannelModel};
use crate::domain::content::{ContentType, StoredContent};
use crate::domain::lorebooks::{LorebookDetail, LorebookMatchInput};
use crate::domain::messages::{
    BuildGenerationContextInput, BuiltGenerationContext, GenerationContextItem, HistoryMaterial,
    MessageRole, MessageVersionView,
};
use crate::domain::presets::PresetDetail;
use crate::domain::user_profiles::UserProfileDetail;
use crate::services::content_store::ContentStore;
use crate::services::{
    agents, api_channels, content, lorebooks, messages, presets, summaries, user_profiles,
};
use crate::support::error::{AppError, Result};

pub async fn build_generation_context(
    db: &SqlitePool,
    store: &ContentStore,
    input: &BuildGenerationContextInput,
) -> Result<BuiltGenerationContext> {
    let participant =
        conversation_repo::get_conversation_participant(db, &input.responder_participant_id)
            .await?;
    if participant.conversation_id != input.conversation_id {
        return Err(AppError::Validation(
            "responder_participant_id does not belong to conversation_id".to_string(),
        ));
    }

    let trigger_message = match &input.trigger_message_version_id {
        Some(version_id) => Some(messages::get_message_version_view(db, store, version_id).await?),
        None => None,
    };

    let (preset, preset_source_scope) = resolve_active_preset_with_scope(
        db,
        store,
        &input.conversation_id,
        &input.responder_participant_id,
    )
    .await?;
    let (lorebook, lorebook_source_scope) = resolve_active_lorebook_with_scope(
        db,
        store,
        &input.conversation_id,
        &input.responder_participant_id,
    )
    .await?;
    let (user_profile, user_profile_source_scope) = resolve_active_user_profile_with_scope(
        db,
        store,
        &input.conversation_id,
        &input.responder_participant_id,
    )
    .await?;

    let responder_agent = match participant.agent_id.as_deref() {
        Some(agent_id) => Some(agents::get_agent_detail(db, store, agent_id).await?),
        None => None,
    };

    let (api_channel, api_channel_model, api_channel_source_scope, api_channel_model_source_scope) =
        resolve_active_channel_model(
            db,
            &input.conversation_id,
            &input.responder_participant_id,
            input.override_api_channel_id.as_deref(),
            input.override_api_channel_model_id.as_deref(),
        )
        .await?;

    let request_parameters_json = merge_request_parameters(
        &api_channel_model.default_parameters_json,
        input.request_parameters_json.as_ref(),
    );

    let materials = select_history_materials(
        db,
        store,
        input,
        preset.as_ref(),
        lorebook.as_ref(),
        user_profile.as_ref(),
        responder_agent.as_ref(),
    )
    .await?;

    Ok(BuiltGenerationContext {
        conversation_id: input.conversation_id.clone(),
        responder_participant_id: input.responder_participant_id.clone(),
        trigger_node_id: trigger_message.as_ref().map(|item| item.node_id.clone()),
        trigger_message_version_id: input.trigger_message_version_id.clone(),
        api_channel,
        api_channel_model,
        request_parameters_json,
        preset_id: preset.as_ref().map(|item| item.preset.id.clone()),
        preset_source_scope,
        lorebook_id: lorebook.as_ref().map(|item| item.lorebook.id.clone()),
        lorebook_source_scope,
        user_profile_id: user_profile.as_ref().map(|item| item.summary.id.clone()),
        user_profile_source_scope,
        api_channel_source_scope: Some(api_channel_source_scope),
        api_channel_model_source_scope: Some(api_channel_model_source_scope),
        items: materialize_context_items(&materials)?,
    })
}

pub async fn resolve_active_preset(
    db: &SqlitePool,
    store: &ContentStore,
    conversation_id: &str,
    responder_participant_id: &str,
) -> Result<Option<PresetDetail>> {
    Ok(
        resolve_active_preset_with_scope(db, store, conversation_id, responder_participant_id)
            .await?
            .0,
    )
}

pub async fn resolve_active_lorebook(
    db: &SqlitePool,
    store: &ContentStore,
    conversation_id: &str,
    responder_participant_id: &str,
) -> Result<Option<LorebookDetail>> {
    Ok(
        resolve_active_lorebook_with_scope(db, store, conversation_id, responder_participant_id)
            .await?
            .0,
    )
}

pub async fn resolve_active_user_profile(
    db: &SqlitePool,
    store: &ContentStore,
    conversation_id: &str,
    responder_participant_id: &str,
) -> Result<Option<UserProfileDetail>> {
    Ok(
        resolve_active_user_profile_with_scope(
            db,
            store,
            conversation_id,
            responder_participant_id,
        )
        .await?
        .0,
    )
}

pub async fn resolve_active_channel_model(
    db: &SqlitePool,
    conversation_id: &str,
    responder_participant_id: &str,
    override_api_channel_id: Option<&str>,
    override_api_channel_model_id: Option<&str>,
) -> Result<(ApiChannel, ApiChannelModel, String, String)> {
    if let Some(channel_model_id) = override_api_channel_model_id {
        let model_row =
            crate::db::repos::api_channels::get_channel_model_by_id(db, channel_model_id).await?;
        let channel_id = override_api_channel_id.unwrap_or(&model_row.channel_id);
        let channel = api_channels::get_channel(db, channel_id).await?;
        if channel.id != model_row.channel_id {
            return Err(AppError::Validation(
                "override_api_channel_model_id does not belong to override_api_channel_id"
                    .to_string(),
            ));
        }
        return Ok((
            channel,
            api_channels::map_channel_model_row(model_row)?,
            "runtime".to_string(),
            "runtime".to_string(),
        ));
    }

    if let Some(channel_id) = override_api_channel_id {
        return Ok((
            api_channels::get_channel(db, channel_id).await?,
            first_model_for_channel(db, channel_id).await?,
            "runtime".to_string(),
            "runtime".to_string(),
        ));
    }

    let conversation_bindings =
        conversation_repo::list_conversation_channel_bindings(db, conversation_id).await?;
    if let Some(binding) = conversation_bindings
        .into_iter()
        .find(|item| item.enabled && item.binding_type == "active")
    {
        let model = match binding.channel_model_id {
            Some(model_id) => api_channels::map_channel_model_row(
                crate::db::repos::api_channels::get_channel_model_by_id(db, &model_id).await?,
            )?,
            None => first_model_for_channel(db, &binding.channel_id).await?,
        };
        return Ok((
            api_channels::get_channel(db, &binding.channel_id).await?,
            model,
            "conversation".to_string(),
            "conversation".to_string(),
        ));
    }

    let participant =
        conversation_repo::get_conversation_participant(db, responder_participant_id).await?;
    if let Some(agent_id) = participant.agent_id.as_deref() {
        let bindings = agent_repo::list_agent_channel_bindings(db, agent_id).await?;
        if let Some(binding) = bindings
            .into_iter()
            .find(|item| item.enabled && item.binding_type == "default")
        {
            let model = match binding.channel_model_id {
                Some(model_id) => api_channels::map_channel_model_row(
                    crate::db::repos::api_channels::get_channel_model_by_id(db, &model_id).await?,
                )?,
                None => first_model_for_channel(db, &binding.channel_id).await?,
            };
            return Ok((
                api_channels::get_channel(db, &binding.channel_id).await?,
                model,
                "agent".to_string(),
                "agent".to_string(),
            ));
        }
    }

    Err(AppError::Validation(
        "no active api channel/model could be resolved".to_string(),
    ))
}

pub async fn select_history_materials(
    db: &SqlitePool,
    store: &ContentStore,
    input: &BuildGenerationContextInput,
    preset: Option<&PresetDetail>,
    lorebook: Option<&LorebookDetail>,
    user_profile: Option<&UserProfileDetail>,
    agent: Option<&AgentDetail>,
) -> Result<Vec<HistoryMaterial>> {
    let visible_messages =
        messages::list_visible_messages(db, store, &input.conversation_id).await?;
    let context_summaries =
        summaries::resolve_context_summaries_for_messages(db, store, &visible_messages).await?;
    let recent_messages = visible_messages
        .iter()
        .filter_map(extract_match_text)
        .collect::<Vec<_>>();

    let matched_lorebook_entries = if let Some(lorebook) = lorebook {
        lorebooks::match_entries(
            db,
            store,
            &LorebookMatchInput {
                conversation_id: Some(input.conversation_id.clone()),
                lorebook_id: lorebook.lorebook.id.clone(),
                recent_messages,
                max_entries: lorebook.lorebook.scan_depth.max(1) as usize,
                include_disabled: false,
            },
        )
        .await?
    } else {
        Vec::new()
    };

    let mut materials = Vec::new();
    let mut sequence_no = 0_i64;

    if let Some(agent) = agent {
        push_agent_materials(&mut materials, &mut sequence_no, agent);
    }

    if let Some(user_profile) = user_profile {
        materials.push(HistoryMaterial {
            sequence_no,
            source_kind: "user_profile".to_string(),
            message_version: None,
            summary_version_id: None,
            summary_content: None,
            preset_entry: None,
            lorebook_entry: None,
            user_profile: Some(user_profile.clone()),
            agent: None,
            preset: None,
            lorebook: None,
            plugin_content: None,
            config_json: serde_json::json!({}),
        });
        sequence_no += 1;
    }

    if let Some(preset) = preset {
        for entry in &preset.entries {
            if entry.enabled {
                materials.push(HistoryMaterial {
                    sequence_no,
                    source_kind: "preset_entry".to_string(),
                    message_version: None,
                    summary_version_id: None,
                    summary_content: None,
                    preset_entry: Some(entry.clone()),
                    lorebook_entry: None,
                    user_profile: None,
                    agent: None,
                    preset: Some(preset.clone()),
                    lorebook: None,
                    plugin_content: None,
                    config_json: serde_json::json!({}),
                });
                sequence_no += 1;
            }
        }
    }

    for entry in matched_lorebook_entries {
        materials.push(HistoryMaterial {
            sequence_no,
            source_kind: "lorebook_entry".to_string(),
            message_version: None,
            summary_version_id: None,
            summary_content: None,
            preset_entry: None,
            lorebook_entry: Some(entry),
            user_profile: None,
            agent: None,
            preset: None,
            lorebook: lorebook.cloned(),
            plugin_content: None,
            config_json: serde_json::json!({}),
        });
        sequence_no += 1;
    }

    for message in visible_messages {
        if matches!(
            message.context_policy,
            crate::domain::messages::ContextPolicy::Exclude
        ) {
            continue;
        }

        let summary = context_summaries.get(&message.version_id).cloned();
        materials.push(HistoryMaterial {
            sequence_no,
            source_kind: "message_version".to_string(),
            message_version: Some(message.clone()),
            summary_version_id: summary.as_ref().map(|item| item.summary_version_id.clone()),
            summary_content: summary.as_ref().map(|item| item.content.clone()),
            preset_entry: None,
            lorebook_entry: None,
            user_profile: None,
            agent: None,
            preset: None,
            lorebook: None,
            plugin_content: None,
            config_json: serde_json::json!({}),
        });
        sequence_no += 1;

        push_message_ref_materials(db, store, &mut materials, &mut sequence_no, &message).await?;
    }

    Ok(materials)
}

pub fn materialize_context_items(
    materials: &[HistoryMaterial],
) -> Result<Vec<GenerationContextItem>> {
    let mut items = Vec::new();
    for material in materials {
        if let Some(message_version) = &material.message_version {
            let rendered_content = material
                .summary_content
                .clone()
                .unwrap_or_else(|| message_version.primary_content.clone());
            items.push(build_item(
                material,
                message_version.role,
                rendered_content,
                Some(message_version.node_id.clone()),
                Some(message_version.version_id.clone()),
                material.summary_version_id.clone(),
                None,
                None,
                None,
                None,
            ));
            continue;
        }

        if let Some(plugin_content) = &material.plugin_content {
            let metadata = build_content_ref_metadata(plugin_content);
            let mut item = build_item(
                material,
                MessageRole::System,
                plugin_content.content.clone(),
                metadata.source_message_node_id.clone(),
                metadata.source_message_version_id,
                None,
                None,
                None,
                None,
                None,
            );
            item.source_tool_invocation_id = metadata.source_tool_invocation_id;
            item.source_rag_ref_id = metadata.source_rag_ref_id;
            item.source_mcp_event_id = metadata.source_mcp_event_id;
            item.source_plugin_id = metadata.source_plugin_id;
            item.config_json = metadata.config_json;
            item.source_kind = metadata.source_kind;
            item.send_role = metadata.send_role;
            items.push(item);
            continue;
        }

        if let Some(entry) = &material.preset_entry {
            items.push(build_item(
                material,
                entry.role,
                entry.primary_content.clone(),
                None,
                None,
                None,
                Some(entry.id.clone()),
                None,
                None,
                None,
            ));
            continue;
        }

        if let Some(entry) = &material.lorebook_entry {
            items.push(build_item(
                material,
                MessageRole::System,
                entry.content.clone(),
                None,
                None,
                None,
                None,
                Some(entry.lorebook_entry_id.clone()),
                None,
                None,
            ));
            continue;
        }

        if let Some(profile) = &material.user_profile {
            if let Some(description_content) = &profile.description_content {
                items.push(build_item(
                    material,
                    profile.injection_role.unwrap_or(MessageRole::System),
                    description_content.clone(),
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(profile.summary.id.clone()),
                    None,
                ));
            }
            continue;
        }

        if let Some(agent) = &material.agent {
            if let Some(content) = resolve_agent_slot_content(agent, &material.config_json) {
                let role = match material
                    .config_json
                    .get("agent_slot")
                    .and_then(serde_json::Value::as_str)
                {
                    Some("character_note") => {
                        agent.character_note_role.unwrap_or(MessageRole::System)
                    }
                    _ => MessageRole::System,
                };
                items.push(build_item(
                    material,
                    role,
                    content,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(agent.summary.id.clone()),
                ));
            }
        }
    }

    Ok(items)
}

async fn resolve_active_preset_with_scope(
    db: &SqlitePool,
    store: &ContentStore,
    conversation_id: &str,
    responder_participant_id: &str,
) -> Result<(Option<PresetDetail>, Option<String>)> {
    let bindings =
        conversation_repo::list_conversation_preset_bindings(db, conversation_id).await?;
    if let Some(binding) = bindings
        .into_iter()
        .find(|item| item.enabled && item.binding_type == "active")
    {
        return Ok((
            Some(presets::get_preset_detail(db, store, &binding.resource_id).await?),
            Some("conversation".to_string()),
        ));
    }

    let participant =
        conversation_repo::get_conversation_participant(db, responder_participant_id).await?;
    if let Some(agent_id) = participant.agent_id.as_deref() {
        let bindings = agent_repo::list_agent_preset_bindings(db, agent_id).await?;
        if let Some(binding) = bindings
            .into_iter()
            .find(|item| item.enabled && item.binding_type == "default")
        {
            return Ok((
                Some(presets::get_preset_detail(db, store, &binding.resource_id).await?),
                Some("agent".to_string()),
            ));
        }
    }

    Ok((None, None))
}

async fn resolve_active_lorebook_with_scope(
    db: &SqlitePool,
    store: &ContentStore,
    conversation_id: &str,
    responder_participant_id: &str,
) -> Result<(Option<LorebookDetail>, Option<String>)> {
    let bindings =
        conversation_repo::list_conversation_lorebook_bindings(db, conversation_id).await?;
    if let Some(binding) = bindings
        .into_iter()
        .find(|item| item.enabled && item.binding_type == "active")
    {
        return Ok((
            Some(lorebooks::get_lorebook_detail(db, store, &binding.resource_id).await?),
            Some("conversation".to_string()),
        ));
    }

    let participant =
        conversation_repo::get_conversation_participant(db, responder_participant_id).await?;
    if let Some(agent_id) = participant.agent_id.as_deref() {
        let bindings = agent_repo::list_agent_lorebook_bindings(db, agent_id).await?;
        if let Some(binding) = bindings
            .into_iter()
            .find(|item| item.enabled && item.binding_type == "default")
        {
            return Ok((
                Some(lorebooks::get_lorebook_detail(db, store, &binding.resource_id).await?),
                Some("agent".to_string()),
            ));
        }
    }

    Ok((None, None))
}

async fn resolve_active_user_profile_with_scope(
    db: &SqlitePool,
    store: &ContentStore,
    conversation_id: &str,
    responder_participant_id: &str,
) -> Result<(Option<UserProfileDetail>, Option<String>)> {
    let bindings =
        conversation_repo::list_conversation_user_profile_bindings(db, conversation_id).await?;
    if let Some(binding) = bindings
        .into_iter()
        .find(|item| item.enabled && item.binding_type == "active")
    {
        return Ok((
            Some(user_profiles::get_user_profile(db, store, &binding.resource_id).await?),
            Some("conversation".to_string()),
        ));
    }

    let participant =
        conversation_repo::get_conversation_participant(db, responder_participant_id).await?;
    if let Some(agent_id) = participant.agent_id.as_deref() {
        let bindings = agent_repo::list_agent_user_profile_bindings(db, agent_id).await?;
        if let Some(binding) = bindings
            .into_iter()
            .find(|item| item.enabled && item.binding_type == "default")
        {
            return Ok((
                Some(user_profiles::get_user_profile(db, store, &binding.resource_id).await?),
                Some("agent".to_string()),
            ));
        }
    }

    Ok((None, None))
}

async fn first_model_for_channel(db: &SqlitePool, channel_id: &str) -> Result<ApiChannelModel> {
    api_channels::list_channel_models(db, channel_id)
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| AppError::Validation(format!("api channel '{channel_id}' has no models")))
}

fn merge_request_parameters(
    defaults: &serde_json::Value,
    overrides: Option<&serde_json::Value>,
) -> serde_json::Value {
    let mut merged = defaults.as_object().cloned().unwrap_or_default();
    if let Some(overrides) = overrides.and_then(serde_json::Value::as_object) {
        for (key, value) in overrides {
            merged.insert(key.clone(), value.clone());
        }
    }
    serde_json::Value::Object(merged)
}

fn push_agent_materials(
    materials: &mut Vec<HistoryMaterial>,
    sequence_no: &mut i64,
    agent: &AgentDetail,
) {
    for slot in [
        "main_prompt_override",
        "description",
        "personality",
        "scenario",
        "character_note",
        "post_history_instructions",
    ] {
        if resolve_agent_slot_content(agent, &serde_json::json!({ "agent_slot": slot })).is_some() {
            materials.push(HistoryMaterial {
                sequence_no: *sequence_no,
                source_kind: "agent".to_string(),
                message_version: None,
                summary_version_id: None,
                summary_content: None,
                preset_entry: None,
                lorebook_entry: None,
                user_profile: None,
                agent: Some(agent.clone()),
                preset: None,
                lorebook: None,
                plugin_content: None,
                config_json: serde_json::json!({ "agent_slot": slot }),
            });
            *sequence_no += 1;
        }
    }
}

fn resolve_agent_slot_content(
    agent: &AgentDetail,
    config_json: &serde_json::Value,
) -> Option<StoredContent> {
    match config_json
        .get("agent_slot")
        .and_then(serde_json::Value::as_str)
    {
        Some("main_prompt_override") => agent.main_prompt_override_content.clone(),
        Some("description") => agent.description_content.clone(),
        Some("personality") => agent.personality_content.clone(),
        Some("scenario") => agent.scenario_content.clone(),
        Some("character_note") => agent.character_note_content.clone(),
        Some("post_history_instructions") => agent.post_history_instructions_content.clone(),
        _ => None,
    }
}

fn build_item(
    material: &HistoryMaterial,
    send_role: MessageRole,
    rendered_content: StoredContent,
    source_message_node_id: Option<String>,
    source_message_version_id: Option<String>,
    source_summary_version_id: Option<String>,
    source_preset_entry_id: Option<String>,
    source_lorebook_entry_id: Option<String>,
    source_user_profile_id: Option<String>,
    source_agent_id: Option<String>,
) -> GenerationContextItem {
    GenerationContextItem {
        sequence_no: material.sequence_no,
        send_role,
        rendered_content,
        source_kind: material.source_kind.clone(),
        source_message_node_id,
        source_message_version_id,
        source_summary_version_id,
        source_preset_entry_id,
        source_lorebook_entry_id,
        source_user_profile_id,
        source_agent_id,
        source_agent_greeting_id: None,
        source_tool_invocation_id: None,
        source_rag_ref_id: None,
        source_mcp_event_id: None,
        source_plugin_id: None,
        included_in_request: true,
        config_json: material.config_json.clone(),
    }
}

async fn push_message_ref_materials(
    db: &SqlitePool,
    store: &ContentStore,
    materials: &mut Vec<HistoryMaterial>,
    sequence_no: &mut i64,
    message: &MessageVersionView,
) -> Result<()> {
    for item in &message.content_refs {
        if !should_include_message_ref_in_context(item) {
            continue;
        }

        let mut item = item.clone();
        item.content = content::get_content(db, store, &item.content.content_id, true).await?;

        materials.push(HistoryMaterial {
            sequence_no: *sequence_no,
            source_kind: "message_ref".to_string(),
            message_version: None,
            summary_version_id: None,
            summary_content: None,
            preset_entry: None,
            lorebook_entry: None,
            user_profile: None,
            agent: None,
            preset: None,
            lorebook: None,
            plugin_content: Some(item.clone()),
            config_json: serde_json::json!({
                "message_version_id": message.version_id,
                "message_node_id": message.node_id,
                "message_role": message.role.as_str(),
                "ref_role": item.ref_role,
            }),
        });
        *sequence_no += 1;
    }

    Ok(())
}

fn should_include_message_ref_in_context(
    item: &crate::domain::messages::MessageContentRefView,
) -> bool {
    matches!(
        item.content.content_type,
        ContentType::Image
            | ContentType::Audio
            | ContentType::Video
            | ContentType::File
            | ContentType::Binary
            | ContentType::ToolRequest
            | ContentType::ToolResponse
            | ContentType::RagExcerpt
            | ContentType::McpPayload
            | ContentType::PluginPayload
            | ContentType::PluginState
            | ContentType::ReasoningTrace
            | ContentType::ProviderSignature
    ) || matches!(
        item.ref_role.as_str(),
        "attachment"
            | "image"
            | "audio"
            | "video"
            | "file"
            | "binary"
            | "tool_request"
            | "tool_response"
            | "rag_excerpt"
            | "mcp_payload"
            | "plugin_payload"
            | "plugin_state"
            | "reasoning_trace"
            | "provider_signature"
    )
}

struct MessageRefMetadata {
    source_kind: String,
    send_role: MessageRole,
    source_message_node_id: Option<String>,
    source_message_version_id: Option<String>,
    source_tool_invocation_id: Option<String>,
    source_rag_ref_id: Option<String>,
    source_mcp_event_id: Option<String>,
    source_plugin_id: Option<String>,
    config_json: serde_json::Value,
}

fn build_content_ref_metadata(
    item: &crate::domain::messages::MessageContentRefView,
) -> MessageRefMetadata {
    let message_role = item
        .config_json
        .get("message_role")
        .and_then(serde_json::Value::as_str)
        .and_then(|value| MessageRole::parse(value).ok())
        .unwrap_or(MessageRole::System);
    let source_kind = match item.ref_role.as_str() {
        "tool_request" | "tool_response" => "tool_invocation",
        "rag_excerpt" => "rag_ref",
        "mcp_payload" => "mcp_event",
        "plugin_payload" | "plugin_state" => "plugin_content",
        "reasoning_trace" => "reasoning_trace",
        "provider_signature" => "provider_signature",
        _ => match item.content.content_type {
            ContentType::ToolRequest | ContentType::ToolResponse => "tool_invocation",
            ContentType::RagExcerpt => "rag_ref",
            ContentType::McpPayload => "mcp_event",
            ContentType::PluginPayload | ContentType::PluginState => "plugin_content",
            ContentType::ReasoningTrace => "reasoning_trace",
            ContentType::ProviderSignature => "provider_signature",
            _ => "message_ref",
        },
    }
    .to_string();

    MessageRefMetadata {
        source_kind,
        send_role: message_role,
        source_message_node_id: item
            .config_json
            .get("message_node_id")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string),
        source_message_version_id: item
            .config_json
            .get("message_version_id")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string),
        source_tool_invocation_id: item
            .config_json
            .get("tool_invocation_id")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string),
        source_rag_ref_id: item
            .config_json
            .get("rag_ref_id")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string),
        source_mcp_event_id: item
            .config_json
            .get("mcp_event_id")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string),
        source_plugin_id: item.plugin_id.clone().or_else(|| {
            item.config_json
                .get("plugin_id")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
        }),
        config_json: item.config_json.clone(),
    }
}

fn extract_match_text(message: &MessageVersionView) -> Option<String> {
    message
        .primary_content
        .text_content
        .clone()
        .or_else(|| message.primary_content.preview_text.clone())
}
