use sqlx::{Sqlite, SqlitePool, Transaction};

use crate::db::models::{
    AgentChannelBindingRow, AgentGreetingRow, AgentMediaRow, AgentResourceBindingRow, AgentRow,
};
use crate::support::error::{AppError, Result};
use crate::support::{ids, time};

pub struct CreateOrUpdateAgent<'a> {
    pub name: &'a str,
    pub title: Option<&'a str>,
    pub description_content_id: Option<&'a str>,
    pub personality_content_id: Option<&'a str>,
    pub scenario_content_id: Option<&'a str>,
    pub example_messages_content_id: Option<&'a str>,
    pub main_prompt_override_content_id: Option<&'a str>,
    pub post_history_instructions_content_id: Option<&'a str>,
    pub character_note_content_id: Option<&'a str>,
    pub creator_notes_content_id: Option<&'a str>,
    pub character_note_depth: Option<i64>,
    pub character_note_role: Option<&'a str>,
    pub talkativeness: i64,
    pub avatar_uri: Option<&'a str>,
    pub creator_name: Option<&'a str>,
    pub character_version: Option<&'a str>,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: &'a str,
}

pub struct CreateOrUpdateAgentGreeting<'a> {
    pub agent_id: &'a str,
    pub greeting_type: &'a str,
    pub name: Option<&'a str>,
    pub primary_content_id: &'a str,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: &'a str,
}

pub struct CreateAgentMedia<'a> {
    pub agent_id: &'a str,
    pub media_type: &'a str,
    pub content_id: &'a str,
    pub name: Option<&'a str>,
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

pub async fn list_agents(db: &SqlitePool) -> Result<Vec<AgentRow>> {
    sqlx::query_as::<_, AgentRow>(
        r#"
        SELECT *
        FROM agents
        ORDER BY enabled DESC, sort_order ASC, created_at ASC
        "#,
    )
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

pub async fn get_agent(db: &SqlitePool, id: &str) -> Result<AgentRow> {
    sqlx::query_as::<_, AgentRow>("SELECT * FROM agents WHERE id = ? LIMIT 1")
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::NotFound {
            entity: "agent",
            id: id.to_string(),
        })
}

pub async fn create_agent(db: &SqlitePool, input: &CreateOrUpdateAgent<'_>) -> Result<AgentRow> {
    let id = ids::new_id();
    let now = time::now_ms();

    sqlx::query(
        r#"
        INSERT INTO agents (
            id, name, title, description_content_id, personality_content_id, scenario_content_id,
            example_messages_content_id, main_prompt_override_content_id, post_history_instructions_content_id,
            character_note_content_id, creator_notes_content_id, character_note_depth, character_note_role,
            talkativeness, avatar_uri, creator_name, character_version, enabled, sort_order, config_json,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(input.name)
    .bind(input.title)
    .bind(input.description_content_id)
    .bind(input.personality_content_id)
    .bind(input.scenario_content_id)
    .bind(input.example_messages_content_id)
    .bind(input.main_prompt_override_content_id)
    .bind(input.post_history_instructions_content_id)
    .bind(input.character_note_content_id)
    .bind(input.creator_notes_content_id)
    .bind(input.character_note_depth)
    .bind(input.character_note_role)
    .bind(input.talkativeness)
    .bind(input.avatar_uri)
    .bind(input.creator_name)
    .bind(input.character_version)
    .bind(input.enabled)
    .bind(input.sort_order)
    .bind(input.config_json)
    .bind(now)
    .bind(now)
    .execute(db)
    .await?;

    get_agent(db, &id).await
}

pub async fn update_agent(
    db: &SqlitePool,
    id: &str,
    input: &CreateOrUpdateAgent<'_>,
) -> Result<AgentRow> {
    let affected = sqlx::query(
        r#"
        UPDATE agents
        SET name = ?, title = ?, description_content_id = ?, personality_content_id = ?, scenario_content_id = ?,
            example_messages_content_id = ?, main_prompt_override_content_id = ?, post_history_instructions_content_id = ?,
            character_note_content_id = ?, creator_notes_content_id = ?, character_note_depth = ?, character_note_role = ?,
            talkativeness = ?, avatar_uri = ?, creator_name = ?, character_version = ?, enabled = ?, sort_order = ?,
            config_json = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(input.name)
    .bind(input.title)
    .bind(input.description_content_id)
    .bind(input.personality_content_id)
    .bind(input.scenario_content_id)
    .bind(input.example_messages_content_id)
    .bind(input.main_prompt_override_content_id)
    .bind(input.post_history_instructions_content_id)
    .bind(input.character_note_content_id)
    .bind(input.creator_notes_content_id)
    .bind(input.character_note_depth)
    .bind(input.character_note_role)
    .bind(input.talkativeness)
    .bind(input.avatar_uri)
    .bind(input.creator_name)
    .bind(input.character_version)
    .bind(input.enabled)
    .bind(input.sort_order)
    .bind(input.config_json)
    .bind(time::now_ms())
    .bind(id)
    .execute(db)
    .await?
    .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "agent",
            id: id.to_string(),
        });
    }

    get_agent(db, id).await
}

