use std::collections::HashMap;

use sqlx::{sqlite::SqliteRow, QueryBuilder, Row, Sqlite, SqlitePool, Transaction};

use crate::db::models::{
    GenerationRunContextItemRow, GenerationRunRow, MessageNodeRow, MessageVersionContentRefRow,
    MessageVersionRow,
};
use crate::support::error::{AppError, Result};
use crate::support::{ids, time};

pub struct AppendMessageNodeRecord<'a> {
    pub conversation_id: &'a str,
    pub author_participant_id: &'a str,
    pub role: &'a str,
    pub reply_to_node_id: Option<&'a str>,
    pub order_key: &'a str,
    pub created_at: i64,
    pub updated_at: i64,
}

pub struct CreateMessageVersionRecord<'a> {
    pub node_id: &'a str,
    pub version_index: i64,
    pub is_active: bool,
    pub primary_content_id: &'a str,
    pub context_policy: &'a str,
    pub viewer_policy: &'a str,
    pub api_channel_id: Option<&'a str>,
    pub api_channel_model_id: Option<&'a str>,
    pub generation_run_id: Option<&'a str>,
    pub prompt_tokens: Option<i64>,
    pub completion_tokens: Option<i64>,
    pub total_tokens: Option<i64>,
    pub finish_reason: Option<&'a str>,
    pub config_json: &'a str,
    pub created_at: i64,
}

pub struct AttachContentRefRecord<'a> {
    pub message_version_id: &'a str,
    pub content_id: &'a str,
    pub plugin_id: Option<&'a str>,
    pub ref_role: &'a str,
    pub sort_order: i64,
    pub config_json: &'a str,
}

pub struct CreateGenerationRunRecord<'a> {
    pub conversation_id: &'a str,
    pub trigger_node_id: Option<&'a str>,
    pub trigger_message_version_id: Option<&'a str>,
    pub responder_participant_id: Option<&'a str>,
    pub api_channel_id: Option<&'a str>,
    pub api_channel_model_id: Option<&'a str>,
    pub preset_id: Option<&'a str>,
    pub preset_source_scope: Option<&'a str>,
    pub lorebook_id: Option<&'a str>,
    pub lorebook_source_scope: Option<&'a str>,
    pub user_profile_id: Option<&'a str>,
    pub user_profile_source_scope: Option<&'a str>,
    pub api_channel_source_scope: Option<&'a str>,
    pub api_channel_model_source_scope: Option<&'a str>,
    pub run_type: &'a str,
    pub request_parameters_json: &'a str,
    pub request_payload_content_id: Option<&'a str>,
}

pub struct FinishGenerationRunSuccessRecord<'a> {
    pub response_payload_content_id: Option<&'a str>,
}

pub struct FinishGenerationRunFailureRecord<'a> {
    pub error_text: &'a str,
    pub response_payload_content_id: Option<&'a str>,
}

pub struct GenerationRunContextItemRecord<'a> {
    pub sequence_no: i64,
    pub send_role: &'a str,
    pub rendered_content_id: &'a str,
    pub source_kind: &'a str,
    pub source_message_node_id: Option<&'a str>,
    pub source_message_version_id: Option<&'a str>,
    pub source_summary_version_id: Option<&'a str>,
    pub source_preset_entry_id: Option<&'a str>,
    pub source_lorebook_entry_id: Option<&'a str>,
    pub source_user_profile_id: Option<&'a str>,
    pub source_agent_id: Option<&'a str>,
    pub source_agent_greeting_id: Option<&'a str>,
    pub source_tool_invocation_id: Option<&'a str>,
    pub source_rag_ref_id: Option<&'a str>,
    pub source_mcp_event_id: Option<&'a str>,
    pub source_plugin_id: Option<&'a str>,
    pub included_in_request: bool,
    pub config_json: &'a str,
}

#[derive(Debug, Clone)]
pub struct ActiveMessageVersionRow {
    pub node: MessageNodeRow,
    pub version: MessageVersionRow,
}

