use std::sync::Arc;

use tauri::{AppHandle, Manager};

use crate::app::state::AppState;
use crate::extensions::runtime::PluginRuntime;
use crate::providers::ProviderRegistry;
use crate::services::content_store::ContentStore;
use crate::support::error::{AppError, Result};

pub async fn build_state(app_handle: AppHandle) -> Result<AppState> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|err| AppError::Other(format!("failed to resolve app data directory: {err}")))?;

    std::fs::create_dir_all(&app_data_dir)?;

    let db_path = app_data_dir.join("buyu.db");
    let db = crate::db::pool::init_pool(&db_path).await?;

    let local_profile_key = "default".to_string();
    let content_store = Arc::new(ContentStore::new(
        app_data_dir
            .join("storage")
            .join("profiles")
            .join(&local_profile_key),
    ));
    content_store.ensure_layout()?;

    let plugin_runtime = Arc::new(PluginRuntime::new());
    let provider_registry = Arc::new(ProviderRegistry::with_defaults());
    crate::services::plugins::sync_runtime(&db, plugin_runtime.as_ref()).await?;

    Ok(AppState::new(
        db,
        content_store,
        plugin_runtime,
        provider_registry,
        local_profile_key,
    ))
}