pub async fn delete_agent(db: &SqlitePool, id: &str) -> Result<()> {
    let affected = sqlx::query("DELETE FROM agents WHERE id = ?")
        .bind(id)
        .execute(db)
        .await?
        .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "agent",
            id: id.to_string(),
        });
    }

    Ok(())
}

pub async fn list_agent_greetings(
    db: &SqlitePool,
    agent_id: &str,
) -> Result<Vec<AgentGreetingRow>> {
    sqlx::query_as::<_, AgentGreetingRow>(
        r#"
        SELECT *
        FROM agent_greetings
        WHERE agent_id = ?
        ORDER BY enabled DESC, sort_order ASC, created_at ASC
        "#,
    )
    .bind(agent_id)
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

pub async fn get_agent_greeting(db: &SqlitePool, id: &str) -> Result<AgentGreetingRow> {
    sqlx::query_as::<_, AgentGreetingRow>("SELECT * FROM agent_greetings WHERE id = ? LIMIT 1")
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::NotFound {
            entity: "agent_greeting",
            id: id.to_string(),
        })
}

pub async fn create_agent_greeting(
    db: &SqlitePool,
    input: &CreateOrUpdateAgentGreeting<'_>,
) -> Result<AgentGreetingRow> {
    let id = ids::new_id();
    let now = time::now_ms();

    sqlx::query(
        r#"
        INSERT INTO agent_greetings (
            id, agent_id, greeting_type, name, primary_content_id, enabled,
            sort_order, config_json, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(input.agent_id)
    .bind(input.greeting_type)
    .bind(input.name)
    .bind(input.primary_content_id)
    .bind(input.enabled)
    .bind(input.sort_order)
    .bind(input.config_json)
    .bind(now)
    .bind(now)
    .execute(db)
    .await?;

    get_agent_greeting(db, &id).await
}

pub async fn update_agent_greeting(
    db: &SqlitePool,
    id: &str,
    input: &CreateOrUpdateAgentGreeting<'_>,
) -> Result<AgentGreetingRow> {
    let affected = sqlx::query(
        r#"
        UPDATE agent_greetings
        SET greeting_type = ?, name = ?, primary_content_id = ?, enabled = ?, sort_order = ?,
            config_json = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(input.greeting_type)
    .bind(input.name)
    .bind(input.primary_content_id)
    .bind(input.enabled)
    .bind(input.sort_order)
    .bind(input.config_json)
    .bind(time::now_ms())
    .bind(id)
    .execute(db)
    .await?
    .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "agent_greeting",
            id: id.to_string(),
        });
    }

    get_agent_greeting(db, id).await
}

pub async fn delete_agent_greeting(db: &SqlitePool, id: &str) -> Result<()> {
    let affected = sqlx::query("DELETE FROM agent_greetings WHERE id = ?")
        .bind(id)
        .execute(db)
        .await?
        .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "agent_greeting",
            id: id.to_string(),
        });
    }

    Ok(())
}

pub async fn list_agent_media(db: &SqlitePool, agent_id: &str) -> Result<Vec<AgentMediaRow>> {
    sqlx::query_as::<_, AgentMediaRow>(
        r#"
        SELECT *
        FROM agent_media
        WHERE agent_id = ?
        ORDER BY enabled DESC, sort_order ASC, created_at ASC
        "#,
    )
    .bind(agent_id)
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

pub async fn create_agent_media(
    db: &SqlitePool,
    input: &CreateAgentMedia<'_>,
) -> Result<AgentMediaRow> {
    let id = ids::new_id();
    let now = time::now_ms();

    sqlx::query(
        r#"
        INSERT INTO agent_media (
            id, agent_id, media_type, content_id, name, enabled,
            sort_order, config_json, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(input.agent_id)
    .bind(input.media_type)
    .bind(input.content_id)
    .bind(input.name)
    .bind(input.enabled)
    .bind(input.sort_order)
    .bind(input.config_json)
    .bind(now)
    .bind(now)
    .execute(db)
    .await?;

    get_agent_media(db, &id).await
}

pub async fn get_agent_media(db: &SqlitePool, id: &str) -> Result<AgentMediaRow> {
    sqlx::query_as::<_, AgentMediaRow>("SELECT * FROM agent_media WHERE id = ? LIMIT 1")
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::NotFound {
            entity: "agent_media",
            id: id.to_string(),
        })
}

pub async fn delete_agent_media(db: &SqlitePool, id: &str) -> Result<()> {
    let affected = sqlx::query("DELETE FROM agent_media WHERE id = ?")
        .bind(id)
        .execute(db)
        .await?
        .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "agent_media",
            id: id.to_string(),
        });
    }

    Ok(())
}

