use std::sync::Arc;

use sqlx::SqlitePool;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub providers: Arc<crate::providers::ProviderRegistry>,
    pub keyring: Arc<crate::services::keyring::KeyringService>,
}

impl AppState {
    pub fn new(
        db: SqlitePool,
        providers: Arc<crate::providers::ProviderRegistry>,
        keyring: Arc<crate::services::keyring::KeyringService>,
    ) -> Self {
        Self {
            db,
            providers,
            keyring,
        }
    }
}
