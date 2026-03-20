use sqlx::SqlitePool;

use crate::db::models::{PresetChannelBindingRow, PresetEntryRow, PresetRow};
use crate::db::repos::{api_channels as channel_repo, presets as repo};
use crate::domain::common::ChannelBindingDetail;
use crate::domain::content::ContentType;
use crate::domain::messages::MessageRole;
use crate::domain::presets::{
    CreatePresetEntryInput, CreatePresetInput, PresetChannelBindingInput, PresetDetail,
    PresetEntryDetail, PresetSummary, UpdatePresetEntryInput, UpdatePresetInput,
};
use crate::services::content as content_service;
use crate::services::content_store::ContentStore;
use crate::support::error::{AppError, Result};

pub async fn list_presets(db: &SqlitePool) -> Result<Vec<PresetSummary>> {
    repo::list_presets(db)
        .await?
        .into_iter()
        .map(map_preset_summary)
        .collect()
}

pub async fn get_preset_detail(
    db: &SqlitePool,
    store: &ContentStore,
    id: &str,
) -> Result<PresetDetail> {
    let preset = repo::get_preset(db, id).await?;
    build_preset_detail(db, store, preset).await
}

pub async fn create_preset(
    db: &SqlitePool,
    store: &ContentStore,
    input: &CreatePresetInput,
) -> Result<PresetDetail> {
    let preset = repo::create_preset(
        db,
        &input.name,
        input.description.as_deref(),
        input.enabled,
        input.sort_order,
        &input.config_json.to_string(),
    )
    .await?;

    build_preset_detail(db, store, preset).await
}

pub async fn update_preset(
    db: &SqlitePool,
    store: &ContentStore,
    id: &str,
    input: &UpdatePresetInput,
) -> Result<PresetDetail> {
    let preset = repo::update_preset(
        db,
        id,
        &input.name,
        input.description.as_deref(),
        input.enabled,
        input.sort_order,
        &input.config_json.to_string(),
    )
    .await?;

    build_preset_detail(db, store, preset).await
}

pub async fn delete_preset(db: &SqlitePool, id: &str) -> Result<()> {
    repo::delete_preset(db, id).await
}

pub async fn create_entry(
    db: &SqlitePool,
    store: &ContentStore,
    input: &CreatePresetEntryInput,
) -> Result<PresetEntryDetail> {
    ensure_prompt_content(&input.primary_content.content_type)?;
    let stored = content_service::create_content(db, store, &input.primary_content).await?;
    let row = repo::create_preset_entry(
        db,
        &repo::InsertPresetEntry {
            preset_id: &input.preset_id,
            name: &input.name,
            role: input.role.as_str(),
            primary_content_id: &stored.content_id,
            position_type: &input.position_type,
            list_order: input.list_order,
            depth: input.depth,
            depth_order: input.depth_order,
            triggers_json: &input.triggers_json.to_string(),
            enabled: input.enabled,
            is_pinned: input.is_pinned,
            config_json: &input.config_json.to_string(),
        },
    )
    .await?;

    map_preset_entry_detail(db, store, row).await
}

pub async fn update_entry(
    db: &SqlitePool,
    store: &ContentStore,
    id: &str,
    input: &UpdatePresetEntryInput,
) -> Result<PresetEntryDetail> {
    ensure_prompt_content(&input.primary_content.content_type)?;
    let stored = content_service::create_content(db, store, &input.primary_content).await?;
    let row = repo::update_preset_entry(
        db,
        id,
        &repo::UpdatePresetEntry {
            name: &input.name,
            role: input.role.as_str(),
            primary_content_id: &stored.content_id,
            position_type: &input.position_type,
            list_order: input.list_order,
            depth: input.depth,
            depth_order: input.depth_order,
            triggers_json: &input.triggers_json.to_string(),
            enabled: input.enabled,
            is_pinned: input.is_pinned,
            config_json: &input.config_json.to_string(),
        },
    )
    .await?;

    map_preset_entry_detail(db, store, row).await
}

pub async fn delete_entry(db: &SqlitePool, id: &str) -> Result<()> {
    repo::delete_preset_entry(db, id).await
}

