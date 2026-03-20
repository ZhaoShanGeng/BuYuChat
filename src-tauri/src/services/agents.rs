use sqlx::SqlitePool;

use crate::db::models::{
    AgentChannelBindingRow, AgentGreetingRow, AgentMediaRow, AgentResourceBindingRow, AgentRow,
};
use crate::db::repos::{
    agents as repo, api_channels as channel_repo, lorebooks, presets, user_profiles,
};
use crate::domain::agents::{
    AddAgentMediaInput, AgentChannelBindingInput, AgentDetail, AgentGreetingDetail,
    AgentMediaDetail, AgentResourceBindingInput, AgentSummary, CreateAgentGreetingInput,
    CreateAgentInput, UpdateAgentGreetingInput, UpdateAgentInput,
};
use crate::domain::common::{ChannelBindingDetail, ResourceBindingDetail};
use crate::domain::content::{ContentType, ContentWriteInput, StoredContent};
use crate::domain::messages::MessageRole;
use crate::services::content as content_service;
use crate::services::content_store::ContentStore;
use crate::support::error::{AppError, Result};

pub async fn list_agents(db: &SqlitePool) -> Result<Vec<AgentSummary>> {
    repo::list_agents(db)
        .await?
        .into_iter()
        .map(map_agent_summary)
        .collect()
}

pub async fn get_agent_detail(
    db: &SqlitePool,
    store: &ContentStore,
    id: &str,
) -> Result<AgentDetail> {
    let agent = repo::get_agent(db, id).await?;
    build_agent_detail(db, store, agent).await
}

pub async fn create_agent(
    db: &SqlitePool,
    store: &ContentStore,
    input: &CreateAgentInput,
) -> Result<AgentDetail> {
    let prepared = prepare_agent_values_from_create(db, store, input).await?;
    let row = repo::create_agent(db, &prepared.as_repo_input_from_create(input)).await?;

    build_agent_detail(db, store, row).await
}

pub async fn update_agent(
    db: &SqlitePool,
    store: &ContentStore,
    id: &str,
    input: &UpdateAgentInput,
) -> Result<AgentDetail> {
    let prepared = prepare_agent_values_from_update(db, store, input).await?;
    let row = repo::update_agent(db, id, &prepared.as_repo_input_from_update(input)).await?;

    build_agent_detail(db, store, row).await
}

pub async fn delete_agent(db: &SqlitePool, id: &str) -> Result<()> {
    repo::delete_agent(db, id).await
}

pub async fn create_greeting(
    db: &SqlitePool,
    store: &ContentStore,
    agent_id: &str,
    input: &CreateAgentGreetingInput,
) -> Result<AgentGreetingDetail> {
    let _ = repo::get_agent(db, agent_id).await?;
    ensure_textual_content(&input.primary_content.content_type, "agent greeting")?;
    let content = content_service::create_content(db, store, &input.primary_content).await?;
    let config_json = input.config_json.to_string();
    let row = repo::create_agent_greeting(
        db,
        &repo::CreateOrUpdateAgentGreeting {
            agent_id,
            greeting_type: &input.greeting_type,
            name: input.name.as_deref(),
            primary_content_id: &content.content_id,
            enabled: input.enabled,
            sort_order: input.sort_order,
            config_json: &config_json,
        },
    )
    .await?;

    map_agent_greeting_detail(db, store, row).await
}

pub async fn update_greeting(
    db: &SqlitePool,
    store: &ContentStore,
    greeting_id: &str,
    input: &UpdateAgentGreetingInput,
) -> Result<AgentGreetingDetail> {
    let existing = repo::get_agent_greeting(db, greeting_id).await?;
    ensure_textual_content(&input.primary_content.content_type, "agent greeting")?;
    let content = content_service::create_content(db, store, &input.primary_content).await?;
    let config_json = input.config_json.to_string();
    let row = repo::update_agent_greeting(
        db,
        greeting_id,
        &repo::CreateOrUpdateAgentGreeting {
            agent_id: &existing.agent_id,
            greeting_type: &input.greeting_type,
            name: input.name.as_deref(),
            primary_content_id: &content.content_id,
            enabled: input.enabled,
            sort_order: input.sort_order,
            config_json: &config_json,
        },
    )
    .await?;

    map_agent_greeting_detail(db, store, row).await
}

