use sqlx::SqlitePool;

use crate::db::{message, models::MessageRow};
use crate::error::{AppError, Result};

pub struct VersioningService {
    db: SqlitePool,
}

impl VersioningService {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    pub async fn switch_version(
        &self,
        version_group_id: &str,
        target_index: i64,
    ) -> Result<MessageRow> {
        let max_index = message::max_version_index(&self.db, version_group_id).await?;
        if target_index < 1 || target_index > max_index {
            return Err(AppError::Other(format!(
                "target version {target_index} is out of range 1..={max_index}"
            )));
        }

        let target = message::get_version(&self.db, version_group_id, target_index).await?;

        let mut tx = self.db.begin().await?;
        message::set_active_message_tx(&mut tx, version_group_id, &target.id).await?;
        tx.commit().await?;
        message::get(&self.db, &target.id).await
    }
}