pub async fn reorder_entries(db: &SqlitePool, preset_id: &str, entry_ids: &[String]) -> Result<()> {
    let mut tx = db.begin().await?;
    repo::reorder_preset_entries(&mut tx, preset_id, entry_ids).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn bind_channel(
    db: &SqlitePool,
    preset_id: &str,
    input: &PresetChannelBindingInput,
) -> Result<ChannelBindingDetail> {
    let _ = repo::get_preset(db, preset_id).await?;
    let _ = channel_repo::get_channel(db, &input.channel_id).await?;
    if let Some(channel_model_id) = &input.channel_model_id {
        let channel_model = channel_repo::get_channel_model_by_id(db, channel_model_id).await?;
        if channel_model.channel_id != input.channel_id {
            return Err(AppError::Validation(
                "channel_model_id does not belong to channel_id".to_string(),
            ));
        }
    }

    let row = repo::upsert_preset_channel_binding(
        db,
        &repo::UpsertPresetChannelBinding {
            preset_id,
            channel_id: &input.channel_id,
            channel_model_id: input.channel_model_id.as_deref(),
            binding_type: &input.binding_type,
            enabled: input.enabled,
            sort_order: input.sort_order,
            config_json: &input.config_json.to_string(),
        },
    )
    .await?;

    map_channel_binding_row(row)
}

pub async fn unbind_channel(
    db: &SqlitePool,
    preset_id: &str,
    channel_id: &str,
    channel_model_id: Option<&str>,
) -> Result<()> {
    repo::delete_preset_channel_binding(db, preset_id, channel_id, channel_model_id).await
}

async fn build_preset_detail(
    db: &SqlitePool,
    store: &ContentStore,
    preset: PresetRow,
) -> Result<PresetDetail> {
    let entries = repo::list_preset_entries(db, &preset.id).await?;
    let bindings = repo::list_preset_channel_bindings(db, &preset.id).await?;

    let mut entry_details = Vec::with_capacity(entries.len());
    for entry in entries {
        entry_details.push(map_preset_entry_detail(db, store, entry).await?);
    }

    let channel_bindings = bindings
        .into_iter()
        .map(map_channel_binding_row)
        .collect::<Result<Vec<_>>>()?;

    Ok(PresetDetail {
        preset: map_preset_summary(preset)?,
        entries: entry_details,
        channel_bindings,
    })
}

fn map_preset_summary(row: PresetRow) -> Result<PresetSummary> {
    Ok(PresetSummary {
        id: row.id,
        name: row.name,
        description: row.description,
        enabled: row.enabled,
        sort_order: row.sort_order,
        config_json: parse_json(&row.config_json, "presets.config_json")?,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

async fn map_preset_entry_detail(
    db: &SqlitePool,
    store: &ContentStore,
    row: PresetEntryRow,
) -> Result<PresetEntryDetail> {
    let name = row
        .name
        .ok_or_else(|| AppError::Validation("preset_entries.name must not be null".to_string()))?;

    Ok(PresetEntryDetail {
        id: row.id,
        preset_id: row.preset_id,
        name,
        role: MessageRole::parse(&row.role)?,
        primary_content: content_service::get_content(db, store, &row.primary_content_id, true)
            .await?,
        position_type: row.position_type,
        list_order: row.list_order,
        depth: row.depth,
        depth_order: row.depth_order,
        triggers_json: parse_json(&row.triggers_json, "preset_entries.triggers_json")?,
        enabled: row.enabled,
        is_pinned: row.is_pinned,
        config_json: parse_json(&row.config_json, "preset_entries.config_json")?,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

fn map_channel_binding_row(row: PresetChannelBindingRow) -> Result<ChannelBindingDetail> {
    Ok(ChannelBindingDetail {
        id: row.id,
        channel_id: row.channel_id,
        channel_model_id: row.channel_model_id,
        binding_type: row.binding_type,
        enabled: row.enabled,
        sort_order: row.sort_order,
        config_json: parse_json(&row.config_json, "preset_channel_bindings.config_json")?,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

fn parse_json(raw: &str, field: &'static str) -> Result<serde_json::Value> {
    serde_json::from_str(raw)
        .map_err(|err| AppError::Validation(format!("failed to parse {field} as json: {err}")))
}

fn ensure_prompt_content(content_type: &ContentType) -> Result<()> {
    match content_type {
        ContentType::Text | ContentType::Markdown | ContentType::Html | ContentType::Json => Ok(()),
        _ => Err(AppError::Validation(
            "preset entry content must be textual".to_string(),
        )),
    }
}
