use sqlx::SqlitePool;

use crate::db::models::PluginDefRow;
use crate::db::repos::plugins as repo;
use crate::domain::plugins::{CreatePluginInput, PluginDef, UpdatePluginInput};
use crate::extensions::runtime::PluginRuntime;
use crate::support::error::{AppError, Result};

pub async fn list_plugins(db: &SqlitePool) -> Result<Vec<PluginDef>> {
    repo::list_plugins(db)
        .await?
        .into_iter()
        .map(map_plugin_row)
        .collect()
}

pub async fn list_plugins_by_capability(
    db: &SqlitePool,
    capability: &str,
) -> Result<Vec<PluginDef>> {
    repo::list_plugins_by_capability(db, capability)
        .await?
        .into_iter()
        .map(map_plugin_row)
        .collect()
}

pub async fn get_plugin(db: &SqlitePool, id: &str) -> Result<PluginDef> {
    map_plugin_row(repo::get_plugin(db, id).await?)
}

pub async fn create_plugin(
    db: &SqlitePool,
    runtime: &PluginRuntime,
    input: &CreatePluginInput,
) -> Result<PluginDef> {
    let plugin = map_plugin_row(
        repo::create_plugin(
            db,
            &repo::CreatePluginRecord {
                name: &input.name,
                plugin_key: &input.plugin_key,
                version: &input.version,
                runtime_kind: &input.runtime_kind,
                entrypoint: input.entrypoint.as_deref(),
                enabled: input.enabled,
                sort_order: input.sort_order,
                capabilities_json: &input.capabilities_json.to_string(),
                permissions_json: &input.permissions_json.to_string(),
                config_json: &input.config_json.to_string(),
            },
        )
        .await?,
    )?;
    runtime.register_plugin(plugin.clone());
    Ok(plugin)
}

pub async fn update_plugin(
    db: &SqlitePool,
    runtime: &PluginRuntime,
    id: &str,
    input: &UpdatePluginInput,
) -> Result<PluginDef> {
    let plugin = map_plugin_row(
        repo::update_plugin(
            db,
            id,
            &repo::UpdatePluginRecord {
                name: &input.name,
                version: &input.version,
                runtime_kind: &input.runtime_kind,
                entrypoint: input.entrypoint.as_deref(),
                enabled: input.enabled,
                sort_order: input.sort_order,
                capabilities_json: &input.capabilities_json.to_string(),
                permissions_json: &input.permissions_json.to_string(),
                config_json: &input.config_json.to_string(),
            },
        )
        .await?,
    )?;
    runtime.register_plugin(plugin.clone());
    Ok(plugin)
}

pub async fn delete_plugin(db: &SqlitePool, runtime: &PluginRuntime, id: &str) -> Result<()> {
    repo::delete_plugin(db, id).await?;
    runtime.unregister_plugin(id);
    Ok(())
}

pub async fn sync_runtime(db: &SqlitePool, runtime: &PluginRuntime) -> Result<()> {
    let plugins = list_plugins(db).await?;
    runtime.replace_all(plugins);
    Ok(())
}

fn map_plugin_row(row: PluginDefRow) -> Result<PluginDef> {
    Ok(PluginDef {
        id: row.id,
        name: row.name,
        plugin_key: row.plugin_key,
        version: row.version,
        runtime_kind: row.runtime_kind,
        entrypoint: row.entrypoint,
        enabled: row.enabled,
        sort_order: row.sort_order,
        capabilities_json: parse_json(&row.capabilities_json, "plugin_defs.capabilities_json")?,
        permissions_json: parse_json(&row.permissions_json, "plugin_defs.permissions_json")?,
        config_json: parse_json(&row.config_json, "plugin_defs.config_json")?,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

fn parse_json(raw: &str, field: &'static str) -> Result<serde_json::Value> {
    serde_json::from_str(raw)
        .map_err(|err| AppError::Validation(format!("failed to parse {field} as json: {err}")))
}