pub async fn delete_greeting(db: &SqlitePool, greeting_id: &str) -> Result<()> {
    repo::delete_agent_greeting(db, greeting_id).await
}

pub async fn add_media(
    db: &SqlitePool,
    store: &ContentStore,
    agent_id: &str,
    input: &AddAgentMediaInput,
) -> Result<AgentMediaDetail> {
    let _ = repo::get_agent(db, agent_id).await?;
    let content = content_service::create_content(db, store, &input.content).await?;
    let config_json = input.config_json.to_string();
    let row = repo::create_agent_media(
        db,
        &repo::CreateAgentMedia {
            agent_id,
            media_type: &input.media_type,
            content_id: &content.content_id,
            name: input.name.as_deref(),
            enabled: input.enabled,
            sort_order: input.sort_order,
            config_json: &config_json,
        },
    )
    .await?;

    map_agent_media_detail(db, store, row).await
}

pub async fn remove_media(db: &SqlitePool, media_id: &str) -> Result<()> {
    repo::delete_agent_media(db, media_id).await
}

pub async fn replace_default_presets(
    db: &SqlitePool,
    agent_id: &str,
    items: &[AgentResourceBindingInput],
) -> Result<()> {
    let _ = repo::get_agent(db, agent_id).await?;
    for item in items {
        let _ = presets::get_preset(db, &item.resource_id).await?;
    }

    let owned = items
        .iter()
        .map(|item| OwnedResourceBinding {
            resource_id: item.resource_id.clone(),
            binding_type: item.binding_type.clone(),
            enabled: item.enabled,
            sort_order: item.sort_order,
            config_json: item.config_json.to_string(),
        })
        .collect::<Vec<_>>();
    let records = owned
        .iter()
        .map(|item| repo::ResourceBindingRecord {
            resource_id: &item.resource_id,
            binding_type: &item.binding_type,
            enabled: item.enabled,
            sort_order: item.sort_order,
            config_json: &item.config_json,
        })
        .collect::<Vec<_>>();

    let mut tx = db.begin().await?;
    repo::replace_agent_preset_bindings(&mut tx, agent_id, &records).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn replace_default_lorebooks(
    db: &SqlitePool,
    agent_id: &str,
    items: &[AgentResourceBindingInput],
) -> Result<()> {
    let _ = repo::get_agent(db, agent_id).await?;
    for item in items {
        let _ = lorebooks::get_lorebook(db, &item.resource_id).await?;
    }

    let owned = items
        .iter()
        .map(|item| OwnedResourceBinding {
            resource_id: item.resource_id.clone(),
            binding_type: item.binding_type.clone(),
            enabled: item.enabled,
            sort_order: item.sort_order,
            config_json: item.config_json.to_string(),
        })
        .collect::<Vec<_>>();
    let records = owned
        .iter()
        .map(|item| repo::ResourceBindingRecord {
            resource_id: &item.resource_id,
            binding_type: &item.binding_type,
            enabled: item.enabled,
            sort_order: item.sort_order,
            config_json: &item.config_json,
        })
        .collect::<Vec<_>>();

    let mut tx = db.begin().await?;
    repo::replace_agent_lorebook_bindings(&mut tx, agent_id, &records).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn replace_default_user_profiles(
    db: &SqlitePool,
    agent_id: &str,
    items: &[AgentResourceBindingInput],
) -> Result<()> {
    let _ = repo::get_agent(db, agent_id).await?;
    for item in items {
        let _ = user_profiles::get_user_profile(db, &item.resource_id).await?;
    }

    let owned = items
        .iter()
        .map(|item| OwnedResourceBinding {
            resource_id: item.resource_id.clone(),
            binding_type: item.binding_type.clone(),
            enabled: item.enabled,
            sort_order: item.sort_order,
            config_json: item.config_json.to_string(),
        })
        .collect::<Vec<_>>();
    let records = owned
        .iter()
        .map(|item| repo::ResourceBindingRecord {
            resource_id: &item.resource_id,
            binding_type: &item.binding_type,
            enabled: item.enabled,
            sort_order: item.sort_order,
            config_json: &item.config_json,
        })
        .collect::<Vec<_>>();

    let mut tx = db.begin().await?;
    repo::replace_agent_user_profile_bindings(&mut tx, agent_id, &records).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn replace_default_channels(
    db: &SqlitePool,
    agent_id: &str,
    items: &[AgentChannelBindingInput],
) -> Result<()> {
    let _ = repo::get_agent(db, agent_id).await?;
    for item in items {
        let _ = channel_repo::get_channel(db, &item.channel_id).await?;
        if let Some(channel_model_id) = &item.channel_model_id {
            let channel_model = channel_repo::get_channel_model_by_id(db, channel_model_id).await?;
            if channel_model.channel_id != item.channel_id {
                return Err(AppError::Validation(
                    "channel_model_id does not belong to channel_id".to_string(),
                ));
            }
        }
    }

    let owned = items
        .iter()
        .map(|item| OwnedChannelBinding {
            channel_id: item.channel_id.clone(),
            channel_model_id: item.channel_model_id.clone(),
            binding_type: item.binding_type.clone(),
            enabled: item.enabled,
            sort_order: item.sort_order,
            config_json: item.config_json.to_string(),
        })
        .collect::<Vec<_>>();
    let records = owned
        .iter()
        .map(|item| repo::ChannelBindingRecord {
            channel_id: &item.channel_id,
            channel_model_id: item.channel_model_id.as_deref(),
            binding_type: &item.binding_type,
            enabled: item.enabled,
            sort_order: item.sort_order,
            config_json: &item.config_json,
        })
        .collect::<Vec<_>>();

    let mut tx = db.begin().await?;
    repo::replace_agent_channel_bindings(&mut tx, agent_id, &records).await?;
    tx.commit().await?;
    Ok(())
}

async fn build_agent_detail(
    db: &SqlitePool,
    store: &ContentStore,
    agent: AgentRow,
) -> Result<AgentDetail> {
    let greetings = repo::list_agent_greetings(db, &agent.id).await?;
    let media = repo::list_agent_media(db, &agent.id).await?;
    let preset_bindings = repo::list_agent_preset_bindings(db, &agent.id).await?;
    let lorebook_bindings = repo::list_agent_lorebook_bindings(db, &agent.id).await?;
    let user_profile_bindings = repo::list_agent_user_profile_bindings(db, &agent.id).await?;
    let channel_bindings = repo::list_agent_channel_bindings(db, &agent.id).await?;

    let mut greeting_details = Vec::with_capacity(greetings.len());
    for greeting in greetings {
        greeting_details.push(map_agent_greeting_detail(db, store, greeting).await?);
    }

    let mut media_details = Vec::with_capacity(media.len());
    for item in media {
        media_details.push(map_agent_media_detail(db, store, item).await?);
    }

    Ok(AgentDetail {
        summary: map_agent_summary(agent.clone())?,
        description_content: load_optional_content(
            db,
            store,
            agent.description_content_id.as_deref(),
            true,
        )
        .await?,
        personality_content: load_optional_content(
            db,
            store,
            agent.personality_content_id.as_deref(),
            true,
        )
        .await?,
        scenario_content: load_optional_content(
            db,
            store,
            agent.scenario_content_id.as_deref(),
            true,
        )
        .await?,
        example_messages_content: load_optional_content(
            db,
            store,
            agent.example_messages_content_id.as_deref(),
            true,
        )
        .await?,
        main_prompt_override_content: load_optional_content(
            db,
            store,
            agent.main_prompt_override_content_id.as_deref(),
            true,
        )
        .await?,
        post_history_instructions_content: load_optional_content(
            db,
            store,
            agent.post_history_instructions_content_id.as_deref(),
            true,
        )
        .await?,
        character_note_content: load_optional_content(
            db,
            store,
            agent.character_note_content_id.as_deref(),
            true,
        )
        .await?,
        creator_notes_content: load_optional_content(
            db,
            store,
            agent.creator_notes_content_id.as_deref(),
            true,
        )
        .await?,
        character_note_depth: agent.character_note_depth,
        character_note_role: agent
            .character_note_role
            .as_deref()
            .map(MessageRole::parse)
            .transpose()?,
        talkativeness: agent.talkativeness,
        creator_name: agent.creator_name,
        character_version: agent.character_version,
        greetings: greeting_details,
        media: media_details,
        preset_bindings: preset_bindings
            .into_iter()
            .map(map_resource_binding_row)
            .collect::<Result<Vec<_>>>()?,
        lorebook_bindings: lorebook_bindings
            .into_iter()
            .map(map_resource_binding_row)
            .collect::<Result<Vec<_>>>()?,
        user_profile_bindings: user_profile_bindings
            .into_iter()
            .map(map_resource_binding_row)
            .collect::<Result<Vec<_>>>()?,
        channel_bindings: channel_bindings
            .into_iter()
            .map(map_channel_binding_row)
            .collect::<Result<Vec<_>>>()?,
        config_json: parse_json(&agent.config_json, "agents.config_json")?,
    })
}

async fn prepare_agent_values_from_create(
    db: &SqlitePool,
    store: &ContentStore,
    input: &CreateAgentInput,
) -> Result<PreparedAgentValues> {
    Ok(PreparedAgentValues {
        description_content_id: store_optional_text_content(
            db,
            store,
            input.description_content.as_ref(),
        )
        .await?
        .map(|content| content.content_id),
        personality_content_id: store_optional_text_content(
            db,
            store,
            input.personality_content.as_ref(),
        )
        .await?
        .map(|content| content.content_id),
        scenario_content_id: store_optional_text_content(
            db,
            store,
            input.scenario_content.as_ref(),
        )
        .await?
        .map(|content| content.content_id),
        example_messages_content_id: store_optional_text_content(
            db,
            store,
            input.example_messages_content.as_ref(),
        )
        .await?
        .map(|content| content.content_id),
        main_prompt_override_content_id: store_optional_text_content(
            db,
            store,
            input.main_prompt_override_content.as_ref(),
        )
        .await?
        .map(|content| content.content_id),
        post_history_instructions_content_id: store_optional_text_content(
            db,
            store,
            input.post_history_instructions_content.as_ref(),
        )
        .await?
        .map(|content| content.content_id),
        character_note_content_id: store_optional_text_content(
            db,
            store,
            input.character_note_content.as_ref(),
        )
        .await?
        .map(|content| content.content_id),
        creator_notes_content_id: store_optional_text_content(
            db,
            store,
            input.creator_notes_content.as_ref(),
        )
        .await?
        .map(|content| content.content_id),
        config_json: input.config_json.to_string(),
    })
}

async fn prepare_agent_values_from_update(
    db: &SqlitePool,
    store: &ContentStore,
    input: &UpdateAgentInput,
) -> Result<PreparedAgentValues> {
    Ok(PreparedAgentValues {
        description_content_id: store_optional_text_content(
            db,
            store,
            input.description_content.as_ref(),
        )
        .await?
        .map(|content| content.content_id),
        personality_content_id: store_optional_text_content(
            db,
            store,
            input.personality_content.as_ref(),
        )
        .await?
        .map(|content| content.content_id),
        scenario_content_id: store_optional_text_content(
            db,
            store,
            input.scenario_content.as_ref(),
        )
        .await?
        .map(|content| content.content_id),
        example_messages_content_id: store_optional_text_content(
            db,
            store,
            input.example_messages_content.as_ref(),
        )
        .await?
        .map(|content| content.content_id),
        main_prompt_override_content_id: store_optional_text_content(
            db,
            store,
            input.main_prompt_override_content.as_ref(),
        )
        .await?
        .map(|content| content.content_id),
        post_history_instructions_content_id: store_optional_text_content(
            db,
            store,
            input.post_history_instructions_content.as_ref(),
        )
        .await?
        .map(|content| content.content_id),
        character_note_content_id: store_optional_text_content(
            db,
            store,
            input.character_note_content.as_ref(),
        )
        .await?
        .map(|content| content.content_id),
        creator_notes_content_id: store_optional_text_content(
            db,
            store,
            input.creator_notes_content.as_ref(),
        )
        .await?
        .map(|content| content.content_id),
        config_json: input.config_json.to_string(),
    })
}

struct PreparedAgentValues {
    description_content_id: Option<String>,
    personality_content_id: Option<String>,
    scenario_content_id: Option<String>,
    example_messages_content_id: Option<String>,
    main_prompt_override_content_id: Option<String>,
    post_history_instructions_content_id: Option<String>,
    character_note_content_id: Option<String>,
    creator_notes_content_id: Option<String>,
    config_json: String,
}

impl PreparedAgentValues {
    fn as_repo_input_from_create<'a>(
        &'a self,
        input: &'a CreateAgentInput,
    ) -> repo::CreateOrUpdateAgent<'a> {
        repo::CreateOrUpdateAgent {
            name: &input.name,
            title: input.title.as_deref(),
            description_content_id: self.description_content_id.as_deref(),
            personality_content_id: self.personality_content_id.as_deref(),
            scenario_content_id: self.scenario_content_id.as_deref(),
            example_messages_content_id: self.example_messages_content_id.as_deref(),
            main_prompt_override_content_id: self.main_prompt_override_content_id.as_deref(),
            post_history_instructions_content_id: self
                .post_history_instructions_content_id
                .as_deref(),
            character_note_content_id: self.character_note_content_id.as_deref(),
            creator_notes_content_id: self.creator_notes_content_id.as_deref(),
            character_note_depth: input.character_note_depth,
            character_note_role: input.character_note_role.map(MessageRole::as_str),
            talkativeness: input.talkativeness,
            avatar_uri: input.avatar_uri.as_deref(),
            creator_name: input.creator_name.as_deref(),
            character_version: input.character_version.as_deref(),
            enabled: input.enabled,
            sort_order: input.sort_order,
            config_json: &self.config_json,
        }
    }

    fn as_repo_input_from_update<'a>(
        &'a self,
        input: &'a UpdateAgentInput,
    ) -> repo::CreateOrUpdateAgent<'a> {
        repo::CreateOrUpdateAgent {
            name: &input.name,
            title: input.title.as_deref(),
            description_content_id: self.description_content_id.as_deref(),
            personality_content_id: self.personality_content_id.as_deref(),
            scenario_content_id: self.scenario_content_id.as_deref(),
            example_messages_content_id: self.example_messages_content_id.as_deref(),
            main_prompt_override_content_id: self.main_prompt_override_content_id.as_deref(),
            post_history_instructions_content_id: self
                .post_history_instructions_content_id
                .as_deref(),
            character_note_content_id: self.character_note_content_id.as_deref(),
            creator_notes_content_id: self.creator_notes_content_id.as_deref(),
            character_note_depth: input.character_note_depth,
            character_note_role: input.character_note_role.map(MessageRole::as_str),
            talkativeness: input.talkativeness,
            avatar_uri: input.avatar_uri.as_deref(),
            creator_name: input.creator_name.as_deref(),
            character_version: input.character_version.as_deref(),
            enabled: input.enabled,
            sort_order: input.sort_order,
            config_json: &self.config_json,
        }
    }
}

