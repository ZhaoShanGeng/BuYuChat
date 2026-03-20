use std::collections::HashMap;

use regex::RegexBuilder;
use sqlx::SqlitePool;

use crate::db::models::{LorebookEntryKeyRow, LorebookEntryRow, LorebookRow};
use crate::db::repos::lorebooks as repo;
use crate::domain::content::ContentType;
use crate::domain::lorebooks::{
    CreateLorebookEntryInput, CreateLorebookInput, LorebookDetail, LorebookEntryDetail,
    LorebookEntryKeyDetail, LorebookMatchInput, LorebookSummary, MatchedLorebookEntry,
    UpdateLorebookEntryInput, UpdateLorebookInput,
};
use crate::domain::messages::MessageRole;
use crate::services::content as content_service;
use crate::services::content_store::ContentStore;
use crate::support::error::{AppError, Result};

pub async fn list_lorebooks(db: &SqlitePool) -> Result<Vec<LorebookSummary>> {
    repo::list_lorebooks(db)
        .await?
        .into_iter()
        .map(map_lorebook_summary)
        .collect()
}

pub async fn get_lorebook_detail(
    db: &SqlitePool,
    store: &ContentStore,
    id: &str,
) -> Result<LorebookDetail> {
    let lorebook = repo::get_lorebook(db, id).await?;
    build_lorebook_detail(db, store, lorebook).await
}

pub async fn create_lorebook(
    db: &SqlitePool,
    store: &ContentStore,
    input: &CreateLorebookInput,
) -> Result<LorebookDetail> {
    let lorebook = repo::create_lorebook(
        db,
        &input.name,
        input.description.as_deref(),
        input.scan_depth,
        input.token_budget,
        &input.insertion_strategy,
        input.enabled,
        input.sort_order,
        &input.config_json.to_string(),
    )
    .await?;

    build_lorebook_detail(db, store, lorebook).await
}

pub async fn update_lorebook(
    db: &SqlitePool,
    store: &ContentStore,
    id: &str,
    input: &UpdateLorebookInput,
) -> Result<LorebookDetail> {
    let lorebook = repo::update_lorebook(
        db,
        id,
        &input.name,
        input.description.as_deref(),
        input.scan_depth,
        input.token_budget,
        &input.insertion_strategy,
        input.enabled,
        input.sort_order,
        &input.config_json.to_string(),
    )
    .await?;

    build_lorebook_detail(db, store, lorebook).await
}

pub async fn delete_lorebook(db: &SqlitePool, id: &str) -> Result<()> {
    repo::delete_lorebook(db, id).await
}

pub async fn create_entry(
    db: &SqlitePool,
    store: &ContentStore,
    input: &CreateLorebookEntryInput,
) -> Result<LorebookEntryDetail> {
    ensure_prompt_content(&input.primary_content.content_type)?;
    let stored = content_service::create_content(db, store, &input.primary_content).await?;
    let row = repo::create_lorebook_entry(
        db,
        &repo::InsertLorebookEntry {
            lorebook_id: &input.lorebook_id,
            title: input.title.as_deref(),
            primary_content_id: &stored.content_id,
            activation_strategy: &input.activation_strategy,
            keyword_logic: &input.keyword_logic,
            insertion_position: &input.insertion_position,
            insertion_order: input.insertion_order,
            insertion_depth: input.insertion_depth,
            insertion_role: input.insertion_role.map(MessageRole::as_str),
            outlet_name: input.outlet_name.as_deref(),
            entry_scope: &input.entry_scope,
            enabled: input.enabled,
            sort_order: input.sort_order,
            config_json: &input.config_json.to_string(),
        },
    )
    .await?;

    map_lorebook_entry_detail(db, store, row, Vec::new()).await
}

