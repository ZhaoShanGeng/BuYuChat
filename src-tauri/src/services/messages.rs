use sqlx::SqlitePool;

use crate::db::models::{MessageNodeRow, MessageVersionContentRefRow, MessageVersionRow};
use crate::db::repos::{conversations as conversations_repo, messages as repo};
use crate::domain::content::StoredContent;
use crate::domain::messages::{
    AddAttachmentInput, ContextPolicy, CreateMessageInput, EditMessageVersionInput,
    MessageContentRefView, MessageRole, MessageVersionView, ViewerPolicy,
};
use crate::services::content;
use crate::services::content_store::ContentStore;
use crate::support::error::{AppError, Result};
use crate::support::{order_keys, time};

pub async fn list_visible_messages(
    db: &SqlitePool,
    store: &ContentStore,
    conversation_id: &str,
) -> Result<Vec<MessageVersionView>> {
    let rows = repo::list_active_message_versions_for_conversation(db, conversation_id).await?;
    let version_ids = rows
        .iter()
        .map(|row| row.version.id.clone())
        .collect::<Vec<_>>();
    let refs_by_version = repo::list_content_refs_for_versions(db, &version_ids).await?;

    let mut views = Vec::with_capacity(rows.len());
    for row in rows {
        let refs = refs_by_version
            .get(&row.version.id)
            .cloned()
            .unwrap_or_default();
        views.push(build_message_view(db, store, row.node, row.version, refs, true).await?);
    }
    Ok(views)
}

pub async fn list_message_versions(
    db: &SqlitePool,
    store: &ContentStore,
    node_id: &str,
) -> Result<Vec<MessageVersionView>> {
    let node = repo::get_message_node(db, node_id).await?;
    let versions = repo::list_message_versions(db, node_id).await?;
    let version_ids = versions
        .iter()
        .map(|row| row.id.clone())
        .collect::<Vec<_>>();
    let refs_by_version = repo::list_content_refs_for_versions(db, &version_ids).await?;

    let mut views = Vec::with_capacity(versions.len());
    for version in versions {
        let refs = refs_by_version
            .get(&version.id)
            .cloned()
            .unwrap_or_default();
        views.push(build_message_view(db, store, node.clone(), version, refs, true).await?);
    }
    Ok(views)
}

pub async fn get_message_body(
    db: &SqlitePool,
    store: &ContentStore,
    version_id: &str,
) -> Result<StoredContent> {
    let version = repo::get_message_version(db, version_id).await?;
    content::get_content(db, store, &version.primary_content_id, true).await
}

pub async fn get_message_version_view(
    db: &SqlitePool,
    store: &ContentStore,
    version_id: &str,
) -> Result<MessageVersionView> {
    let version = repo::get_message_version(db, version_id).await?;
    let node = repo::get_message_node(db, &version.node_id).await?;
    let refs = repo::list_content_refs(db, version_id).await?;
    build_message_view(db, store, node, version, refs, true).await
}

pub async fn create_user_message(
    db: &SqlitePool,
    store: &ContentStore,
    input: &CreateMessageInput,
) -> Result<MessageVersionView> {
    create_message(db, store, input, MessageRole::User).await
}

pub async fn create_system_message(
    db: &SqlitePool,
    store: &ContentStore,
    input: &CreateMessageInput,
) -> Result<MessageVersionView> {
    create_message(db, store, input, MessageRole::System).await
}

pub async fn create_assistant_message(
    db: &SqlitePool,
    store: &ContentStore,
    input: &CreateMessageInput,
) -> Result<MessageVersionView> {
    create_message(db, store, input, MessageRole::Assistant).await
}