struct OwnedResourceBinding {
    resource_id: String,
    binding_type: String,
    enabled: bool,
    sort_order: i64,
    config_json: String,
}

struct OwnedChannelBinding {
    channel_id: String,
    channel_model_id: Option<String>,
    binding_type: String,
    enabled: bool,
    sort_order: i64,
    config_json: String,
}

fn map_agent_summary(row: AgentRow) -> Result<AgentSummary> {
    Ok(AgentSummary {
        id: row.id,
        name: row.name,
        title: row.title,
        avatar_uri: row.avatar_uri,
        enabled: row.enabled,
        sort_order: row.sort_order,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

async fn map_agent_greeting_detail(
    db: &SqlitePool,
    store: &ContentStore,
    row: AgentGreetingRow,
) -> Result<AgentGreetingDetail> {
    Ok(AgentGreetingDetail {
        id: row.id,
        agent_id: row.agent_id,
        greeting_type: row.greeting_type,
        name: row.name,
        primary_content: content_service::get_content(db, store, &row.primary_content_id, true)
            .await?,
        enabled: row.enabled,
        sort_order: row.sort_order,
        config_json: parse_json(&row.config_json, "agent_greetings.config_json")?,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

async fn map_agent_media_detail(
    db: &SqlitePool,
    store: &ContentStore,
    row: AgentMediaRow,
) -> Result<AgentMediaDetail> {
    Ok(AgentMediaDetail {
        id: row.id,
        agent_id: row.agent_id,
        media_type: row.media_type,
        name: row.name,
        content: content_service::get_content(db, store, &row.content_id, false).await?,
        enabled: row.enabled,
        sort_order: row.sort_order,
        config_json: parse_json(&row.config_json, "agent_media.config_json")?,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

fn map_resource_binding_row(row: AgentResourceBindingRow) -> Result<ResourceBindingDetail> {
    Ok(ResourceBindingDetail {
        id: row.id,
        resource_id: row.resource_id,
        binding_type: row.binding_type,
        enabled: row.enabled,
        sort_order: row.sort_order,
        config_json: parse_json(&row.config_json, "agent_resource_bindings.config_json")?,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

fn map_channel_binding_row(row: AgentChannelBindingRow) -> Result<ChannelBindingDetail> {
    Ok(ChannelBindingDetail {
        id: row.id,
        channel_id: row.channel_id,
        channel_model_id: row.channel_model_id,
        binding_type: row.binding_type,
        enabled: row.enabled,
        sort_order: row.sort_order,
        config_json: parse_json(&row.config_json, "agent_channel_bindings.config_json")?,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

async fn store_optional_text_content(
    db: &SqlitePool,
    store: &ContentStore,
    input: Option<&ContentWriteInput>,
) -> Result<Option<StoredContent>> {
    let Some(input) = input else {
        return Ok(None);
    };

    ensure_textual_content(&input.content_type, "agent content")?;
    content_service::create_content(db, store, input)
        .await
        .map(Some)
}

async fn load_optional_content(
    db: &SqlitePool,
    store: &ContentStore,
    content_id: Option<&str>,
    include_body: bool,
) -> Result<Option<StoredContent>> {
    let Some(content_id) = content_id else {
        return Ok(None);
    };

    content_service::get_content(db, store, content_id, include_body)
        .await
        .map(Some)
}

fn parse_json(raw: &str, field: &'static str) -> Result<serde_json::Value> {
    serde_json::from_str(raw)
        .map_err(|err| AppError::Validation(format!("failed to parse {field} as json: {err}")))
}

fn ensure_textual_content(content_type: &ContentType, label: &str) -> Result<()> {
    match content_type {
        ContentType::Text | ContentType::Markdown | ContentType::Html | ContentType::Json => Ok(()),
        _ => Err(AppError::Validation(format!("{label} must be textual"))),
    }
}
