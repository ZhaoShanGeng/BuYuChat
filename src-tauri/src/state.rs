//! Tauri 异步命令共享的运行时状态。

use std::{path::PathBuf, str::FromStr, sync::Arc, time::Duration};

use dashmap::DashMap;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    SqlitePool,
};
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;
use tauri::Manager;

use crate::{mcp::tools::ToolRegistry, repo::migrations::MIGRATOR};

/// 通过 `State<AppState>` 注入命令层的共享应用状态。
#[derive(Debug, Clone)]
pub struct AppState {
    /// SQLite 连接池。
    pub db: SqlitePool,
    /// 复用的 HTTP 客户端。
    pub http_client: reqwest::Client,
    /// 按版本 ID 保存的生成取消令牌。
    pub cancellation_tokens: Arc<DashMap<String, CancellationToken>>,
    /// 生成任务并发上限控制器。
    pub generation_semaphore: Arc<Semaphore>,
    /// 内置工具注册表。
    pub tool_registry: Arc<ToolRegistry>,
}

impl AppState {
    /// 初始化默认应用状态。
    pub async fn initialize() -> anyhow::Result<Self> {
        Self::initialize_with_url("sqlite://buyu.db").await
    }

    /// 使用应用数据目录中的 SQLite 数据库初始化状态。
    pub async fn initialize_for_app(app: &tauri::App) -> anyhow::Result<Self> {
        let database_path = app_database_path(app)?;
        Self::initialize_with_path(database_path).await
    }

    /// 使用指定文件路径初始化 SQLite 状态。
    pub async fn initialize_with_path(path: PathBuf) -> anyhow::Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .busy_timeout(Duration::from_secs(5))
            .foreign_keys(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal);

        Self::initialize_with_options(options, 5).await
    }

    /// 使用指定数据库地址初始化应用状态。
    pub async fn initialize_with_url(database_url: &str) -> anyhow::Result<Self> {
        let options = build_connect_options(database_url, !database_url.contains(":memory:"))?;
        let max_connections = if database_url.contains(":memory:") { 1 } else { 5 };
        Self::initialize_with_options(options, max_connections).await
    }

    async fn initialize_with_options(
        options: SqliteConnectOptions,
        max_connections: u32,
    ) -> anyhow::Result<Self> {
        let db = SqlitePoolOptions::new()
            .max_connections(max_connections)
            .connect_with(options)
            .await?;
        MIGRATOR.run(&db).await?;
        sqlx::query("UPDATE message_versions SET status = 'failed' WHERE status = 'generating'")
            .execute(&db)
            .await?;

        Ok(Self {
            db,
            http_client: build_http_client()?,
            cancellation_tokens: Arc::new(DashMap::new()),
            generation_semaphore: Arc::new(Semaphore::new(5)),
            tool_registry: Arc::new(ToolRegistry::new()),
        })
    }
}

fn app_database_path(app: &tauri::App) -> anyhow::Result<PathBuf> {
    let base_dir = app
        .path()
        .app_local_data_dir()
        .or_else(|_| app.path().app_data_dir())?;
    Ok(base_dir.join("buyu.db"))
}

fn build_http_client() -> anyhow::Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .pool_idle_timeout(Duration::from_secs(90))
        .build()?)
}

/// 构建符合架构文档要求的 SQLite 连接选项。
pub fn build_connect_options(
    database_url: &str,
    create_if_missing: bool,
) -> anyhow::Result<SqliteConnectOptions> {
    Ok(SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(create_if_missing)
        .busy_timeout(Duration::from_secs(5))
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal))
}
