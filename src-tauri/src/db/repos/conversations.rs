use sqlx::{Sqlite, SqlitePool, Transaction};

use crate::db::models::{
    ConversationChannelBindingRow, ConversationParticipantRow, ConversationResourceBindingRow,
    ConversationRow,
};
use crate::support::error::{AppError, Result};
use crate::support::{ids, time};

pub struct CreateConversationRecord<'a> {
    pub title: &'a str,
    pub description: Option<&'a str>,
    pub conversation_mode: &'a str,
    pub archived: bool,
    pub pinned: bool,
    pub config_json: &'a str,
}

pub struct UpdateConversationRecord<'a> {
    pub title: &'a str,
    pub description: Option<&'a str>,
    pub archived: bool,
    pub pinned: bool,
    pub config_json: &'a str,
}

pub struct ConversationParticipantRecord<'a> {
    pub agent_id: Option<&'a str>,
    pub display_name: Option<&'a str>,
    pub participant_type: &'a str,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: &'a str,
}

pub struct ResourceBindingRecord<'a> {
    pub resource_id: &'a str,
    pub binding_type: &'a str,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: &'a str,
}

pub struct ChannelBindingRecord<'a> {
    pub channel_id: &'a str,
    pub channel_model_id: Option<&'a str>,
    pub binding_type: &'a str,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: &'a str,
}

pub async fn list_conversations(db: &SqlitePool) -> Result<Vec<ConversationRow>> {
    sqlx::query_as::<_, ConversationRow>(
        r#"
        SELECT *
        FROM conversations
        ORDER BY pinned DESC, updated_at DESC
        "#,
    )
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

pub async fn get_conversation(db: &SqlitePool, id: &str) -> Result<ConversationRow> {
    sqlx::query_as::<_, ConversationRow>("SELECT * FROM conversations WHERE id = ? LIMIT 1")
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::NotFound {
            entity: "conversation",
            id: id.to_string(),
        })
}

pub async fn get_conversation_participant(
    db: &SqlitePool,
    id: &str,
) -> Result<ConversationParticipantRow> {
    sqlx::query_as::<_, ConversationParticipantRow>(
        "SELECT * FROM conversation_participants WHERE id = ? LIMIT 1",
    )
    .bind(id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::NotFound {
        entity: "conversation_participant",
        id: id.to_string(),
    })
}

pub async fn create_conversation(
    db: &SqlitePool,
    input: &CreateConversationRecord<'_>,
) -> Result<ConversationRow> {
    let id = ids::new_id();
    let now = time::now_ms();

    sqlx::query(
        r#"
        INSERT INTO conversations (
            id, title, description, conversation_mode, archived, pinned,
            config_json, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(input.title)
    .bind(input.description)
    .bind(input.conversation_mode)
    .bind(input.archived)
    .bind(input.pinned)
    .bind(input.config_json)
    .bind(now)
    .bind(now)
    .execute(db)
    .await?;

    get_conversation(db, &id).await
}

pub async fn update_conversation(
    db: &SqlitePool,
    id: &str,
    input: &UpdateConversationRecord<'_>,
) -> Result<ConversationRow> {
    let affected = sqlx::query(
        r#"
        UPDATE conversations
        SET title = ?, description = ?, archived = ?, pinned = ?, config_json = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(input.title)
    .bind(input.description)
    .bind(input.archived)
    .bind(input.pinned)
    .bind(input.config_json)
    .bind(time::now_ms())
    .bind(id)
    .execute(db)
    .await?
    .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "conversation",
            id: id.to_string(),
        });
    }

    get_conversation(db, id).await
}

pub async fn rename_conversation(
    db: &SqlitePool,
    id: &str,
    title: &str,
) -> Result<ConversationRow> {
    let affected = sqlx::query("UPDATE conversations SET title = ?, updated_at = ? WHERE id = ?")
        .bind(title)
        .bind(time::now_ms())
        .bind(id)
        .execute(db)
        .await?
        .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "conversation",
            id: id.to_string(),
        });
    }

    get_conversation(db, id).await
}

