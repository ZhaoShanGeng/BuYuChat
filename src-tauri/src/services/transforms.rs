use std::collections::HashSet;

use regex::RegexBuilder;
use sqlx::SqlitePool;

use crate::db::models::{TransformBindingRow, TransformPipelineRow, TransformStepRow};
use crate::db::repos::{
    agents, content as content_repo, conversations, presets, transforms as repo, workflows,
};
use crate::domain::content::{ContentType, ContentWriteInput, StoredContent};
use crate::domain::transforms::{
    ApplyTransformsInput, ApplyTransformsResult, CreateTransformPipelineInput, TransformBinding,
    TransformBindingInput, TransformPipeline, TransformStage, TransformStep, TransformStepInput,
    UpdateTransformPipelineInput,
};
use crate::services::content as content_service;
use crate::services::content_store::ContentStore;
use crate::support::error::{AppError, Result};

const DEFAULT_MAX_PIPELINE_DEPTH: usize = 8;

pub async fn list_transform_pipelines(
    db: &SqlitePool,
    store: &ContentStore,
) -> Result<Vec<TransformPipeline>> {
    let rows = repo::list_transform_pipelines(db).await?;
    let mut pipelines = Vec::with_capacity(rows.len());
    for row in rows {
        pipelines.push(map_transform_pipeline(db, store, row).await?);
    }
    Ok(pipelines)
}

pub async fn get_transform_pipeline(
    db: &SqlitePool,
    store: &ContentStore,
    id: &str,
) -> Result<TransformPipeline> {
    let row = repo::get_transform_pipeline(db, id).await?;
    map_transform_pipeline(db, store, row).await
}

pub async fn create_transform_pipeline(
    db: &SqlitePool,
    store: &ContentStore,
    input: &CreateTransformPipelineInput,
) -> Result<TransformPipeline> {
    validate_pipeline_kind(&input.pipeline_kind)?;
    ensure_optional_description_exists(db, input.description_content_id.as_deref()).await?;

    let row = repo::create_transform_pipeline(
        db,
        &repo::CreateTransformPipelineRecord {
            name: &input.name,
            pipeline_key: &input.pipeline_key,
            pipeline_kind: &input.pipeline_kind,
            description_content_id: input.description_content_id.as_deref(),
            enabled: input.enabled,
            sort_order: input.sort_order,
            config_json: &input.config_json.to_string(),
        },
    )
    .await?;

    map_transform_pipeline(db, store, row).await
}

pub async fn update_transform_pipeline(
    db: &SqlitePool,
    store: &ContentStore,
    id: &str,
    input: &UpdateTransformPipelineInput,
) -> Result<TransformPipeline> {
    validate_pipeline_kind(&input.pipeline_kind)?;
    ensure_optional_description_exists(db, input.description_content_id.as_deref()).await?;

    let row = repo::update_transform_pipeline(
        db,
        id,
        &repo::UpdateTransformPipelineRecord {
            name: &input.name,
            pipeline_kind: &input.pipeline_kind,
            description_content_id: input.description_content_id.as_deref(),
            enabled: input.enabled,
            sort_order: input.sort_order,
            config_json: &input.config_json.to_string(),
        },
    )
    .await?;

    map_transform_pipeline(db, store, row).await
}

pub async fn delete_transform_pipeline(db: &SqlitePool, id: &str) -> Result<()> {
    repo::delete_transform_pipeline(db, id).await
}

pub async fn list_transform_steps(
    db: &SqlitePool,
    pipeline_id: &str,
) -> Result<Vec<TransformStep>> {
    repo::list_transform_steps(db, pipeline_id)
        .await?
        .into_iter()
        .map(map_transform_step)
        .collect()
}