pub async fn edit_message_version(
    db: &SqlitePool,
    store: &ContentStore,
    input: &EditMessageVersionInput,
) -> Result<MessageVersionView> {
    let node = repo::get_message_node(db, &input.node_id).await?;
    let base_version = repo::get_message_version(db, &input.base_version_id).await?;
    if base_version.node_id != input.node_id {
        return Err(AppError::Validation(
            "base_version_id does not belong to node_id".to_string(),
        ));
    }

    let primary_content = content::create_content(db, store, &input.primary_content).await?;
    let next_version_index = repo::list_message_versions(db, &input.node_id)
        .await?
        .into_iter()
        .map(|row| row.version_index)
        .max()
        .unwrap_or(0)
        + 1;

    let now = time::now_ms();
    let mut tx = db.begin().await?;
    repo::set_active_message_version(&mut tx, &input.node_id, &input.base_version_id).await?;
    sqlx::query("UPDATE message_versions SET is_active = 0 WHERE node_id = ?")
        .bind(&input.node_id)
        .execute(tx.as_mut())
        .await?;
    let version = repo::create_message_version(
        &mut tx,
        &repo::CreateMessageVersionRecord {
            node_id: &input.node_id,
            version_index: next_version_index,
            is_active: true,
            primary_content_id: &primary_content.content_id,
            context_policy: input.context_policy.as_str(),
            viewer_policy: input.viewer_policy.as_str(),
            api_channel_id: base_version.api_channel_id.as_deref(),
            api_channel_model_id: base_version.api_channel_model_id.as_deref(),
            generation_run_id: None,
            prompt_tokens: None,
            completion_tokens: None,
            total_tokens: None,
            finish_reason: None,
            config_json: &input.config_json.to_string(),
            created_at: now,
        },
    )
    .await?;
    sqlx::query("UPDATE message_nodes SET updated_at = ? WHERE id = ?")
        .bind(now)
        .bind(&input.node_id)
        .execute(tx.as_mut())
        .await?;
    tx.commit().await?;
    conversations_repo::touch_conversation(db, &node.conversation_id).await?;

    build_message_view(db, store, node, version, Vec::new(), true).await
}

pub async fn switch_message_version(
    db: &SqlitePool,
    store: &ContentStore,
    node_id: &str,
    version_id: &str,
) -> Result<MessageVersionView> {
    let node = repo::get_message_node(db, node_id).await?;
    let mut tx = db.begin().await?;
    repo::set_active_message_version(&mut tx, node_id, version_id).await?;
    sqlx::query("UPDATE message_nodes SET updated_at = ? WHERE id = ?")
        .bind(time::now_ms())
        .bind(node_id)
        .execute(tx.as_mut())
        .await?;
    tx.commit().await?;
    conversations_repo::touch_conversation(db, &node.conversation_id).await?;

    let version = repo::get_message_version(db, version_id).await?;
    let refs = repo::list_content_refs(db, version_id).await?;
    build_message_view(db, store, node, version, refs, true).await
}

pub async fn delete_message_version(
    db: &SqlitePool,
    node_id: &str,
    version_id: &str,
) -> Result<()> {
    let node = repo::get_message_node(db, node_id).await?;
    let versions = repo::list_message_versions(db, node_id).await?;
    let target = versions
        .iter()
        .find(|row| row.id == version_id)
        .cloned()
        .ok_or_else(|| AppError::NotFound {
            entity: "message_version",
            id: version_id.to_string(),
        })?;

    let mut tx = db.begin().await?;
    if versions.len() == 1 {
        repo::delete_message_node(&mut tx, node_id).await?;
    } else {
        if target.is_active {
            let replacement = versions
                .iter()
                .filter(|row| row.id != version_id)
                .max_by_key(|row| row.version_index)
                .ok_or_else(|| {
                    AppError::Validation("failed to select replacement message version".to_string())
                })?;
            repo::set_active_message_version(&mut tx, node_id, &replacement.id).await?;
        }
        repo::delete_message_version(&mut tx, version_id).await?;
        sqlx::query("UPDATE message_nodes SET updated_at = ? WHERE id = ?")
            .bind(time::now_ms())
            .bind(node_id)
            .execute(tx.as_mut())
            .await?;
    }
    tx.commit().await?;
    conversations_repo::touch_conversation(db, &node.conversation_id).await?;
    Ok(())
}

