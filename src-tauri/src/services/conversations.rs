use sqlx::SqlitePool;

use crate::db::models::{
    ConversationChannelBindingRow, ConversationParticipantRow, ConversationResourceBindingRow,
    ConversationRow,
};
use crate::db::repos::{
    agents, api_channels as channel_repo, conversations as repo, lorebooks, presets, user_profiles,
};
use crate::domain::common::{
    ChannelBindingDetail, ChannelBindingInput, ResourceBindingDetail, ResourceBindingInput,
};
use crate::domain::conversations::{
    ConversationDetail, ConversationParticipantDetail, ConversationParticipantInput,
    ConversationSummary, CreateConversationInput, UpdateConversationMetaInput,
};
use crate::support::error::{AppError, Result};

pub async fn list_conversations(db: &SqlitePool) -> Result<Vec<ConversationSummary>> {
    repo::list_conversations(db)
        .await?
        .into_iter()
        .map(map_conversation_summary)
        .collect()
}

pub async fn get_conversation_detail(db: &SqlitePool, id: &str) -> Result<ConversationDetail> {
    let conversation = repo::get_conversation(db, id).await?;
    build_conversation_detail(db, conversation).await
}

pub async fn create_conversation(
    db: &SqlitePool,
    input: &CreateConversationInput,
) -> Result<ConversationDetail> {
    validate_participants(db, &input.participants).await?;

    let conversation = repo::create_conversation(
        db,
        &repo::CreateConversationRecord {
            title: &input.title,
            description: input.description.as_deref(),
            conversation_mode: &input.conversation_mode,
            archived: input.archived,
            pinned: input.pinned,
            config_json: &input.config_json.to_string(),
        },
    )
    .await?;

    let owned_participants = to_owned_participants(&input.participants);
    let participants = owned_participants
        .iter()
        .map(|item| repo::ConversationParticipantRecord {
            agent_id: item.agent_id.as_deref(),
            display_name: item.display_name.as_deref(),
            participant_type: &item.participant_type,
            enabled: item.enabled,
            sort_order: item.sort_order,
            config_json: &item.config_json,
        })
        .collect::<Vec<_>>();
    let mut tx = db.begin().await?;
    repo::replace_conversation_participants(&mut tx, &conversation.id, &participants).await?;
    tx.commit().await?;

    build_conversation_detail(db, conversation).await
}

pub async fn rename_conversation(
    db: &SqlitePool,
    id: &str,
    title: &str,
) -> Result<ConversationDetail> {
    let row = repo::rename_conversation(db, id, title).await?;
    build_conversation_detail(db, row).await
}

pub async fn update_conversation_meta(
    db: &SqlitePool,
    id: &str,
    input: &UpdateConversationMetaInput,
) -> Result<ConversationDetail> {
    let row = repo::update_conversation(
        db,
        id,
        &repo::UpdateConversationRecord {
            title: &input.title,
            description: input.description.as_deref(),
            archived: input.archived,
            pinned: input.pinned,
            config_json: &input.config_json.to_string(),
        },
    )
    .await?;

    build_conversation_detail(db, row).await
}

pub async fn delete_conversation(db: &SqlitePool, id: &str) -> Result<()> {
    repo::delete_conversation(db, id).await
}