pub async fn update_entry(
    db: &SqlitePool,
    store: &ContentStore,
    id: &str,
    input: &UpdateLorebookEntryInput,
) -> Result<LorebookEntryDetail> {
    ensure_prompt_content(&input.primary_content.content_type)?;
    let stored = content_service::create_content(db, store, &input.primary_content).await?;
    let row = repo::update_lorebook_entry(
        db,
        id,
        &repo::UpdateLorebookEntry {
            title: input.title.as_deref(),
            primary_content_id: &stored.content_id,
            activation_strategy: &input.activation_strategy,
            keyword_logic: &input.keyword_logic,
            insertion_position: &input.insertion_position,
            insertion_order: input.insertion_order,
            insertion_depth: input.insertion_depth,
            insertion_role: input.insertion_role.map(MessageRole::as_str),
            outlet_name: input.outlet_name.as_deref(),
            entry_scope: &input.entry_scope,
            enabled: input.enabled,
            sort_order: input.sort_order,
            config_json: &input.config_json.to_string(),
        },
    )
    .await?;

    let entry_keys = repo::list_lorebook_entry_keys_by_lorebook(db, &row.lorebook_id)
        .await?
        .into_iter()
        .filter(|key| key.entry_id == row.id)
        .collect();
    map_lorebook_entry_detail(db, store, row, entry_keys).await
}

pub async fn delete_entry(db: &SqlitePool, id: &str) -> Result<()> {
    repo::delete_lorebook_entry(db, id).await
}

