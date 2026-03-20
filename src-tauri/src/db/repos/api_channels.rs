use sqlx::{Sqlite, SqlitePool, Transaction};

use crate::db::models::{ApiChannelModelRow, ApiChannelRow};
use crate::domain::api_channels::{
    CreateApiChannelInput, UpdateApiChannelInput, UpsertApiChannelModelInput,
};
use crate::support::error::{AppError, Result};
use crate::support::{ids, time};

pub async fn list_channels(db: &SqlitePool) -> Result<Vec<ApiChannelRow>> {
    sqlx::query_as::<_, ApiChannelRow>(
        r#"
        SELECT *
        FROM api_channels
        ORDER BY enabled DESC, sort_order ASC, created_at ASC
        "#,
    )
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

pub async fn get_channel(db: &SqlitePool, id: &str) -> Result<ApiChannelRow> {
    sqlx::query_as::<_, ApiChannelRow>("SELECT * FROM api_channels WHERE id = ? LIMIT 1")
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::NotFound {
            entity: "api_channel",
            id: id.to_string(),
        })
}

pub async fn create_channel(
    db: &SqlitePool,
    input: &CreateApiChannelInput,
) -> Result<ApiChannelRow> {
    let id = ids::new_id();
    let now = time::now_ms();

    sqlx::query(
        r#"
        INSERT INTO api_channels (
            id, name, channel_type, base_url, auth_type, api_key,
            models_endpoint, chat_endpoint, stream_endpoint, models_mode,
            enabled, sort_order, config_json, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&input.name)
    .bind(&input.channel_type)
    .bind(&input.base_url)
    .bind(&input.auth_type)
    .bind(&input.api_key)
    .bind(&input.models_endpoint)
    .bind(&input.chat_endpoint)
    .bind(&input.stream_endpoint)
    .bind(&input.models_mode)
    .bind(input.enabled)
    .bind(input.sort_order)
    .bind(input.config_json.to_string())
    .bind(now)
    .bind(now)
    .execute(db)
    .await?;

    get_channel(db, &id).await
}

pub async fn update_channel(
    db: &SqlitePool,
    id: &str,
    input: &UpdateApiChannelInput,
) -> Result<ApiChannelRow> {
    let affected = sqlx::query(
        r#"
        UPDATE api_channels
        SET name = ?, channel_type = ?, base_url = ?, auth_type = ?, api_key = ?,
            models_endpoint = ?, chat_endpoint = ?, stream_endpoint = ?, models_mode = ?,
            enabled = ?, sort_order = ?, config_json = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(&input.name)
    .bind(&input.channel_type)
    .bind(&input.base_url)
    .bind(&input.auth_type)
    .bind(&input.api_key)
    .bind(&input.models_endpoint)
    .bind(&input.chat_endpoint)
    .bind(&input.stream_endpoint)
    .bind(&input.models_mode)
    .bind(input.enabled)
    .bind(input.sort_order)
    .bind(input.config_json.to_string())
    .bind(time::now_ms())
    .bind(id)
    .execute(db)
    .await?
    .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "api_channel",
            id: id.to_string(),
        });
    }

    get_channel(db, id).await
}

pub async fn delete_channel(db: &SqlitePool, id: &str) -> Result<()> {
    let affected = sqlx::query("DELETE FROM api_channels WHERE id = ?")
        .bind(id)
        .execute(db)
        .await?
        .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "api_channel",
            id: id.to_string(),
        });
    }

    Ok(())
}