pub async fn replace_transform_steps(
    db: &SqlitePool,
    pipeline_id: &str,
    items: &[TransformStepInput],
) -> Result<Vec<TransformStep>> {
    let pipeline = repo::get_transform_pipeline(db, pipeline_id).await?;
    let _ = pipeline;
    validate_step_inputs(db, pipeline_id, items).await?;

    let owned = items
        .iter()
        .map(|item| OwnedTransformStep {
            step_order: item.step_order,
            step_type: item.step_type.clone(),
            pattern: item.pattern.clone(),
            replacement_template: item.replacement_template.clone(),
            regex_flags: item.regex_flags.clone(),
            max_replacements: item.max_replacements,
            stop_on_match: item.stop_on_match,
            child_pipeline_id: item.child_pipeline_id.clone(),
            config_json: item.config_json.to_string(),
        })
        .collect::<Vec<_>>();

    let records = owned
        .iter()
        .map(|item| repo::TransformStepRecord {
            step_order: item.step_order,
            step_type: &item.step_type,
            pattern: item.pattern.as_deref(),
            replacement_template: item.replacement_template.as_deref(),
            regex_flags: &item.regex_flags,
            max_replacements: item.max_replacements,
            stop_on_match: item.stop_on_match,
            child_pipeline_id: item.child_pipeline_id.as_deref(),
            config_json: &item.config_json,
        })
        .collect::<Vec<_>>();

    let mut tx = db.begin().await?;
    let rows = repo::replace_transform_steps(&mut tx, pipeline_id, &records).await?;
    tx.commit().await?;

    rows.into_iter().map(map_transform_step).collect()
}

pub async fn list_transform_bindings(db: &SqlitePool) -> Result<Vec<TransformBinding>> {
    repo::list_transform_bindings(db)
        .await?
        .into_iter()
        .map(map_transform_binding)
        .collect()
}

pub async fn create_transform_binding(
    db: &SqlitePool,
    input: &TransformBindingInput,
) -> Result<TransformBinding> {
    validate_binding_input(db, input).await?;
    let row = repo::create_transform_binding(
        db,
        &repo::CreateOrUpdateTransformBindingRecord {
            pipeline_id: &input.pipeline_id,
            conversation_id: input.conversation_id.as_deref(),
            agent_id: input.agent_id.as_deref(),
            preset_id: input.preset_id.as_deref(),
            workflow_def_node_id: input.workflow_def_node_id.as_deref(),
            apply_viewer: input.apply_viewer,
            apply_request: input.apply_request,
            apply_file: input.apply_file,
            enabled: input.enabled,
            sort_order: input.sort_order,
            config_json: &input.config_json.to_string(),
        },
    )
    .await?;

    map_transform_binding(row)
}

pub async fn update_transform_binding(
    db: &SqlitePool,
    id: &str,
    input: &TransformBindingInput,
) -> Result<TransformBinding> {
    let _ = repo::get_transform_binding(db, id).await?;
    validate_binding_input(db, input).await?;

    let row = repo::update_transform_binding(
        db,
        id,
        &repo::CreateOrUpdateTransformBindingRecord {
            pipeline_id: &input.pipeline_id,
            conversation_id: input.conversation_id.as_deref(),
            agent_id: input.agent_id.as_deref(),
            preset_id: input.preset_id.as_deref(),
            workflow_def_node_id: input.workflow_def_node_id.as_deref(),
            apply_viewer: input.apply_viewer,
            apply_request: input.apply_request,
            apply_file: input.apply_file,
            enabled: input.enabled,
            sort_order: input.sort_order,
            config_json: &input.config_json.to_string(),
        },
    )
    .await?;

    map_transform_binding(row)
}

pub async fn delete_transform_binding(db: &SqlitePool, id: &str) -> Result<()> {
    repo::delete_transform_binding(db, id).await
}

