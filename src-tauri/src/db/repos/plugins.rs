use crate::db::models::PluginDefRow;
use crate::domain::plugins::json_has_capability;
use crate::support::error::{AppError, Result};
use crate::support::{ids, time};
use sqlx::SqlitePool;

pub struct CreatePluginRecord<'a> {
    pub name: &'a str,
    pub plugin_key: &'a str,
    pub version: &'a str,
    pub runtime_kind: &'a str,
    pub entrypoint: Option<&'a str>,
    pub enabled: bool,
    pub sort_order: i64,
    pub capabilities_json: &'a str,
    pub permissions_json: &'a str,
    pub config_json: &'a str,
}

pub struct UpdatePluginRecord<'a> {
    pub name: &'a str,
    pub version: &'a str,
    pub runtime_kind: &'a str,
    pub entrypoint: Option<&'a str>,
    pub enabled: bool,
    pub sort_order: i64,
    pub capabilities_json: &'a str,
    pub permissions_json: &'a str,
    pub config_json: &'a str,
}

pub async fn list_plugins(db: &SqlitePool) -> Result<Vec<PluginDefRow>> {
    sqlx::query_as::<_, PluginDefRow>(
        r#"
        SELECT *
        FROM plugin_defs
        ORDER BY enabled DESC, sort_order ASC, updated_at DESC
        "#,
    )
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

pub async fn get_plugin(db: &SqlitePool, id: &str) -> Result<PluginDefRow> {
    sqlx::query_as::<_, PluginDefRow>("SELECT * FROM plugin_defs WHERE id = ? LIMIT 1")
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::NotFound {
            entity: "plugin",
            id: id.to_string(),
        })
}

pub async fn create_plugin(
    db: &SqlitePool,
    input: &CreatePluginRecord<'_>,
) -> Result<PluginDefRow> {
    let id = ids::new_id();
    let now = time::now_ms();
    sqlx::query(
        r#"
        INSERT INTO plugin_defs (
            id, name, plugin_key, version, runtime_kind, entrypoint, capabilities_json,
            permissions_json, enabled, sort_order, config_json, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(input.name)
    .bind(input.plugin_key)
    .bind(input.version)
    .bind(input.runtime_kind)
    .bind(input.entrypoint)
    .bind(input.capabilities_json)
    .bind(input.permissions_json)
    .bind(input.enabled)
    .bind(input.sort_order)
    .bind(input.config_json)
    .bind(now)
    .bind(now)
    .execute(db)
    .await?;

    get_plugin(db, &id).await
}

pub async fn update_plugin(
    db: &SqlitePool,
    id: &str,
    input: &UpdatePluginRecord<'_>,
) -> Result<PluginDefRow> {
    let affected = sqlx::query(
        r#"
        UPDATE plugin_defs
        SET name = ?, version = ?, runtime_kind = ?, entrypoint = ?, capabilities_json = ?,
            permissions_json = ?, enabled = ?, sort_order = ?, config_json = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(input.name)
    .bind(input.version)
    .bind(input.runtime_kind)
    .bind(input.entrypoint)
    .bind(input.capabilities_json)
    .bind(input.permissions_json)
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
            entity: "plugin",
            id: id.to_string(),
        });
    }

    get_plugin(db, id).await
}

pub async fn delete_plugin(db: &SqlitePool, id: &str) -> Result<()> {
    let affected = sqlx::query("DELETE FROM plugin_defs WHERE id = ?")
        .bind(id)
        .execute(db)
        .await?
        .rows_affected();
    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "plugin",
            id: id.to_string(),
        });
    }
    Ok(())
}

pub async fn list_plugins_by_capability(
    db: &SqlitePool,
    capability: &str,
) -> Result<Vec<PluginDefRow>> {
    let rows = list_plugins(db).await?;
    let mut matched = Vec::new();
    for row in rows {
        let capabilities_json = serde_json::from_str::<serde_json::Value>(&row.capabilities_json)?;
        if json_has_capability(&capabilities_json, capability) {
            matched.push(row);
        }
    }
    Ok(matched)
}