pub async fn list_channel_models(
    db: &SqlitePool,
    channel_id: &str,
) -> Result<Vec<ApiChannelModelRow>> {
    sqlx::query_as::<_, ApiChannelModelRow>(
        r#"
        SELECT *
        FROM api_channel_models
        WHERE channel_id = ?
        ORDER BY sort_order ASC, model_id ASC
        "#,
    )
    .bind(channel_id)
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

pub async fn get_channel_model(
    db: &SqlitePool,
    channel_id: &str,
    model_id: &str,
) -> Result<ApiChannelModelRow> {
    sqlx::query_as::<_, ApiChannelModelRow>(
        "SELECT * FROM api_channel_models WHERE channel_id = ? AND model_id = ? LIMIT 1",
    )
    .bind(channel_id)
    .bind(model_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::NotFound {
        entity: "api_channel_model",
        id: format!("{channel_id}:{model_id}"),
    })
}

pub async fn get_channel_model_by_id(db: &SqlitePool, id: &str) -> Result<ApiChannelModelRow> {
    sqlx::query_as::<_, ApiChannelModelRow>("SELECT * FROM api_channel_models WHERE id = ? LIMIT 1")
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::NotFound {
            entity: "api_channel_model",
            id: id.to_string(),
        })
}

pub async fn upsert_channel_model(
    db: &SqlitePool,
    input: &UpsertApiChannelModelInput,
) -> Result<ApiChannelModelRow> {
    sqlx::query(
        r#"
        INSERT INTO api_channel_models (
            id, channel_id, model_id, display_name, model_type, context_window,
            max_output_tokens, capabilities_json, pricing_json,
            default_parameters_json, sort_order, config_json
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(channel_id, model_id) DO UPDATE SET
            display_name = excluded.display_name,
            model_type = excluded.model_type,
            context_window = excluded.context_window,
            max_output_tokens = excluded.max_output_tokens,
            capabilities_json = excluded.capabilities_json,
            pricing_json = excluded.pricing_json,
            default_parameters_json = excluded.default_parameters_json,
            sort_order = excluded.sort_order,
            config_json = excluded.config_json
        "#,
    )
    .bind(ids::new_id())
    .bind(&input.channel_id)
    .bind(&input.model_id)
    .bind(&input.display_name)
    .bind(&input.model_type)
    .bind(input.context_window)
    .bind(input.max_output_tokens)
    .bind(input.capabilities_json.to_string())
    .bind(input.pricing_json.to_string())
    .bind(input.default_parameters_json.to_string())
    .bind(input.sort_order)
    .bind(input.config_json.to_string())
    .execute(db)
    .await?;

    get_channel_model(db, &input.channel_id, &input.model_id).await
}

pub async fn replace_channel_models(
    tx: &mut Transaction<'_, Sqlite>,
    channel_id: &str,
    models: &[UpsertApiChannelModelInput],
) -> Result<()> {
    sqlx::query("DELETE FROM api_channel_models WHERE channel_id = ?")
        .bind(channel_id)
        .execute(tx.as_mut())
        .await?;

    for input in models {
        sqlx::query(
            r#"
            INSERT INTO api_channel_models (
                id, channel_id, model_id, display_name, model_type, context_window,
                max_output_tokens, capabilities_json, pricing_json,
                default_parameters_json, sort_order, config_json
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(ids::new_id())
        .bind(channel_id)
        .bind(&input.model_id)
        .bind(&input.display_name)
        .bind(&input.model_type)
        .bind(input.context_window)
        .bind(input.max_output_tokens)
        .bind(input.capabilities_json.to_string())
        .bind(input.pricing_json.to_string())
        .bind(input.default_parameters_json.to_string())
        .bind(input.sort_order)
        .bind(input.config_json.to_string())
        .execute(tx.as_mut())
        .await?;
    }

    Ok(())
}

pub async fn delete_channel_model(db: &SqlitePool, channel_id: &str, model_id: &str) -> Result<()> {
    let affected =
        sqlx::query("DELETE FROM api_channel_models WHERE channel_id = ? AND model_id = ?")
            .bind(channel_id)
            .bind(model_id)
            .execute(db)
            .await?
            .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "api_channel_model",
            id: format!("{channel_id}:{model_id}"),
        });
    }

    Ok(())
}