pub async fn replace_keys(db: &SqlitePool, entry_id: &str, keys: &[String]) -> Result<()> {
    let _ = repo::get_lorebook_entry(db, entry_id).await?;
    let mut tx = db.begin().await?;
    repo::replace_lorebook_entry_keys(&mut tx, entry_id, keys).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn match_entries(
    db: &SqlitePool,
    store: &ContentStore,
    input: &LorebookMatchInput,
) -> Result<Vec<MatchedLorebookEntry>> {
    let lorebook = repo::get_lorebook(db, &input.lorebook_id).await?;
    let detail = build_lorebook_detail(db, store, lorebook).await?;
    let corpus = input.recent_messages.join("\n");

    let mut matched = Vec::new();
    for entry in detail.entries {
        if !input.include_disabled && !entry.enabled {
            continue;
        }

        match entry.activation_strategy.as_str() {
            "constant" => matched.push(MatchedLorebookEntry {
                lorebook_entry_id: entry.id,
                score: 1.0,
                matched_keys: Vec::new(),
                content: entry.primary_content,
                config_json: entry.config_json,
            }),
            "keyword" => {
                if let Some((score, matched_keys)) = evaluate_keyword_match(&entry, &corpus)? {
                    matched.push(MatchedLorebookEntry {
                        lorebook_entry_id: entry.id,
                        score,
                        matched_keys,
                        content: entry.primary_content,
                        config_json: entry.config_json,
                    });
                }
            }
            "vector" => {}
            _ => {}
        }
    }

    matched.sort_by(|a, b| b.score.total_cmp(&a.score));
    if matched.len() > input.max_entries {
        matched.truncate(input.max_entries);
    }

    Ok(matched)
}

async fn build_lorebook_detail(
    db: &SqlitePool,
    store: &ContentStore,
    lorebook: LorebookRow,
) -> Result<LorebookDetail> {
    let entries = repo::list_lorebook_entries(db, &lorebook.id).await?;
    let keys = repo::list_lorebook_entry_keys_by_lorebook(db, &lorebook.id).await?;

    let mut keys_by_entry: HashMap<String, Vec<LorebookEntryKeyRow>> = HashMap::new();
    for key in keys {
        keys_by_entry
            .entry(key.entry_id.clone())
            .or_default()
            .push(key);
    }

    let mut entry_details = Vec::with_capacity(entries.len());
    for entry in entries {
        let entry_keys = keys_by_entry.remove(&entry.id).unwrap_or_default();
        entry_details.push(map_lorebook_entry_detail(db, store, entry, entry_keys).await?);
    }

    Ok(LorebookDetail {
        lorebook: map_lorebook_summary(lorebook)?,
        entries: entry_details,
    })
}

fn map_lorebook_summary(row: LorebookRow) -> Result<LorebookSummary> {
    Ok(LorebookSummary {
        id: row.id,
        name: row.name,
        description: row.description,
        scan_depth: row.scan_depth,
        token_budget: row.token_budget,
        insertion_strategy: row.insertion_strategy,
        enabled: row.enabled,
        sort_order: row.sort_order,
        config_json: parse_json(&row.config_json, "lorebooks.config_json")?,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

async fn map_lorebook_entry_detail(
    db: &SqlitePool,
    store: &ContentStore,
    row: LorebookEntryRow,
    keys: Vec<LorebookEntryKeyRow>,
) -> Result<LorebookEntryDetail> {
    let mut key_details = Vec::with_capacity(keys.len());
    for key in keys {
        key_details.push(LorebookEntryKeyDetail {
            id: key.id,
            entry_id: key.entry_id,
            key_type: key.key_type,
            match_type: key.match_type,
            pattern_text: key.pattern_text,
            case_sensitive: key.case_sensitive,
            enabled: key.enabled,
            sort_order: key.sort_order,
            config_json: parse_json(&key.config_json, "lorebook_entry_keys.config_json")?,
        });
    }

    Ok(LorebookEntryDetail {
        id: row.id,
        lorebook_id: row.lorebook_id,
        title: row.title,
        primary_content: content_service::get_content(db, store, &row.primary_content_id, true)
            .await?,
        activation_strategy: row.activation_strategy,
        keyword_logic: row.keyword_logic,
        insertion_position: row.insertion_position,
        insertion_order: row.insertion_order,
        insertion_depth: row.insertion_depth,
        insertion_role: row
            .insertion_role
            .as_deref()
            .map(MessageRole::parse)
            .transpose()?,
        outlet_name: row.outlet_name,
        entry_scope: row.entry_scope,
        enabled: row.enabled,
        sort_order: row.sort_order,
        keys: key_details,
        config_json: parse_json(&row.config_json, "lorebook_entries.config_json")?,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

fn evaluate_keyword_match(
    entry: &LorebookEntryDetail,
    corpus: &str,
) -> Result<Option<(f32, Vec<String>)>> {
    let enabled_keys = entry
        .keys
        .iter()
        .filter(|key| key.enabled)
        .collect::<Vec<_>>();
    if enabled_keys.is_empty() {
        return Ok(None);
    }

    let mut matched_keys = Vec::new();
    for key in &enabled_keys {
        if key_matches(key, corpus)? {
            matched_keys.push(key.pattern_text.clone());
        }
    }

    let matched_count = matched_keys.len();
    let total_count = enabled_keys.len();
    let matched = match entry.keyword_logic.as_str() {
        "and_any" => matched_count > 0,
        "and_all" => matched_count == total_count,
        "not_any" => matched_count == 0,
        "not_all" => matched_count < total_count,
        _ => matched_count > 0,
    };

    if !matched {
        return Ok(None);
    }

    let score = if total_count == 0 {
        0.0
    } else {
        matched_count as f32 / total_count as f32
    };

    Ok(Some((score.max(1.0 / total_count as f32), matched_keys)))
}

fn key_matches(key: &LorebookEntryKeyDetail, corpus: &str) -> Result<bool> {
    match key.match_type.as_str() {
        "regex" => {
            let regex = RegexBuilder::new(&key.pattern_text)
                .case_insensitive(!key.case_sensitive)
                .build()
                .map_err(|err| {
                    AppError::Validation(format!(
                        "invalid lorebook entry regex '{}': {err}",
                        key.pattern_text
                    ))
                })?;
            Ok(regex.is_match(corpus))
        }
        _ => {
            if key.case_sensitive {
                Ok(corpus.contains(&key.pattern_text))
            } else {
                Ok(corpus
                    .to_lowercase()
                    .contains(&key.pattern_text.to_lowercase()))
            }
        }
    }
}

fn parse_json(raw: &str, field: &'static str) -> Result<serde_json::Value> {
    serde_json::from_str(raw)
        .map_err(|err| AppError::Validation(format!("failed to parse {field} as json: {err}")))
}

fn ensure_prompt_content(content_type: &ContentType) -> Result<()> {
    match content_type {
        ContentType::Text | ContentType::Markdown | ContentType::Html | ContentType::Json => Ok(()),
        _ => Err(AppError::Validation(
            "lorebook entry content must be textual".to_string(),
        )),
    }
}