pub async fn apply_transforms(
    db: &SqlitePool,
    store: &ContentStore,
    input: &ApplyTransformsInput,
) -> Result<ApplyTransformsResult> {
    let bindings = repo::list_matching_transform_bindings(
        db,
        input.stage.as_str(),
        input.conversation_id.as_deref(),
        input.agent_id.as_deref(),
        input.preset_id.as_deref(),
        input.workflow_def_node_id.as_deref(),
    )
    .await?;

    if bindings.is_empty() {
        return Ok(ApplyTransformsResult {
            content: input.source_content.clone(),
            applied_pipeline_ids: Vec::new(),
            changed: false,
        });
    }

    let Some(mut current_text) = load_text_body_if_needed(db, store, &input.source_content).await?
    else {
        return Ok(ApplyTransformsResult {
            content: input.source_content.clone(),
            applied_pipeline_ids: Vec::new(),
            changed: false,
        });
    };

    let mut applied_pipeline_ids = Vec::new();
    let mut seen_pipeline_ids = HashSet::new();
    let mut changed = false;

    for binding in &bindings {
        let pipeline = repo::get_transform_pipeline(db, &binding.pipeline_id).await?;
        if !pipeline.enabled {
            continue;
        }

        let (next_text, pipeline_changed, traversed_pipeline_ids) =
            execute_pipeline(db, &pipeline, current_text, 0).await?;

        current_text = next_text;
        changed |= pipeline_changed;
        for pipeline_id in traversed_pipeline_ids {
            if seen_pipeline_ids.insert(pipeline_id.clone()) {
                applied_pipeline_ids.push(pipeline_id);
            }
        }
    }

    let content = match input.stage {
        TransformStage::Viewer | TransformStage::Request => {
            build_ephemeral_text_content(&input.source_content, current_text)
        }
        TransformStage::File => {
            if changed {
                content_service::create_content(
                    db,
                    store,
                    &ContentWriteInput {
                        content_type: input.source_content.content_type,
                        mime_type: input.source_content.mime_type.clone(),
                        text_content: Some(current_text),
                        source_file_path: None,
                        primary_storage_uri: None,
                        size_bytes_hint: None,
                        preview_text: None,
                        config_json: input.source_content.config_json.clone(),
                    },
                )
                .await?
            } else {
                input.source_content.clone()
            }
        }
    };

    Ok(ApplyTransformsResult {
        content,
        applied_pipeline_ids,
        changed,
    })
}

async fn execute_pipeline(
    db: &SqlitePool,
    pipeline: &TransformPipelineRow,
    input_text: String,
    depth: usize,
) -> Result<(String, bool, Vec<String>)> {
    let max_depth = pipeline
        .config_json
        .parse::<serde_json::Value>()
        .ok()
        .and_then(|json| json.get("max_depth").and_then(|value| value.as_u64()))
        .map(|value| value as usize)
        .unwrap_or(DEFAULT_MAX_PIPELINE_DEPTH);

    if depth >= max_depth {
        return Err(AppError::Validation(format!(
            "transform pipeline recursion depth exceeded for pipeline '{}'",
            pipeline.id
        )));
    }

    let steps = repo::list_transform_steps(db, &pipeline.id).await?;
    let mut current = input_text;
    let mut changed = false;
    let mut traversed = vec![pipeline.id.clone()];

    for step in steps {
        let outcome = execute_step(db, &step, current, depth, max_depth).await?;
        current = outcome.text;
        changed |= outcome.changed;
        for pipeline_id in outcome.traversed_pipeline_ids {
            traversed.push(pipeline_id);
        }
        if outcome.stop_pipeline {
            break;
        }
    }

    Ok((current, changed, traversed))
}

async fn execute_step(
    db: &SqlitePool,
    step: &TransformStepRow,
    input_text: String,
    depth: usize,
    max_depth: usize,
) -> Result<StepOutcome> {
    match step.step_type.as_str() {
        "regex_replace" => apply_regex_replace(step, input_text),
        "regex_remove" => apply_regex_remove(step, input_text),
        "regex_extract" => apply_regex_extract(step, input_text),
        "regex_match_gate" => apply_regex_match_gate(step, input_text),
        "pipeline_ref" => {
            let child_pipeline_id = step.child_pipeline_id.as_deref().ok_or_else(|| {
                AppError::Validation("pipeline_ref step requires child_pipeline_id".to_string())
            })?;
            if depth + 1 > max_depth {
                return Err(AppError::Validation(
                    "transform pipeline recursion depth exceeded".to_string(),
                ));
            }
            let child = repo::get_transform_pipeline(db, child_pipeline_id).await?;
            let (text, changed, traversed) =
                Box::pin(execute_pipeline(db, &child, input_text, depth + 1)).await?;
            Ok(StepOutcome {
                stop_pipeline: step.stop_on_match && changed,
                text,
                changed,
                traversed_pipeline_ids: traversed,
            })
        }
        other => Err(AppError::Validation(format!(
            "unsupported transform step_type '{other}'"
        ))),
    }
}

