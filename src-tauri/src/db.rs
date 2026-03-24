use std::sync::{Arc, Mutex};

use anyhow::Context;
use rusqlite::Connection;

pub type Db = Arc<Mutex<Connection>>;
const INITIAL_MIGRATION: &str = include_str!("../migrations/20260324000000_init.sql");

pub fn init_db() -> anyhow::Result<Db> {
    let conn = Connection::open_in_memory().context("open sqlite database")?;
    migrate(&conn)?;
    Ok(Arc::new(Mutex::new(conn)))
}

pub fn migrate(conn: &Connection) -> anyhow::Result<()> {
    conn.execute_batch(INITIAL_MIGRATION)
        .context("run schema migration")
}
