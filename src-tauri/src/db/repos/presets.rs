use sqlx::{Sqlite, SqlitePool, Transaction};

use crate::db::models::{PresetChannelBindingRow, PresetEntryRow, PresetRow};
use crate::support::error::{AppError, Result};
use crate::support::{ids, time};

pub struct InsertPresetEntry<'a> {
    pub preset_id: &'a str,
    pub name: &'a str,
    pub role: &'a str,
    pub primary_content_id: &'a str,
    pub position_type: &'a str,
    pub list_order: i64,
    pub depth: Option<i64>,
    pub depth_order: i64,
    pub triggers_json: &'a str,
    pub enabled: bool,
    pub is_pinned: bool,
    pub config_json: &'a str,
}

pub struct UpdatePresetEntry<'a> {
    pub name: &'a str,
    pub role: &'a str,
    pub primary_content_id: &'a str,
    pub position_type: &'a str,
    pub list_order: i64,
    pub depth: Option<i64>,
    pub depth_order: i64,
    pub triggers_json: &'a str,
    pub enabled: bool,
    pub is_pinned: bool,
    pub config_json: &'a str,
}

pub struct UpsertPresetChannelBinding<'a> {
    pub preset_id: &'a str,
    pub channel_id: &'a str,
    pub channel_model_id: Option<&'a str>,
    pub binding_type: &'a str,
    pub enabled: bool,
    pub sort_order: i64,
    pub config_json: &'a str,
}

pub async fn list_presets(db: &SqlitePool) -> Result<Vec<PresetRow>> {
    sqlx::query_as::<_, PresetRow>(
        r#"
        SELECT *
        FROM presets
        ORDER BY enabled DESC, sort_order ASC, created_at ASC
        "#,
    )
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

pub async fn get_preset(db: &SqlitePool, id: &str) -> Result<PresetRow> {
    sqlx::query_as::<_, PresetRow>("SELECT * FROM presets WHERE id = ? LIMIT 1")
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::NotFound {
            entity: "preset",
            id: id.to_string(),
        })
}

pub async fn create_preset(
    db: &SqlitePool,
    name: &str,
    description: Option<&str>,
    enabled: bool,
    sort_order: i64,
    config_json: &str,
) -> Result<PresetRow> {
    let id = ids::new_id();
    let now = time::now_ms();

    sqlx::query(
        r#"
        INSERT INTO presets (
            id, name, description, enabled, sort_order, config_json, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(name)
    .bind(description)
    .bind(enabled)
    .bind(sort_order)
    .bind(config_json)
    .bind(now)
    .bind(now)
    .execute(db)
    .await?;

    get_preset(db, &id).await
}

pub async fn update_preset(
    db: &SqlitePool,
    id: &str,
    name: &str,
    description: Option<&str>,
    enabled: bool,
    sort_order: i64,
    config_json: &str,
) -> Result<PresetRow> {
    let affected = sqlx::query(
        r#"
        UPDATE presets
        SET name = ?, description = ?, enabled = ?, sort_order = ?, config_json = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(name)
    .bind(description)
    .bind(enabled)
    .bind(sort_order)
    .bind(config_json)
    .bind(time::now_ms())
    .bind(id)
    .execute(db)
    .await?
    .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "preset",
            id: id.to_string(),
        });
    }

    get_preset(db, id).await
}

pub async fn delete_preset(db: &SqlitePool, id: &str) -> Result<()> {
    let affected = sqlx::query("DELETE FROM presets WHERE id = ?")
        .bind(id)
        .execute(db)
        .await?
        .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "preset",
            id: id.to_string(),
        });
    }

    Ok(())
}

pub async fn list_preset_entries(db: &SqlitePool, preset_id: &str) -> Result<Vec<PresetEntryRow>> {
    sqlx::query_as::<_, PresetEntryRow>(
        r#"
        SELECT *
        FROM preset_entries
        WHERE preset_id = ?
        ORDER BY list_order ASC, depth_order ASC, created_at ASC
        "#,
    )
    .bind(preset_id)
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

pub async fn get_preset_entry(db: &SqlitePool, id: &str) -> Result<PresetEntryRow> {
    sqlx::query_as::<_, PresetEntryRow>("SELECT * FROM preset_entries WHERE id = ? LIMIT 1")
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::NotFound {
            entity: "preset_entry",
            id: id.to_string(),
        })
}

pub async fn create_preset_entry(
    db: &SqlitePool,
    input: &InsertPresetEntry<'_>,
) -> Result<PresetEntryRow> {
    let id = ids::new_id();
    let now = time::now_ms();

    sqlx::query(
        r#"
        INSERT INTO preset_entries (
            id, preset_id, name, role, primary_content_id, position_type, list_order, depth,
            depth_order, triggers_json, enabled, is_pinned, config_json, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(input.preset_id)
    .bind(input.name)
    .bind(input.role)
    .bind(input.primary_content_id)
    .bind(input.position_type)
    .bind(input.list_order)
    .bind(input.depth)
    .bind(input.depth_order)
    .bind(input.triggers_json)
    .bind(input.enabled)
    .bind(input.is_pinned)
    .bind(input.config_json)
    .bind(now)
    .bind(now)
    .execute(db)
    .await?;

    get_preset_entry(db, &id).await
}

pub async fn update_preset_entry(
    db: &SqlitePool,
    id: &str,
    input: &UpdatePresetEntry<'_>,
) -> Result<PresetEntryRow> {
    let affected = sqlx::query(
        r#"
        UPDATE preset_entries
        SET name = ?, role = ?, primary_content_id = ?, position_type = ?, list_order = ?, depth = ?,
            depth_order = ?, triggers_json = ?, enabled = ?, is_pinned = ?, config_json = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(input.name)
    .bind(input.role)
    .bind(input.primary_content_id)
    .bind(input.position_type)
    .bind(input.list_order)
    .bind(input.depth)
    .bind(input.depth_order)
    .bind(input.triggers_json)
    .bind(input.enabled)
    .bind(input.is_pinned)
    .bind(input.config_json)
    .bind(time::now_ms())
    .bind(id)
    .execute(db)
    .await?
    .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "preset_entry",
            id: id.to_string(),
        });
    }

    get_preset_entry(db, id).await
}