fn apply_regex_replace(step: &TransformStepRow, input_text: String) -> Result<StepOutcome> {
    let pattern = step
        .pattern
        .as_deref()
        .ok_or_else(|| AppError::Validation("regex_replace step requires pattern".to_string()))?;
    let replacement = step.replacement_template.as_deref().ok_or_else(|| {
        AppError::Validation("regex_replace step requires replacement_template".to_string())
    })?;
    let regex = build_regex(pattern, &step.regex_flags)?;
    let limit = normalize_max_replacements(step.max_replacements)?;
    let matched = regex.is_match(&input_text);
    let replaced = if let Some(limit) = limit {
        regex.replacen(&input_text, limit, replacement).to_string()
    } else {
        regex.replace_all(&input_text, replacement).to_string()
    };

    Ok(StepOutcome {
        stop_pipeline: step.stop_on_match && matched,
        changed: replaced != input_text,
        text: replaced,
        traversed_pipeline_ids: Vec::new(),
    })
}

fn apply_regex_remove(step: &TransformStepRow, input_text: String) -> Result<StepOutcome> {
    let pattern = step
        .pattern
        .as_deref()
        .ok_or_else(|| AppError::Validation("regex_remove step requires pattern".to_string()))?;
    let regex = build_regex(pattern, &step.regex_flags)?;
    let limit = normalize_max_replacements(step.max_replacements)?;
    let matched = regex.is_match(&input_text);
    let replaced = if let Some(limit) = limit {
        regex.replacen(&input_text, limit, "").to_string()
    } else {
        regex.replace_all(&input_text, "").to_string()
    };

    Ok(StepOutcome {
        stop_pipeline: step.stop_on_match && matched,
        changed: replaced != input_text,
        text: replaced,
        traversed_pipeline_ids: Vec::new(),
    })
}

fn apply_regex_extract(step: &TransformStepRow, input_text: String) -> Result<StepOutcome> {
    let pattern = step
        .pattern
        .as_deref()
        .ok_or_else(|| AppError::Validation("regex_extract step requires pattern".to_string()))?;
    let regex = build_regex(pattern, &step.regex_flags)?;
    let config_json: serde_json::Value = serde_json::from_str(&step.config_json)?;
    let join_with = config_json
        .get("join_with")
        .and_then(|value| value.as_str())
        .unwrap_or("\n");
    let group_index = config_json.get("group").and_then(|value| value.as_u64());
    let group_name = config_json
        .get("group_name")
        .and_then(|value| value.as_str());
    let limit = normalize_max_replacements(step.max_replacements)?;

    let mut values = Vec::new();
    for captures in regex.captures_iter(&input_text) {
        if let Some(limit) = limit {
            if values.len() >= limit {
                break;
            }
        }

        let extracted = if let Some(group_name) = group_name {
            captures
                .name(group_name)
                .map(|value| value.as_str().to_string())
        } else if let Some(group_index) = group_index {
            captures
                .get(group_index as usize)
                .map(|value| value.as_str().to_string())
        } else {
            captures.get(0).map(|value| value.as_str().to_string())
        };

        if let Some(extracted) = extracted {
            values.push(extracted);
        }
    }

    let matched = !values.is_empty();
    let extracted = values.join(join_with);

    Ok(StepOutcome {
        stop_pipeline: step.stop_on_match && matched,
        changed: extracted != input_text,
        text: extracted,
        traversed_pipeline_ids: Vec::new(),
    })
}

fn apply_regex_match_gate(step: &TransformStepRow, input_text: String) -> Result<StepOutcome> {
    let pattern = step.pattern.as_deref().ok_or_else(|| {
        AppError::Validation("regex_match_gate step requires pattern".to_string())
    })?;
    let regex = build_regex(pattern, &step.regex_flags)?;
    let matched = regex.is_match(&input_text);

    Ok(StepOutcome {
        stop_pipeline: !matched || (step.stop_on_match && matched),
        changed: false,
        text: input_text,
        traversed_pipeline_ids: Vec::new(),
    })
}

