//! Tauri 异步命令共享的运行时状态。

use std::str::FromStr;

use dashmap::DashMap;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    SqlitePool,
};
use tokio_util::sync::CancellationToken;

use crate::repo::migrations::MIGRATOR;

/// 通过 `State<AppState>` 注入命令层的共享应用状态。
#[derive(Debug)]
pub struct AppState {
    /// SQLite 连接池。
    pub db: SqlitePool,
    /// 复用的 HTTP 客户端。
    pub http_client: reqwest::Client,
    /// 按版本 ID 保存的生成取消令牌。
    pub cancellation_tokens: DashMap<String, CancellationToken>,
}

impl AppState {
    /// 初始化默认应用状态。
    pub async fn initialize() -> anyhow::Result<Self> {
        Self::initialize_with_url("sqlite://buyu.db").await
    }

    /// 使用指定数据库地址初始化应用状态。
    pub async fn initialize_with_url(database_url: &str) -> anyhow::Result<Self> {
        let options = build_connect_options(database_url, !database_url.contains(":memory:"))?;
        let db = SqlitePoolOptions::new()
            .max_connections(if database_url.contains(":memory:") {
                1
            } else {
                5
            })
            .connect_with(options)
            .await?;

        MIGRATOR.run(&db).await?;

        Ok(Self {
            db,
            http_client: reqwest::Client::new(),
            cancellation_tokens: DashMap::new(),
        })
    }
}

/// 构建符合架构文档要求的 SQLite 连接选项。
pub fn build_connect_options(
    database_url: &str,
    create_if_missing: bool,
) -> anyhow::Result<SqliteConnectOptions> {
    Ok(SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(create_if_missing)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal))
}