pub async fn delete_message_node(db: &SqlitePool, node_id: &str) -> Result<()> {
    let node = repo::get_message_node(db, node_id).await?;
    let mut tx = db.begin().await?;
    repo::delete_message_node(&mut tx, node_id).await?;
    tx.commit().await?;
    conversations_repo::touch_conversation(db, &node.conversation_id).await?;
    Ok(())
}

pub async fn append_attachment(
    db: &SqlitePool,
    store: &ContentStore,
    input: &AddAttachmentInput,
) -> Result<MessageContentRefView> {
    let _ = repo::get_message_version(db, &input.message_version_id).await?;
    let stored = content::create_content(db, store, &input.content).await?;
    let mut tx = db.begin().await?;
    let row = repo::attach_content_ref(
        &mut tx,
        &repo::AttachContentRefRecord {
            message_version_id: &input.message_version_id,
            content_id: &stored.content_id,
            plugin_id: input.plugin_id.as_deref(),
            ref_role: &input.ref_role,
            sort_order: input.sort_order,
            config_json: &input.config_json.to_string(),
        },
    )
    .await?;
    tx.commit().await?;

    Ok(MessageContentRefView {
        ref_id: row.id,
        ref_role: row.ref_role,
        plugin_id: row.plugin_id,
        sort_order: row.sort_order,
        content: stored,
        config_json: parse_json(&row.config_json, "message_version_content_refs.config_json")?,
    })
}

async fn create_message(
    db: &SqlitePool,
    store: &ContentStore,
    input: &CreateMessageInput,
    expected_role: MessageRole,
) -> Result<MessageVersionView> {
    if input.role.as_str() != expected_role.as_str() {
        return Err(AppError::Validation(format!(
            "message role mismatch: expected {}, got {}",
            expected_role.as_str(),
            input.role.as_str()
        )));
    }

    let _ = conversations_repo::get_conversation(db, &input.conversation_id).await?;
    let participant =
        conversations_repo::get_conversation_participant(db, &input.author_participant_id).await?;
    if participant.conversation_id != input.conversation_id {
        return Err(AppError::Validation(
            "author_participant_id does not belong to conversation_id".to_string(),
        ));
    }

    if let Some(reply_to_node_id) = &input.reply_to_node_id {
        let reply_to = repo::get_message_node(db, reply_to_node_id).await?;
        if reply_to.conversation_id != input.conversation_id {
            return Err(AppError::Validation(
                "reply_to_node_id does not belong to conversation_id".to_string(),
            ));
        }
    }

    let primary_content = content::create_content(db, store, &input.primary_content).await?;
    let order_key = resolve_order_key(
        db,
        &input.conversation_id,
        input.order_after_node_id.as_deref(),
    )
    .await?;
    let now = time::now_ms();

    let mut tx = db.begin().await?;
    let node = repo::append_message_node(
        &mut tx,
        &repo::AppendMessageNodeRecord {
            conversation_id: &input.conversation_id,
            author_participant_id: &input.author_participant_id,
            role: input.role.as_str(),
            reply_to_node_id: input.reply_to_node_id.as_deref(),
            order_key: &order_key,
            created_at: now,
            updated_at: now,
        },
    )
    .await?;
    let version = repo::create_message_version(
        &mut tx,
        &repo::CreateMessageVersionRecord {
            node_id: &node.id,
            version_index: 1,
            is_active: true,
            primary_content_id: &primary_content.content_id,
            context_policy: input.context_policy.as_str(),
            viewer_policy: input.viewer_policy.as_str(),
            api_channel_id: None,
            api_channel_model_id: None,
            generation_run_id: None,
            prompt_tokens: None,
            completion_tokens: None,
            total_tokens: None,
            finish_reason: None,
            config_json: &input.config_json.to_string(),
            created_at: now,
        },
    )
    .await?;
    tx.commit().await?;
    conversations_repo::touch_conversation(db, &input.conversation_id).await?;

    build_message_view(db, store, node, version, Vec::new(), true).await
}

