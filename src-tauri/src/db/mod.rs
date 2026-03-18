pub mod conversation;
pub mod custom_channel;
pub mod message;
pub mod models;
pub mod provider_config;

use std::path::Path;

use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    SqlitePool,
};

pub async fn init_pool(db_path: impl AsRef<Path>) -> crate::error::Result<SqlitePool> {
    let options = SqliteConnectOptions::new()
        .filename(db_path.as_ref())
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    sqlx::query("PRAGMA foreign_keys=ON").execute(&pool).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    message::backfill_turns_from_legacy_messages(&pool).await?;

    Ok(pool)
}
