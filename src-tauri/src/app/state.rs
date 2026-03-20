use std::sync::Arc;

use sqlx::SqlitePool;

use crate::extensions::runtime::PluginRuntime;
use crate::providers::ProviderRegistry;
use crate::services::content_store::ContentStore;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub content_store: Arc<ContentStore>,
    pub plugin_runtime: Arc<PluginRuntime>,
    pub provider_registry: Arc<ProviderRegistry>,
    pub local_profile_key: String,
}

impl AppState {
    pub fn new(
        db: SqlitePool,
        content_store: Arc<ContentStore>,
        plugin_runtime: Arc<PluginRuntime>,
        provider_registry: Arc<ProviderRegistry>,
        local_profile_key: String,
    ) -> Self {
        Self {
            db,
            content_store,
            plugin_runtime,
            provider_registry,
            local_profile_key,
        }
    }
}
