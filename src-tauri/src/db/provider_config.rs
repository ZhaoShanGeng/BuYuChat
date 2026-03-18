use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::db::models::ProviderConfigRow;
use crate::error::Result;

pub async fn list_all(db: &SqlitePool) -> Result<Vec<ProviderConfigRow>> {
    let items = sqlx::query_as::<_, ProviderConfigRow>(
        "SELECT * FROM provider_configs ORDER BY provider ASC",
    )
    .fetch_all(db)
    .await?;
    Ok(items)
}

pub async fn list_enabled(db: &SqlitePool) -> Result<Vec<ProviderConfigRow>> {
    let items = sqlx::query_as::<_, ProviderConfigRow>(
        "SELECT * FROM provider_configs WHERE enabled = 1 ORDER BY provider ASC",
    )
    .fetch_all(db)
    .await?;
    Ok(items)
}

pub async fn get_by_provider(db: &SqlitePool, provider: &str) -> Result<Option<ProviderConfigRow>> {
    let item = sqlx::query_as::<_, ProviderConfigRow>(
        "SELECT * FROM provider_configs WHERE provider = ? LIMIT 1",
    )
    .bind(provider)
    .fetch_optional(db)
    .await?;
    Ok(item)
}

pub async fn save(
    db: &SqlitePool,
    provider: &str,
    api_key_id: Option<&str>,
    base_url: Option<&str>,
    enabled: bool,
) -> Result<ProviderConfigRow> {
    let now = Utc::now().timestamp_millis();
    let id = Uuid::now_v7().to_string();

    sqlx::query(
        r#"
        INSERT INTO provider_configs (id, provider, api_key_id, base_url, extra_json, enabled, updated_at)
        VALUES (?, ?, ?, ?, NULL, ?, ?)
        ON CONFLICT(provider) DO UPDATE SET
            api_key_id = excluded.api_key_id,
            base_url = excluded.base_url,
            enabled = excluded.enabled,
            updated_at = excluded.updated_at
        "#,
    )
    .bind(id)
    .bind(provider)
    .bind(api_key_id)
    .bind(base_url)
    .bind(enabled)
    .bind(now)
    .execute(db)
    .await?;

    get_by_provider(db, provider).await?.ok_or_else(|| {
        crate::error::AppError::Other(format!("failed to save provider config '{provider}'"))
    })
}