pub async fn replace_participants(
    db: &SqlitePool,
    conversation_id: &str,
    items: &[ConversationParticipantInput],
) -> Result<()> {
    let _ = repo::get_conversation(db, conversation_id).await?;
    validate_participants(db, items).await?;
    let owned = to_owned_participants(items);
    let records = owned
        .iter()
        .map(|item| repo::ConversationParticipantRecord {
            agent_id: item.agent_id.as_deref(),
            display_name: item.display_name.as_deref(),
            participant_type: &item.participant_type,
            enabled: item.enabled,
            sort_order: item.sort_order,
            config_json: &item.config_json,
        })
        .collect::<Vec<_>>();
    let mut tx = db.begin().await?;
    repo::replace_conversation_participants(&mut tx, conversation_id, &records).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn replace_presets(
    db: &SqlitePool,
    conversation_id: &str,
    items: &[ResourceBindingInput],
) -> Result<()> {
    let _ = repo::get_conversation(db, conversation_id).await?;
    for item in items {
        let _ = presets::get_preset(db, &item.resource_id).await?;
    }

    let owned = to_owned_resource_bindings(items);
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
    repo::replace_conversation_preset_bindings(&mut tx, conversation_id, &records).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn replace_lorebooks(
    db: &SqlitePool,
    conversation_id: &str,
    items: &[ResourceBindingInput],
) -> Result<()> {
    let _ = repo::get_conversation(db, conversation_id).await?;
    for item in items {
        let _ = lorebooks::get_lorebook(db, &item.resource_id).await?;
    }

    let owned = to_owned_resource_bindings(items);
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
    repo::replace_conversation_lorebook_bindings(&mut tx, conversation_id, &records).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn replace_user_profiles(
    db: &SqlitePool,
    conversation_id: &str,
    items: &[ResourceBindingInput],
) -> Result<()> {
    let _ = repo::get_conversation(db, conversation_id).await?;
    for item in items {
        let _ = user_profiles::get_user_profile(db, &item.resource_id).await?;
    }

    let owned = to_owned_resource_bindings(items);
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
    repo::replace_conversation_user_profile_bindings(&mut tx, conversation_id, &records).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn replace_channels(
    db: &SqlitePool,
    conversation_id: &str,
    items: &[ChannelBindingInput],
) -> Result<()> {
    let _ = repo::get_conversation(db, conversation_id).await?;
    for item in items {
        let _ = channel_repo::get_channel(db, &item.channel_id).await?;
        if let Some(channel_model_id) = &item.channel_model_id {
            let model = channel_repo::get_channel_model_by_id(db, channel_model_id).await?;
            if model.channel_id != item.channel_id {
                return Err(AppError::Validation(
                    "channel_model_id does not belong to channel_id".to_string(),
                ));
            }
        }
    }

    let owned = to_owned_channel_bindings(items);
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
    repo::replace_conversation_channel_bindings(&mut tx, conversation_id, &records).await?;
    tx.commit().await?;
    Ok(())
}

async fn build_conversation_detail(
    db: &SqlitePool,
    conversation: ConversationRow,
) -> Result<ConversationDetail> {
    let participants = repo::list_conversation_participants(db, &conversation.id).await?;
    let preset_bindings = repo::list_conversation_preset_bindings(db, &conversation.id).await?;
    let lorebook_bindings = repo::list_conversation_lorebook_bindings(db, &conversation.id).await?;
    let user_profile_bindings =
        repo::list_conversation_user_profile_bindings(db, &conversation.id).await?;
    let channel_bindings = repo::list_conversation_channel_bindings(db, &conversation.id).await?;

    Ok(ConversationDetail {
        summary: map_conversation_summary(conversation)?,
        participants: participants
            .into_iter()
            .map(map_conversation_participant)
            .collect::<Result<Vec<_>>>()?,
        preset_bindings: preset_bindings
            .into_iter()
            .map(map_resource_binding)
            .collect::<Result<Vec<_>>>()?,
        lorebook_bindings: lorebook_bindings
            .into_iter()
            .map(map_resource_binding)
            .collect::<Result<Vec<_>>>()?,
        user_profile_bindings: user_profile_bindings
            .into_iter()
            .map(map_resource_binding)
            .collect::<Result<Vec<_>>>()?,
        channel_bindings: channel_bindings
            .into_iter()
            .map(map_channel_binding)
            .collect::<Result<Vec<_>>>()?,
    })
}

fn map_conversation_summary(row: ConversationRow) -> Result<ConversationSummary> {
    Ok(ConversationSummary {
        id: row.id,
        title: row.title,
        description: row.description,
        conversation_mode: row.conversation_mode,
        archived: row.archived,
        pinned: row.pinned,
        config_json: parse_json(&row.config_json, "conversations.config_json")?,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

fn map_conversation_participant(
    row: ConversationParticipantRow,
) -> Result<ConversationParticipantDetail> {
    Ok(ConversationParticipantDetail {
        id: row.id,
        conversation_id: row.conversation_id,
        agent_id: row.agent_id,
        display_name: row.display_name,
        participant_type: row.participant_type,
        enabled: row.enabled,
        sort_order: row.sort_order,
        config_json: parse_json(&row.config_json, "conversation_participants.config_json")?,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

fn map_resource_binding(row: ConversationResourceBindingRow) -> Result<ResourceBindingDetail> {
    Ok(ResourceBindingDetail {
        id: row.id,
        resource_id: row.resource_id,
        binding_type: row.binding_type,
        enabled: row.enabled,
        sort_order: row.sort_order,
        config_json: parse_json(
            &row.config_json,
            "conversation_resource_bindings.config_json",
        )?,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

fn map_channel_binding(row: ConversationChannelBindingRow) -> Result<ChannelBindingDetail> {
    Ok(ChannelBindingDetail {
        id: row.id,
        channel_id: row.channel_id,
        channel_model_id: row.channel_model_id,
        binding_type: row.binding_type,
        enabled: row.enabled,
        sort_order: row.sort_order,
        config_json: parse_json(
            &row.config_json,
            "conversation_channel_bindings.config_json",
        )?,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

async fn validate_participants(
    db: &SqlitePool,
    items: &[ConversationParticipantInput],
) -> Result<()> {
    for item in items {
        match item.participant_type.as_str() {
            "agent" => {
                let agent_id = item.agent_id.as_deref().ok_or_else(|| {
                    AppError::Validation("agent participant requires agent_id".to_string())
                })?;
                let _ = agents::get_agent(db, agent_id).await?;
            }
            "human" | "system" => {}
            _ => {
                return Err(AppError::Validation(format!(
                    "unsupported participant_type '{}'",
                    item.participant_type
                )))
            }
        }
    }

    Ok(())
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

struct OwnedParticipant {
    agent_id: Option<String>,
    display_name: Option<String>,
    participant_type: String,
    enabled: bool,
    sort_order: i64,
    config_json: String,
}

fn to_owned_resource_bindings(items: &[ResourceBindingInput]) -> Vec<OwnedResourceBinding> {
    items
        .iter()
        .map(|item| OwnedResourceBinding {
            resource_id: item.resource_id.clone(),
            binding_type: item.binding_type.clone(),
            enabled: item.enabled,
            sort_order: item.sort_order,
            config_json: item.config_json.to_string(),
        })
        .collect()
}

fn to_owned_channel_bindings(items: &[ChannelBindingInput]) -> Vec<OwnedChannelBinding> {
    items
        .iter()
        .map(|item| OwnedChannelBinding {
            channel_id: item.channel_id.clone(),
            channel_model_id: item.channel_model_id.clone(),
            binding_type: item.binding_type.clone(),
            enabled: item.enabled,
            sort_order: item.sort_order,
            config_json: item.config_json.to_string(),
        })
        .collect()
}

fn to_owned_participants(items: &[ConversationParticipantInput]) -> Vec<OwnedParticipant> {
    items
        .iter()
        .map(|item| OwnedParticipant {
            agent_id: item.agent_id.clone(),
            display_name: item.display_name.clone(),
            participant_type: item.participant_type.clone(),
            enabled: item.enabled,
            sort_order: item.sort_order,
            config_json: item.config_json.to_string(),
        })
        .collect()
}

fn parse_json(raw: &str, field: &'static str) -> Result<serde_json::Value> {
    serde_json::from_str(raw)
        .map_err(|err| AppError::Validation(format!("failed to parse {field} as json: {err}")))
}