async fn validate_step_inputs(
    db: &SqlitePool,
    pipeline_id: &str,
    items: &[TransformStepInput],
) -> Result<()> {
    let mut seen_orders = HashSet::new();
    for item in items {
        if !seen_orders.insert(item.step_order) {
            return Err(AppError::Validation(format!(
                "duplicate transform step_order {}",
                item.step_order
            )));
        }

        validate_step_type(&item.step_type)?;

        match item.step_type.as_str() {
            "regex_replace" => {
                if item.pattern.as_deref().unwrap_or("").is_empty() {
                    return Err(AppError::Validation(
                        "regex_replace step requires pattern".to_string(),
                    ));
                }
                if item.replacement_template.is_none() {
                    return Err(AppError::Validation(
                        "regex_replace step requires replacement_template".to_string(),
                    ));
                }
            }
            "regex_remove" | "regex_extract" | "regex_match_gate" => {
                if item.pattern.as_deref().unwrap_or("").is_empty() {
                    return Err(AppError::Validation(format!(
                        "{} step requires pattern",
                        item.step_type
                    )));
                }
            }
            "pipeline_ref" => {
                let Some(child_pipeline_id) = item.child_pipeline_id.as_deref() else {
                    return Err(AppError::Validation(
                        "pipeline_ref step requires child_pipeline_id".to_string(),
                    ));
                };
                if child_pipeline_id == pipeline_id {
                    return Err(AppError::Validation(
                        "transform pipeline cannot directly reference itself".to_string(),
                    ));
                }
                let _ = repo::get_transform_pipeline(db, child_pipeline_id).await?;
            }
            _ => {}
        }

        if item.max_replacements.unwrap_or(1) < 0 {
            return Err(AppError::Validation(
                "max_replacements must be >= 0".to_string(),
            ));
        }

        if item.step_type.starts_with("regex_") && item.pattern.is_some() {
            let _ = build_regex(
                item.pattern.as_deref().unwrap_or_default(),
                &item.regex_flags,
            )?;
        }
    }

    Ok(())
}

async fn validate_binding_input(db: &SqlitePool, input: &TransformBindingInput) -> Result<()> {
    let _ = repo::get_transform_pipeline(db, &input.pipeline_id).await?;
    validate_binding_scope(input)?;
    validate_binding_stages(input)?;

    if let Some(conversation_id) = input.conversation_id.as_deref() {
        let _ = conversations::get_conversation(db, conversation_id).await?;
    }
    if let Some(agent_id) = input.agent_id.as_deref() {
        let _ = agents::get_agent(db, agent_id).await?;
    }
    if let Some(preset_id) = input.preset_id.as_deref() {
        let _ = presets::get_preset(db, preset_id).await?;
    }
    if let Some(workflow_def_node_id) = input.workflow_def_node_id.as_deref() {
        let _ = workflows::get_workflow_def_node(db, workflow_def_node_id).await?;
    }

    Ok(())
}

fn validate_binding_scope(input: &TransformBindingInput) -> Result<()> {
    let count = [
        input.conversation_id.is_some(),
        input.agent_id.is_some(),
        input.preset_id.is_some(),
        input.workflow_def_node_id.is_some(),
    ]
    .into_iter()
    .filter(|value| *value)
    .count();

    if count != 1 {
        return Err(AppError::Validation(
            "transform binding must target exactly one scope".to_string(),
        ));
    }

    Ok(())
}

fn validate_binding_stages(input: &TransformBindingInput) -> Result<()> {
    if !(input.apply_viewer || input.apply_request || input.apply_file) {
        return Err(AppError::Validation(
            "transform binding must enable at least one stage".to_string(),
        ));
    }

    Ok(())
}

fn validate_pipeline_kind(value: &str) -> Result<()> {
    match value {
        "regex" | "composite" => Ok(()),
        _ => Err(AppError::Validation(format!(
            "unsupported transform pipeline_kind '{value}'"
        ))),
    }
}

fn validate_step_type(value: &str) -> Result<()> {
    match value {
        "regex_replace" | "regex_extract" | "regex_remove" | "regex_match_gate"
        | "pipeline_ref" => Ok(()),
        _ => Err(AppError::Validation(format!(
            "unsupported transform step_type '{value}'"
        ))),
    }
}

async fn ensure_optional_description_exists(
    db: &SqlitePool,
    description_content_id: Option<&str>,
) -> Result<()> {
    let Some(description_content_id) = description_content_id else {
        return Ok(());
    };

    let _ = content_repo::get_content_object(db, description_content_id).await?;
    Ok(())
}

