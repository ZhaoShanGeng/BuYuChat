use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::db::models::CustomChannelRow;
use crate::error::{AppError, Result};

pub async fn list_all(db: &SqlitePool) -> Result<Vec<CustomChannelRow>> {
    let items = sqlx::query_as::<_, CustomChannelRow>(
        "SELECT * FROM custom_channels ORDER BY updated_at DESC, created_at DESC",
    )
    .fetch_all(db)
    .await?;
    Ok(items)
}

pub async fn list_enabled(db: &SqlitePool) -> Result<Vec<CustomChannelRow>> {
    let items = sqlx::query_as::<_, CustomChannelRow>(
        "SELECT * FROM custom_channels WHERE enabled = 1 ORDER BY updated_at DESC, created_at DESC",
    )
    .fetch_all(db)
    .await?;
    Ok(items)
}

pub async fn get(db: &SqlitePool, id: &str) -> Result<CustomChannelRow> {
    sqlx::query_as::<_, CustomChannelRow>("SELECT * FROM custom_channels WHERE id = ?")
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::NotFound {
            entity: "custom_channel",
            id: id.to_string(),
        })
}

pub async fn create(
    db: &SqlitePool,
    name: &str,
    channel_type: &str,
    base_url: &str,
    auth_json: &str,
    endpoints_json: &str,
    request_template_json: &str,
    response_mapping_json: &str,
    stream_mapping_json: &str,
    models_json: &str,
) -> Result<CustomChannelRow> {
    let id = Uuid::now_v7().to_string();
    let now = Utc::now().timestamp_millis();

    sqlx::query(
        r#"
        INSERT INTO custom_channels (
            id, name, channel_type, base_url, auth_json, endpoints_json, stream_protocol,
            request_template_json, response_mapping_json, stream_mapping_json, models_json,
            enabled, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, 'sse', ?, ?, ?, ?, 1, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(name)
    .bind(channel_type)
    .bind(base_url)
    .bind(auth_json)
    .bind(endpoints_json)
    .bind(request_template_json)
    .bind(response_mapping_json)
    .bind(stream_mapping_json)
    .bind(models_json)
    .bind(now)
    .bind(now)
    .execute(db)
    .await?;

    get(db, &id).await
}

pub async fn update(
    db: &SqlitePool,
    id: &str,
    name: &str,
    channel_type: &str,
    base_url: &str,
    auth_json: &str,
    endpoints_json: &str,
    request_template_json: &str,
    response_mapping_json: &str,
    stream_mapping_json: &str,
    models_json: &str,
    enabled: bool,
) -> Result<CustomChannelRow> {
    let affected = sqlx::query(
        r#"
        UPDATE custom_channels
        SET name = ?, channel_type = ?, base_url = ?, auth_json = ?, endpoints_json = ?, request_template_json = ?,
            response_mapping_json = ?, stream_mapping_json = ?, models_json = ?, enabled = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(name)
    .bind(channel_type)
    .bind(base_url)
    .bind(auth_json)
    .bind(endpoints_json)
    .bind(request_template_json)
    .bind(response_mapping_json)
    .bind(stream_mapping_json)
    .bind(models_json)
    .bind(enabled)
    .bind(Utc::now().timestamp_millis())
    .bind(id)
    .execute(db)
    .await?
    .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "custom_channel",
            id: id.to_string(),
        });
    }

    get(db, id).await
}

pub async fn delete(db: &SqlitePool, id: &str) -> Result<()> {
    let affected = sqlx::query("DELETE FROM custom_channels WHERE id = ?")
        .bind(id)
        .execute(db)
        .await?
        .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "custom_channel",
            id: id.to_string(),
        });
    }

    Ok(())
}

pub async fn update_models_json(
    db: &SqlitePool,
    id: &str,
    models_json: &str,
) -> Result<CustomChannelRow> {
    let affected =
        sqlx::query("UPDATE custom_channels SET models_json = ?, updated_at = ? WHERE id = ?")
            .bind(models_json)
            .bind(Utc::now().timestamp_millis())
            .bind(id)
            .execute(db)
            .await?
            .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "custom_channel",
            id: id.to_string(),
        });
    }

    get(db, id).await
}