pub async fn append_message_node(
    tx: &mut Transaction<'_, Sqlite>,
    input: &AppendMessageNodeRecord<'_>,
) -> Result<MessageNodeRow> {
    let id = ids::new_id();

    sqlx::query(
        r#"
        INSERT INTO message_nodes (
            id, conversation_id, author_participant_id, role, reply_to_node_id,
            order_key, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(input.conversation_id)
    .bind(input.author_participant_id)
    .bind(input.role)
    .bind(input.reply_to_node_id)
    .bind(input.order_key)
    .bind(input.created_at)
    .bind(input.updated_at)
    .execute(tx.as_mut())
    .await?;

    get_message_node_with_executor(tx.as_mut(), &id).await
}

pub async fn create_message_version(
    tx: &mut Transaction<'_, Sqlite>,
    input: &CreateMessageVersionRecord<'_>,
) -> Result<MessageVersionRow> {
    let id = ids::new_id();

    sqlx::query(
        r#"
        INSERT INTO message_versions (
            id, node_id, version_index, is_active, primary_content_id, context_policy,
            viewer_policy, api_channel_id, api_channel_model_id, generation_run_id,
            prompt_tokens, completion_tokens, total_tokens, finish_reason, config_json, created_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(input.node_id)
    .bind(input.version_index)
    .bind(input.is_active)
    .bind(input.primary_content_id)
    .bind(input.context_policy)
    .bind(input.viewer_policy)
    .bind(input.api_channel_id)
    .bind(input.api_channel_model_id)
    .bind(input.generation_run_id)
    .bind(input.prompt_tokens)
    .bind(input.completion_tokens)
    .bind(input.total_tokens)
    .bind(input.finish_reason)
    .bind(input.config_json)
    .bind(input.created_at)
    .execute(tx.as_mut())
    .await?;

    get_message_version_with_executor(tx.as_mut(), &id).await
}

pub async fn get_message_node(db: &SqlitePool, id: &str) -> Result<MessageNodeRow> {
    get_message_node_with_executor(db, id).await
}

pub async fn list_message_nodes(
    db: &SqlitePool,
    conversation_id: &str,
) -> Result<Vec<MessageNodeRow>> {
    sqlx::query_as::<_, MessageNodeRow>(
        r#"
        SELECT *
        FROM message_nodes
        WHERE conversation_id = ?
        ORDER BY order_key ASC, created_at ASC
        "#,
    )
    .bind(conversation_id)
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

pub async fn list_message_versions(
    db: &SqlitePool,
    node_id: &str,
) -> Result<Vec<MessageVersionRow>> {
    sqlx::query_as::<_, MessageVersionRow>(
        r#"
        SELECT *
        FROM message_versions
        WHERE node_id = ?
        ORDER BY version_index DESC
        "#,
    )
    .bind(node_id)
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

pub async fn get_message_version(db: &SqlitePool, id: &str) -> Result<MessageVersionRow> {
    get_message_version_with_executor(db, id).await
}

pub async fn get_active_message_version(
    db: &SqlitePool,
    node_id: &str,
) -> Result<MessageVersionRow> {
    sqlx::query_as::<_, MessageVersionRow>(
        r#"
        SELECT *
        FROM message_versions
        WHERE node_id = ? AND is_active = 1
        LIMIT 1
        "#,
    )
    .bind(node_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::NotFound {
        entity: "active_message_version",
        id: node_id.to_string(),
    })
}

pub async fn set_active_message_version(
    tx: &mut Transaction<'_, Sqlite>,
    node_id: &str,
    version_id: &str,
) -> Result<()> {
    let belongs = sqlx::query_scalar::<_, String>(
        "SELECT id FROM message_versions WHERE id = ? AND node_id = ? LIMIT 1",
    )
    .bind(version_id)
    .bind(node_id)
    .fetch_optional(tx.as_mut())
    .await?;

    if belongs.is_none() {
        return Err(AppError::NotFound {
            entity: "message_version",
            id: version_id.to_string(),
        });
    }

    sqlx::query("UPDATE message_versions SET is_active = 0 WHERE node_id = ?")
        .bind(node_id)
        .execute(tx.as_mut())
        .await?;

    sqlx::query("UPDATE message_versions SET is_active = 1 WHERE id = ?")
        .bind(version_id)
        .execute(tx.as_mut())
        .await?;

    Ok(())
}

pub async fn update_message_node_order_keys(
    tx: &mut Transaction<'_, Sqlite>,
    updates: &[(String, String)],
) -> Result<()> {
    for (id, order_key) in updates {
        sqlx::query("UPDATE message_nodes SET order_key = ?, updated_at = ? WHERE id = ?")
            .bind(order_key)
            .bind(time::now_ms())
            .bind(id)
            .execute(tx.as_mut())
            .await?;
    }

    Ok(())
}

pub async fn attach_content_ref(
    tx: &mut Transaction<'_, Sqlite>,
    input: &AttachContentRefRecord<'_>,
) -> Result<MessageVersionContentRefRow> {
    let id = ids::new_id();

    sqlx::query(
        r#"
        INSERT INTO message_version_content_refs (
            id, message_version_id, content_id, plugin_id, ref_role, sort_order, config_json
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(input.message_version_id)
    .bind(input.content_id)
    .bind(input.plugin_id)
    .bind(input.ref_role)
    .bind(input.sort_order)
    .bind(input.config_json)
    .execute(tx.as_mut())
    .await?;

    get_content_ref_with_executor(tx.as_mut(), &id).await
}

pub async fn update_message_version_generation_metadata(
    tx: &mut Transaction<'_, Sqlite>,
    version_id: &str,
    api_channel_id: Option<&str>,
    api_channel_model_id: Option<&str>,
    generation_run_id: Option<&str>,
    prompt_tokens: Option<i64>,
    completion_tokens: Option<i64>,
    total_tokens: Option<i64>,
    finish_reason: Option<&str>,
) -> Result<()> {
    let affected = sqlx::query(
        r#"
        UPDATE message_versions
        SET api_channel_id = ?,
            api_channel_model_id = ?,
            generation_run_id = ?,
            prompt_tokens = ?,
            completion_tokens = ?,
            total_tokens = ?,
            finish_reason = ?
        WHERE id = ?
        "#,
    )
    .bind(api_channel_id)
    .bind(api_channel_model_id)
    .bind(generation_run_id)
    .bind(prompt_tokens)
    .bind(completion_tokens)
    .bind(total_tokens)
    .bind(finish_reason)
    .bind(version_id)
    .execute(tx.as_mut())
    .await?
    .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "message_version",
            id: version_id.to_string(),
        });
    }

    Ok(())
}

pub async fn list_content_refs(
    db: &SqlitePool,
    message_version_id: &str,
) -> Result<Vec<MessageVersionContentRefRow>> {
    sqlx::query_as::<_, MessageVersionContentRefRow>(
        r#"
        SELECT *
        FROM message_version_content_refs
        WHERE message_version_id = ?
        ORDER BY sort_order ASC, id ASC
        "#,
    )
    .bind(message_version_id)
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

pub async fn list_content_refs_for_versions(
    db: &SqlitePool,
    version_ids: &[String],
) -> Result<HashMap<String, Vec<MessageVersionContentRefRow>>> {
    if version_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let mut builder = QueryBuilder::<Sqlite>::new(
        "SELECT * FROM message_version_content_refs WHERE message_version_id IN (",
    );
    let mut separated = builder.separated(", ");
    for version_id in version_ids {
        separated.push_bind(version_id);
    }
    separated.push_unseparated(") ORDER BY message_version_id ASC, sort_order ASC, id ASC");

    let rows = builder
        .build_query_as::<MessageVersionContentRefRow>()
        .fetch_all(db)
        .await?;

    let mut grouped = HashMap::<String, Vec<MessageVersionContentRefRow>>::new();
    for row in rows {
        grouped
            .entry(row.message_version_id.clone())
            .or_default()
            .push(row);
    }
    Ok(grouped)
}

pub async fn create_generation_run(
    db: &SqlitePool,
    input: &CreateGenerationRunRecord<'_>,
) -> Result<GenerationRunRow> {
    let id = ids::new_id();
    let now = time::now_ms();

    sqlx::query(
        r#"
        INSERT INTO generation_runs (
            id, conversation_id, trigger_node_id, trigger_message_version_id, responder_participant_id,
            api_channel_id, api_channel_model_id, preset_id, preset_source_scope, lorebook_id,
            lorebook_source_scope, user_profile_id, user_profile_source_scope, api_channel_source_scope,
            api_channel_model_source_scope, run_type, status, request_parameters_json,
            request_payload_content_id, created_at, started_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'running', ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(input.conversation_id)
    .bind(input.trigger_node_id)
    .bind(input.trigger_message_version_id)
    .bind(input.responder_participant_id)
    .bind(input.api_channel_id)
    .bind(input.api_channel_model_id)
    .bind(input.preset_id)
    .bind(input.preset_source_scope)
    .bind(input.lorebook_id)
    .bind(input.lorebook_source_scope)
    .bind(input.user_profile_id)
    .bind(input.user_profile_source_scope)
    .bind(input.api_channel_source_scope)
    .bind(input.api_channel_model_source_scope)
    .bind(input.run_type)
    .bind(input.request_parameters_json)
    .bind(input.request_payload_content_id)
    .bind(now)
    .bind(now)
    .execute(db)
    .await?;

    get_generation_run(db, &id).await
}

pub async fn get_generation_run(db: &SqlitePool, id: &str) -> Result<GenerationRunRow> {
    sqlx::query_as::<_, GenerationRunRow>("SELECT * FROM generation_runs WHERE id = ? LIMIT 1")
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::NotFound {
            entity: "generation_run",
            id: id.to_string(),
        })
}

pub async fn finish_generation_run_success(
    db: &SqlitePool,
    id: &str,
    input: &FinishGenerationRunSuccessRecord<'_>,
) -> Result<GenerationRunRow> {
    let affected = sqlx::query(
        r#"
        UPDATE generation_runs
        SET status = 'succeeded',
            response_payload_content_id = ?,
            finished_at = ?
        WHERE id = ?
        "#,
    )
    .bind(input.response_payload_content_id)
    .bind(time::now_ms())
    .bind(id)
    .execute(db)
    .await?
    .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "generation_run",
            id: id.to_string(),
        });
    }

    get_generation_run(db, id).await
}

pub async fn finish_generation_run_failure(
    db: &SqlitePool,
    id: &str,
    input: &FinishGenerationRunFailureRecord<'_>,
) -> Result<GenerationRunRow> {
    let affected = sqlx::query(
        r#"
        UPDATE generation_runs
        SET status = 'failed',
            error_text = ?,
            response_payload_content_id = ?,
            finished_at = ?
        WHERE id = ?
        "#,
    )
    .bind(input.error_text)
    .bind(input.response_payload_content_id)
    .bind(time::now_ms())
    .bind(id)
    .execute(db)
    .await?
    .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "generation_run",
            id: id.to_string(),
        });
    }

    get_generation_run(db, id).await
}

pub async fn replace_generation_run_context_items(
    tx: &mut Transaction<'_, Sqlite>,
    generation_run_id: &str,
    items: &[GenerationRunContextItemRecord<'_>],
) -> Result<()> {
    sqlx::query("DELETE FROM generation_run_context_items WHERE generation_run_id = ?")
        .bind(generation_run_id)
        .execute(tx.as_mut())
        .await?;

    for item in items {
        sqlx::query(
            r#"
            INSERT INTO generation_run_context_items (
                id, generation_run_id, sequence_no, send_role, rendered_content_id, source_kind,
                source_message_node_id, source_message_version_id, source_summary_version_id,
                source_preset_entry_id, source_lorebook_entry_id, source_user_profile_id,
                source_agent_id, source_agent_greeting_id, source_tool_invocation_id,
                source_rag_ref_id, source_mcp_event_id, source_plugin_id,
                included_in_request, config_json
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(ids::new_id())
        .bind(generation_run_id)
        .bind(item.sequence_no)
        .bind(item.send_role)
        .bind(item.rendered_content_id)
        .bind(item.source_kind)
        .bind(item.source_message_node_id)
        .bind(item.source_message_version_id)
        .bind(item.source_summary_version_id)
        .bind(item.source_preset_entry_id)
        .bind(item.source_lorebook_entry_id)
        .bind(item.source_user_profile_id)
        .bind(item.source_agent_id)
        .bind(item.source_agent_greeting_id)
        .bind(item.source_tool_invocation_id)
        .bind(item.source_rag_ref_id)
        .bind(item.source_mcp_event_id)
        .bind(item.source_plugin_id)
        .bind(item.included_in_request)
        .bind(item.config_json)
        .execute(tx.as_mut())
        .await?;
    }

    Ok(())
}

pub async fn list_generation_run_context_items(
    db: &SqlitePool,
    generation_run_id: &str,
) -> Result<Vec<GenerationRunContextItemRow>> {
    sqlx::query_as::<_, GenerationRunContextItemRow>(
        r#"
        SELECT *
        FROM generation_run_context_items
        WHERE generation_run_id = ?
        ORDER BY sequence_no ASC
        "#,
    )
    .bind(generation_run_id)
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

pub async fn delete_message_version(
    tx: &mut Transaction<'_, Sqlite>,
    version_id: &str,
) -> Result<()> {
    sqlx::query("DELETE FROM message_version_content_refs WHERE message_version_id = ?")
        .bind(version_id)
        .execute(tx.as_mut())
        .await?;

    let affected = sqlx::query("DELETE FROM message_versions WHERE id = ?")
        .bind(version_id)
        .execute(tx.as_mut())
        .await?
        .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "message_version",
            id: version_id.to_string(),
        });
    }

    Ok(())
}

pub async fn delete_message_node(tx: &mut Transaction<'_, Sqlite>, node_id: &str) -> Result<()> {
    let version_ids =
        sqlx::query_scalar::<_, String>("SELECT id FROM message_versions WHERE node_id = ?")
            .bind(node_id)
            .fetch_all(tx.as_mut())
            .await?;

    for version_id in version_ids {
        sqlx::query("DELETE FROM message_version_content_refs WHERE message_version_id = ?")
            .bind(&version_id)
            .execute(tx.as_mut())
            .await?;
        sqlx::query("DELETE FROM message_versions WHERE id = ?")
            .bind(&version_id)
            .execute(tx.as_mut())
            .await?;
    }

    let affected = sqlx::query("DELETE FROM message_nodes WHERE id = ?")
        .bind(node_id)
        .execute(tx.as_mut())
        .await?
        .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "message_node",
            id: node_id.to_string(),
        });
    }

    Ok(())
}