async fn resolve_order_key(
    db: &SqlitePool,
    conversation_id: &str,
    order_after_node_id: Option<&str>,
) -> Result<String> {
    let nodes = repo::list_message_nodes(db, conversation_id).await?;
    if nodes.is_empty() {
        return Ok(order_keys::initial_key());
    }

    match order_after_node_id {
        None => {
            let last = nodes.last().ok_or_else(|| {
                AppError::Validation("failed to locate last message node".to_string())
            })?;
            order_keys::append_after(Some(&last.order_key))
        }
        Some(after_node_id) => {
            let idx = nodes
                .iter()
                .position(|row| row.id == after_node_id)
                .ok_or_else(|| AppError::NotFound {
                    entity: "message_node",
                    id: after_node_id.to_string(),
                })?;
            let after = &nodes[idx];
            let next = nodes.get(idx + 1);
            if let Some(next_node) = next {
                if let Some(key) = order_keys::between(&after.order_key, &next_node.order_key)? {
                    return Ok(key);
                }

                let mut tx = db.begin().await?;
                let mut updates = Vec::with_capacity(nodes.len());
                for (position, node) in nodes.iter().enumerate() {
                    let new_key =
                        order_keys::format_key((position as u128 + 1) * 1_000_000_000u128);
                    updates.push((node.id.clone(), new_key));
                }
                repo::update_message_node_order_keys(&mut tx, &updates).await?;
                tx.commit().await?;

                let after_key = updates
                    .iter()
                    .find(|(id, _)| id == after_node_id)
                    .map(|(_, key)| key.clone())
                    .ok_or_else(|| {
                        AppError::Validation("failed to refresh after-node order key".to_string())
                    })?;
                let next_key = updates
                    .iter()
                    .find(|(id, _)| id == &next_node.id)
                    .map(|(_, key)| key.clone())
                    .ok_or_else(|| {
                        AppError::Validation("failed to refresh next-node order key".to_string())
                    })?;
                order_keys::between(&after_key, &next_key)?.ok_or_else(|| {
                    AppError::Validation(
                        "failed to allocate message order key after reindex".to_string(),
                    )
                })
            } else {
                order_keys::append_after(Some(&after.order_key))
            }
        }
    }
}

async fn build_message_view(
    db: &SqlitePool,
    store: &ContentStore,
    node: MessageNodeRow,
    version: MessageVersionRow,
    refs: Vec<MessageVersionContentRefRow>,
    include_primary_body: bool,
) -> Result<MessageVersionView> {
    let primary_content =
        content::get_content(db, store, &version.primary_content_id, include_primary_body).await?;
    let mut content_refs = Vec::with_capacity(refs.len());
    for row in refs {
        content_refs.push(MessageContentRefView {
            ref_id: row.id,
            ref_role: row.ref_role,
            plugin_id: row.plugin_id,
            sort_order: row.sort_order,
            content: content::get_content(db, store, &row.content_id, false).await?,
            config_json: parse_json(&row.config_json, "message_version_content_refs.config_json")?,
        });
    }

    Ok(MessageVersionView {
        node_id: node.id,
        version_id: version.id,
        conversation_id: node.conversation_id,
        author_participant_id: node.author_participant_id,
        role: MessageRole::parse(&node.role)?,
        reply_to_node_id: node.reply_to_node_id,
        order_key: node.order_key,
        version_index: version.version_index,
        is_active: version.is_active,
        primary_content,
        content_refs,
        context_policy: ContextPolicy::parse(&version.context_policy)?,
        viewer_policy: ViewerPolicy::parse(&version.viewer_policy)?,
        api_channel_id: version.api_channel_id,
        api_channel_model_id: version.api_channel_model_id,
        prompt_tokens: version.prompt_tokens,
        completion_tokens: version.completion_tokens,
        total_tokens: version.total_tokens,
        finish_reason: version.finish_reason,
        generation_run_id: version.generation_run_id,
        created_at: version.created_at,
    })
}

fn parse_json(raw: &str, field: &'static str) -> Result<serde_json::Value> {
    serde_json::from_str(raw)
        .map_err(|err| AppError::Validation(format!("failed to parse {field} as json: {err}")))
}
