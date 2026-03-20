use std::path::Path;

use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    SqlitePool,
};

use crate::support::error::Result;

pub async fn init_pool(db_path: impl AsRef<Path>) -> Result<SqlitePool> {
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

    Ok(pool)
}