pub async fn list_active_message_versions_for_conversation(
    db: &SqlitePool,
    conversation_id: &str,
) -> Result<Vec<ActiveMessageVersionRow>> {
    let rows = sqlx::query(
        r#"
        SELECT
            n.id AS node_id,
            n.conversation_id,
            n.author_participant_id,
            n.role,
            n.reply_to_node_id,
            n.order_key,
            n.created_at AS node_created_at,
            n.updated_at AS node_updated_at,
            v.id AS version_id,
            v.node_id AS version_node_id,
            v.version_index,
            v.is_active,
            v.primary_content_id,
            v.context_policy,
            v.viewer_policy,
            v.api_channel_id,
            v.api_channel_model_id,
            v.generation_run_id,
            v.prompt_tokens,
            v.completion_tokens,
            v.total_tokens,
            v.finish_reason,
            v.config_json,
            v.created_at AS version_created_at
        FROM message_nodes n
        INNER JOIN message_versions v ON v.node_id = n.id AND v.is_active = 1
        WHERE n.conversation_id = ?
          AND v.viewer_policy != 'hidden'
        ORDER BY n.order_key ASC, n.created_at ASC
        "#,
    )
    .bind(conversation_id)
    .fetch_all(db)
    .await?;

    rows.into_iter().map(map_active_message_row).collect()
}