pub async fn list_agent_preset_bindings(
    db: &SqlitePool,
    agent_id: &str,
) -> Result<Vec<AgentResourceBindingRow>> {
    list_agent_resource_bindings(db, "agent_preset_bindings", "preset_id", agent_id).await
}

pub async fn list_agent_lorebook_bindings(
    db: &SqlitePool,
    agent_id: &str,
) -> Result<Vec<AgentResourceBindingRow>> {
    list_agent_resource_bindings(db, "agent_lorebook_bindings", "lorebook_id", agent_id).await
}

pub async fn list_agent_user_profile_bindings(
    db: &SqlitePool,
    agent_id: &str,
) -> Result<Vec<AgentResourceBindingRow>> {
    list_agent_resource_bindings(
        db,
        "agent_user_profile_bindings",
        "user_profile_id",
        agent_id,
    )
    .await
}

pub async fn replace_agent_preset_bindings(
    tx: &mut Transaction<'_, Sqlite>,
    agent_id: &str,
    items: &[ResourceBindingRecord<'_>],
) -> Result<()> {
    replace_agent_resource_bindings(tx, "agent_preset_bindings", "preset_id", agent_id, items).await
}

pub async fn replace_agent_lorebook_bindings(
    tx: &mut Transaction<'_, Sqlite>,
    agent_id: &str,
    items: &[ResourceBindingRecord<'_>],
) -> Result<()> {
    replace_agent_resource_bindings(
        tx,
        "agent_lorebook_bindings",
        "lorebook_id",
        agent_id,
        items,
    )
    .await
}

pub async fn replace_agent_user_profile_bindings(
    tx: &mut Transaction<'_, Sqlite>,
    agent_id: &str,
    items: &[ResourceBindingRecord<'_>],
) -> Result<()> {
    replace_agent_resource_bindings(
        tx,
        "agent_user_profile_bindings",
        "user_profile_id",
        agent_id,
        items,
    )
    .await
}

pub async fn list_agent_channel_bindings(
    db: &SqlitePool,
    agent_id: &str,
) -> Result<Vec<AgentChannelBindingRow>> {
    sqlx::query_as::<_, AgentChannelBindingRow>(
        r#"
        SELECT *
        FROM agent_channel_bindings
        WHERE agent_id = ?
        ORDER BY enabled DESC, sort_order ASC, created_at ASC
        "#,
    )
    .bind(agent_id)
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

pub async fn replace_agent_channel_bindings(
    tx: &mut Transaction<'_, Sqlite>,
    agent_id: &str,
    items: &[ChannelBindingRecord<'_>],
) -> Result<()> {
    sqlx::query("DELETE FROM agent_channel_bindings WHERE agent_id = ?")
        .bind(agent_id)
        .execute(tx.as_mut())
        .await?;

    let now = time::now_ms();
    for item in items {
        sqlx::query(
            r#"
            INSERT INTO agent_channel_bindings (
                id, agent_id, channel_id, channel_model_id, binding_type, enabled,
                sort_order, config_json, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(ids::new_id())
        .bind(agent_id)
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

async fn list_agent_resource_bindings(
    db: &SqlitePool,
    table: &str,
    resource_column: &str,
    agent_id: &str,
) -> Result<Vec<AgentResourceBindingRow>> {
    let sql = format!(
        r#"
        SELECT
            id,
            agent_id,
            {resource_column} AS resource_id,
            binding_type,
            enabled,
            sort_order,
            config_json,
            created_at,
            updated_at
        FROM {table}
        WHERE agent_id = ?
        ORDER BY enabled DESC, sort_order ASC, created_at ASC
        "#
    );

    sqlx::query_as::<_, AgentResourceBindingRow>(&sql)
        .bind(agent_id)
        .fetch_all(db)
        .await
        .map_err(Into::into)
}

async fn replace_agent_resource_bindings(
    tx: &mut Transaction<'_, Sqlite>,
    table: &str,
    resource_column: &str,
    agent_id: &str,
    items: &[ResourceBindingRecord<'_>],
) -> Result<()> {
    let delete_sql = format!("DELETE FROM {table} WHERE agent_id = ?");
    sqlx::query(&delete_sql)
        .bind(agent_id)
        .execute(tx.as_mut())
        .await?;

    let insert_sql = format!(
        r#"
        INSERT INTO {table} (
            id, agent_id, {resource_column}, binding_type, enabled,
            sort_order, config_json, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#
    );

    let now = time::now_ms();
    for item in items {
        sqlx::query(&insert_sql)
            .bind(ids::new_id())
            .bind(agent_id)
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