async fn map_transform_pipeline(
    db: &SqlitePool,
    store: &ContentStore,
    row: TransformPipelineRow,
) -> Result<TransformPipeline> {
    let description_content = match row.description_content_id.as_deref() {
        Some(content_id) => Some(content_service::get_content(db, store, content_id, false).await?),
        None => None,
    };

    Ok(TransformPipeline {
        id: row.id,
        name: row.name,
        pipeline_key: row.pipeline_key,
        pipeline_kind: row.pipeline_kind,
        description_content,
        enabled: row.enabled,
        sort_order: row.sort_order,
        config_json: serde_json::from_str(&row.config_json)?,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

fn map_transform_step(row: TransformStepRow) -> Result<TransformStep> {
    Ok(TransformStep {
        id: row.id,
        pipeline_id: row.pipeline_id,
        step_order: row.step_order,
        step_type: row.step_type,
        pattern: row.pattern,
        replacement_template: row.replacement_template,
        regex_flags: row.regex_flags,
        max_replacements: row.max_replacements,
        stop_on_match: row.stop_on_match,
        child_pipeline_id: row.child_pipeline_id,
        config_json: serde_json::from_str(&row.config_json)?,
    })
}

fn map_transform_binding(row: TransformBindingRow) -> Result<TransformBinding> {
    Ok(TransformBinding {
        id: row.id,
        pipeline_id: row.pipeline_id,
        conversation_id: row.conversation_id,
        agent_id: row.agent_id,
        preset_id: row.preset_id,
        workflow_def_node_id: row.workflow_def_node_id,
        apply_viewer: row.apply_viewer,
        apply_request: row.apply_request,
        apply_file: row.apply_file,
        enabled: row.enabled,
        sort_order: row.sort_order,
        config_json: serde_json::from_str(&row.config_json)?,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

async fn load_text_body_if_needed(
    db: &SqlitePool,
    store: &ContentStore,
    content: &StoredContent,
) -> Result<Option<String>> {
    if !is_textual_content_type(content.content_type) {
        return Ok(None);
    }

    if let Some(text) = &content.text_content {
        return Ok(Some(text.clone()));
    }

    let full = content_service::get_content(db, store, &content.content_id, true).await?;
    Ok(full.text_content)
}

fn is_textual_content_type(content_type: ContentType) -> bool {
    matches!(
        content_type,
        ContentType::Text | ContentType::Markdown | ContentType::Json | ContentType::Html
    )
}

fn build_ephemeral_text_content(source: &StoredContent, text: String) -> StoredContent {
    let mut content = source.clone();
    content.text_content = Some(text.clone());
    content.preview_text = Some(derive_preview_text(&text));
    content.size_bytes = text.len() as u64;
    content
}

fn derive_preview_text(text: &str) -> String {
    text.chars().take(1024).collect()
}

fn normalize_max_replacements(value: Option<i64>) -> Result<Option<usize>> {
    match value {
        Some(value) if value < 0 => Err(AppError::Validation(
            "max_replacements must be >= 0".to_string(),
        )),
        Some(0) => Ok(Some(0)),
        Some(value) => Ok(Some(value as usize)),
        None => Ok(None),
    }
}

fn build_regex(pattern: &str, flags: &str) -> Result<regex::Regex> {
    let mut builder = RegexBuilder::new(pattern);
    builder
        .case_insensitive(flags.contains('i'))
        .multi_line(flags.contains('m'))
        .dot_matches_new_line(flags.contains('s'))
        .ignore_whitespace(flags.contains('x'))
        .unicode(true);
    builder
        .build()
        .map_err(|err| AppError::Validation(format!("invalid regex '{pattern}': {err}")))
}

struct OwnedTransformStep {
    step_order: i64,
    step_type: String,
    pattern: Option<String>,
    replacement_template: Option<String>,
    regex_flags: String,
    max_replacements: Option<i64>,
    stop_on_match: bool,
    child_pipeline_id: Option<String>,
    config_json: String,
}

struct StepOutcome {
    stop_pipeline: bool,
    text: String,
    changed: bool,
    traversed_pipeline_ids: Vec<String>,
}