async fn get_message_node_with_executor<'e, E>(executor: E, id: &str) -> Result<MessageNodeRow>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    sqlx::query_as::<_, MessageNodeRow>("SELECT * FROM message_nodes WHERE id = ? LIMIT 1")
        .bind(id)
        .fetch_optional(executor)
        .await?
        .ok_or_else(|| AppError::NotFound {
            entity: "message_node",
            id: id.to_string(),
        })
}

async fn get_message_version_with_executor<'e, E>(
    executor: E,
    id: &str,
) -> Result<MessageVersionRow>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    sqlx::query_as::<_, MessageVersionRow>("SELECT * FROM message_versions WHERE id = ? LIMIT 1")
        .bind(id)
        .fetch_optional(executor)
        .await?
        .ok_or_else(|| AppError::NotFound {
            entity: "message_version",
            id: id.to_string(),
        })
}

async fn get_content_ref_with_executor<'e, E>(
    executor: E,
    id: &str,
) -> Result<MessageVersionContentRefRow>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    sqlx::query_as::<_, MessageVersionContentRefRow>(
        "SELECT * FROM message_version_content_refs WHERE id = ? LIMIT 1",
    )
    .bind(id)
    .fetch_optional(executor)
    .await?
    .ok_or_else(|| AppError::NotFound {
        entity: "message_version_content_ref",
        id: id.to_string(),
    })
}