pub async fn delete_conversation(db: &SqlitePool, id: &str) -> Result<()> {
    let affected = sqlx::query("DELETE FROM conversations WHERE id = ?")
        .bind(id)
        .execute(db)
        .await?
        .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "conversation",
            id: id.to_string(),
        });
    }

    Ok(())
}

pub async fn touch_conversation(db: &SqlitePool, id: &str) -> Result<()> {
    let affected = sqlx::query("UPDATE conversations SET updated_at = ? WHERE id = ?")
        .bind(time::now_ms())
        .bind(id)
        .execute(db)
        .await?
        .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "conversation",
            id: id.to_string(),
        });
    }

    Ok(())
}

pub async fn list_conversation_participants(
    db: &SqlitePool,
    conversation_id: &str,
) -> Result<Vec<ConversationParticipantRow>> {
    sqlx::query_as::<_, ConversationParticipantRow>(
        r#"
        SELECT *
        FROM conversation_participants
        WHERE conversation_id = ?
        ORDER BY enabled DESC, sort_order ASC, created_at ASC
        "#,
    )
    .bind(conversation_id)
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

pub async fn replace_conversation_participants(
    tx: &mut Transaction<'_, Sqlite>,
    conversation_id: &str,
    items: &[ConversationParticipantRecord<'_>],
) -> Result<()> {
    sqlx::query("DELETE FROM conversation_participants WHERE conversation_id = ?")
        .bind(conversation_id)
        .execute(tx.as_mut())
        .await?;

    let now = time::now_ms();
    for item in items {
        sqlx::query(
            r#"
            INSERT INTO conversation_participants (
                id, conversation_id, agent_id, display_name, participant_type,
                enabled, sort_order, config_json, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(ids::new_id())
        .bind(conversation_id)
        .bind(item.agent_id)
        .bind(item.display_name)
        .bind(item.participant_type)
        .bind(item.enabled)
        .bind(item.sort_order)
        .bind(item.config_json)
        .bind(now)
        .bind(now)
        .execute(tx.as_mut())
        .await?;
    }

    Ok(())
}

pub async fn create_conversation_participant(
    db: &SqlitePool,
    conversation_id: &str,
    input: &ConversationParticipantRecord<'_>,
) -> Result<ConversationParticipantRow> {
    let id = ids::new_id();
    let now = time::now_ms();

    sqlx::query(
        r#"
        INSERT INTO conversation_participants (
            id, conversation_id, agent_id, display_name, participant_type,
            enabled, sort_order, config_json, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(conversation_id)
    .bind(input.agent_id)
    .bind(input.display_name)
    .bind(input.participant_type)
    .bind(input.enabled)
    .bind(input.sort_order)
    .bind(input.config_json)
    .bind(now)
    .bind(now)
    .execute(db)
    .await?;

    get_conversation_participant(db, &id).await
}

pub async fn list_conversation_preset_bindings(
    db: &SqlitePool,
    conversation_id: &str,
) -> Result<Vec<ConversationResourceBindingRow>> {
    list_conversation_resource_bindings(
        db,
        "conversation_preset_bindings",
        "preset_id",
        conversation_id,
    )
    .await
}

pub async fn list_conversation_lorebook_bindings(
    db: &SqlitePool,
    conversation_id: &str,
) -> Result<Vec<ConversationResourceBindingRow>> {
    list_conversation_resource_bindings(
        db,
        "conversation_lorebook_bindings",
        "lorebook_id",
        conversation_id,
    )
    .await
}

pub async fn list_conversation_user_profile_bindings(
    db: &SqlitePool,
    conversation_id: &str,
) -> Result<Vec<ConversationResourceBindingRow>> {
    list_conversation_resource_bindings(
        db,
        "conversation_user_profile_bindings",
        "user_profile_id",
        conversation_id,
    )
    .await
}

pub async fn replace_conversation_preset_bindings(
    tx: &mut Transaction<'_, Sqlite>,
    conversation_id: &str,
    items: &[ResourceBindingRecord<'_>],
) -> Result<()> {
    replace_conversation_resource_bindings(
        tx,
        "conversation_preset_bindings",
        "preset_id",
        conversation_id,
        items,
    )
    .await
}

pub async fn replace_conversation_lorebook_bindings(
    tx: &mut Transaction<'_, Sqlite>,
    conversation_id: &str,
    items: &[ResourceBindingRecord<'_>],
) -> Result<()> {
    replace_conversation_resource_bindings(
        tx,
        "conversation_lorebook_bindings",
        "lorebook_id",
        conversation_id,
        items,
    )
    .await
}

pub async fn replace_conversation_user_profile_bindings(
    tx: &mut Transaction<'_, Sqlite>,
    conversation_id: &str,
    items: &[ResourceBindingRecord<'_>],
) -> Result<()> {
    replace_conversation_resource_bindings(
        tx,
        "conversation_user_profile_bindings",
        "user_profile_id",
        conversation_id,
        items,
    )
    .await
}

pub async fn list_conversation_channel_bindings(
    db: &SqlitePool,
    conversation_id: &str,
) -> Result<Vec<ConversationChannelBindingRow>> {
    sqlx::query_as::<_, ConversationChannelBindingRow>(
        r#"
        SELECT *
        FROM conversation_channel_bindings
        WHERE conversation_id = ?
        ORDER BY enabled DESC, sort_order ASC, created_at ASC
        "#,
    )
    .bind(conversation_id)
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

pub async fn replace_conversation_channel_bindings(
    tx: &mut Transaction<'_, Sqlite>,
    conversation_id: &str,
    items: &[ChannelBindingRecord<'_>],
) -> Result<()> {
    sqlx::query("DELETE FROM conversation_channel_bindings WHERE conversation_id = ?")
        .bind(conversation_id)
        .execute(tx.as_mut())
        .await?;

    let now = time::now_ms();
    for item in items {
        sqlx::query(
            r#"
            INSERT INTO conversation_channel_bindings (
                id, conversation_id, channel_id, channel_model_id, binding_type, enabled,
                sort_order, config_json, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(ids::new_id())
        .bind(conversation_id)
        .bind(item.channel_id)
        .bind(item.channel_model_id)
        .bind(item.binding_type)
        .bind(item.enabled)
        .bind(item.sort_order)
        .bind(item.config_json)
        .bind(now)
        .bind(now)
        .execute(tx.as_mut())
        .await?;
    }

    Ok(())
}

async fn list_conversation_resource_bindings(
    db: &SqlitePool,
    table: &str,
    resource_column: &str,
    conversation_id: &str,
) -> Result<Vec<ConversationResourceBindingRow>> {
    let sql = format!(
        r#"
        SELECT
            id,
            conversation_id,
            {resource_column} AS resource_id,
            binding_type,
            enabled,
            sort_order,
            config_json,
            created_at,
            updated_at
        FROM {table}
        WHERE conversation_id = ?
        ORDER BY enabled DESC, sort_order ASC, created_at ASC
        "#
    );

    sqlx::query_as::<_, ConversationResourceBindingRow>(&sql)
        .bind(conversation_id)
        .fetch_all(db)
        .await
        .map_err(Into::into)
}

async fn replace_conversation_resource_bindings(
    tx: &mut Transaction<'_, Sqlite>,
    table: &str,
    resource_column: &str,
    conversation_id: &str,
    items: &[ResourceBindingRecord<'_>],
) -> Result<()> {
    let delete_sql = format!("DELETE FROM {table} WHERE conversation_id = ?");
    sqlx::query(&delete_sql)
        .bind(conversation_id)
        .execute(tx.as_mut())
        .await?;

    let insert_sql = format!(
        r#"
        INSERT INTO {table} (
            id, conversation_id, {resource_column}, binding_type, enabled,
            sort_order, config_json, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#
    );

    let now = time::now_ms();
    for item in items {
        sqlx::query(&insert_sql)
            .bind(ids::new_id())
            .bind(conversation_id)
            .bind(item.resource_id)
            .bind(item.binding_type)
            .bind(item.enabled)
            .bind(item.sort_order)
            .bind(item.config_json)
            .bind(now)
            .bind(now)
            .execute(tx.as_mut())
            .await?;
    }

    Ok(())
}