pub async fn delete_preset_entry(db: &SqlitePool, id: &str) -> Result<()> {
    let affected = sqlx::query("DELETE FROM preset_entries WHERE id = ?")
        .bind(id)
        .execute(db)
        .await?
        .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "preset_entry",
            id: id.to_string(),
        });
    }

    Ok(())
}

pub async fn reorder_preset_entries(
    tx: &mut Transaction<'_, Sqlite>,
    preset_id: &str,
    entry_ids: &[String],
) -> Result<()> {
    for (index, entry_id) in entry_ids.iter().enumerate() {
        let affected = sqlx::query(
            r#"
            UPDATE preset_entries
            SET list_order = ?, updated_at = ?
            WHERE id = ? AND preset_id = ?
            "#,
        )
        .bind(index as i64)
        .bind(time::now_ms())
        .bind(entry_id)
        .bind(preset_id)
        .execute(tx.as_mut())
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFound {
                entity: "preset_entry",
                id: entry_id.clone(),
            });
        }
    }

    Ok(())
}

pub async fn list_preset_channel_bindings(
    db: &SqlitePool,
    preset_id: &str,
) -> Result<Vec<PresetChannelBindingRow>> {
    sqlx::query_as::<_, PresetChannelBindingRow>(
        r#"
        SELECT *
        FROM preset_channel_bindings
        WHERE preset_id = ?
        ORDER BY enabled DESC, sort_order ASC, created_at ASC
        "#,
    )
    .bind(preset_id)
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

pub async fn upsert_preset_channel_binding(
    db: &SqlitePool,
    input: &UpsertPresetChannelBinding<'_>,
) -> Result<PresetChannelBindingRow> {
    if let Some(existing_id) = find_preset_channel_binding_id(
        db,
        input.preset_id,
        input.channel_id,
        input.channel_model_id,
    )
    .await?
    {
        let affected = sqlx::query(
            r#"
            UPDATE preset_channel_bindings
            SET binding_type = ?, enabled = ?, sort_order = ?, config_json = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(input.binding_type)
        .bind(input.enabled)
        .bind(input.sort_order)
        .bind(input.config_json)
        .bind(time::now_ms())
        .bind(&existing_id)
        .execute(db)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFound {
                entity: "preset_channel_binding",
                id: existing_id,
            });
        }

        return get_preset_channel_binding_by_id(db, &existing_id).await;
    }

    let id = ids::new_id();
    let now = time::now_ms();

    sqlx::query(
        r#"
        INSERT INTO preset_channel_bindings (
            id, preset_id, channel_id, channel_model_id, binding_type, enabled,
            sort_order, config_json, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(input.preset_id)
    .bind(input.channel_id)
    .bind(input.channel_model_id)
    .bind(input.binding_type)
    .bind(input.enabled)
    .bind(input.sort_order)
    .bind(input.config_json)
    .bind(now)
    .bind(now)
    .execute(db)
    .await?;

    get_preset_channel_binding_by_id(db, &id).await
}

pub async fn delete_preset_channel_binding(
    db: &SqlitePool,
    preset_id: &str,
    channel_id: &str,
    channel_model_id: Option<&str>,
) -> Result<()> {
    let sql = if channel_model_id.is_some() {
        "DELETE FROM preset_channel_bindings WHERE preset_id = ? AND channel_id = ? AND channel_model_id = ?"
    } else {
        "DELETE FROM preset_channel_bindings WHERE preset_id = ? AND channel_id = ? AND channel_model_id IS NULL"
    };

    let mut query = sqlx::query(sql).bind(preset_id).bind(channel_id);
    if let Some(channel_model_id) = channel_model_id {
        query = query.bind(channel_model_id);
    }

    let affected = query.execute(db).await?.rows_affected();
    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "preset_channel_binding",
            id: format!(
                "{preset_id}:{channel_id}:{}",
                channel_model_id.unwrap_or("null")
            ),
        });
    }

    Ok(())
}

async fn find_preset_channel_binding_id(
    db: &SqlitePool,
    preset_id: &str,
    channel_id: &str,
    channel_model_id: Option<&str>,
) -> Result<Option<String>> {
    let sql = if channel_model_id.is_some() {
        r#"
        SELECT id
        FROM preset_channel_bindings
        WHERE preset_id = ? AND channel_id = ? AND channel_model_id = ?
        LIMIT 1
        "#
    } else {
        r#"
        SELECT id
        FROM preset_channel_bindings
        WHERE preset_id = ? AND channel_id = ? AND channel_model_id IS NULL
        LIMIT 1
        "#
    };

    let mut query = sqlx::query_scalar::<_, String>(sql)
        .bind(preset_id)
        .bind(channel_id);
    if let Some(channel_model_id) = channel_model_id {
        query = query.bind(channel_model_id);
    }

    query.fetch_optional(db).await.map_err(Into::into)
}

async fn get_preset_channel_binding_by_id(
    db: &SqlitePool,
    id: &str,
) -> Result<PresetChannelBindingRow> {
    sqlx::query_as::<_, PresetChannelBindingRow>(
        "SELECT * FROM preset_channel_bindings WHERE id = ? LIMIT 1",
    )
    .bind(id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::NotFound {
        entity: "preset_channel_binding",
        id: id.to_string(),
    })
}