fn map_active_message_row(row: SqliteRow) -> Result<ActiveMessageVersionRow> {
    Ok(ActiveMessageVersionRow {
        node: MessageNodeRow {
            id: row.try_get("node_id")?,
            conversation_id: row.try_get("conversation_id")?,
            author_participant_id: row.try_get("author_participant_id")?,
            role: row.try_get("role")?,
            reply_to_node_id: row.try_get("reply_to_node_id")?,
            order_key: row.try_get("order_key")?,
            created_at: row.try_get("node_created_at")?,
            updated_at: row.try_get("node_updated_at")?,
        },
        version: MessageVersionRow {
            id: row.try_get("version_id")?,
            node_id: row.try_get("version_node_id")?,
            version_index: row.try_get("version_index")?,
            is_active: row.try_get("is_active")?,
            primary_content_id: row.try_get("primary_content_id")?,
            context_policy: row.try_get("context_policy")?,
            viewer_policy: row.try_get("viewer_policy")?,
            api_channel_id: row.try_get("api_channel_id")?,
            api_channel_model_id: row.try_get("api_channel_model_id")?,
            prompt_tokens: row.try_get("prompt_tokens")?,
            completion_tokens: row.try_get("completion_tokens")?,
            total_tokens: row.try_get("total_tokens")?,
            finish_reason: row.try_get("finish_reason")?,
            generation_run_id: row.try_get("generation_run_id")?,
            config_json: row.try_get("config_json")?,
            created_at: row.try_get("version_created_at")?,
        },
    })
}
