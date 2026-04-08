//! Tauri 异步命令共享的运行时状态。

use std::{fs, path::{Path, PathBuf}, str::FromStr, sync::Arc, time::Duration};

use dashmap::DashMap;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    Row, SqlitePool,
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
        if let Some(source) = migrate_legacy_database_if_needed(app, &database_path).await? {
            eprintln!(
                "migrated legacy database from {} to {}",
                source.display(),
                database_path.display()
            );
        }
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

const LEGACY_APP_DIR_NAMES: &[&str] = &["com.buyu.app", "BuYu", "buyu"];
const DATABASE_FAMILY_SUFFIXES: &[&str] = &["", "-wal", "-shm"];

async fn migrate_legacy_database_if_needed(
    app: &tauri::App,
    target_database_path: &Path,
) -> anyhow::Result<Option<PathBuf>> {
    let legacy_candidates = legacy_database_candidates(app, target_database_path);
    if legacy_candidates.is_empty() {
        return Ok(None);
    }

    let current_conversation_count = database_conversation_count(target_database_path).await?;
    let should_migrate = match current_conversation_count {
        None => true,
        Some(count) => count == 0,
    };

    if !should_migrate {
        return Ok(None);
    }

    for source_database_path in legacy_candidates {
        let source_conversation_count = database_conversation_count(&source_database_path).await?;
        if source_conversation_count.unwrap_or(0) <= 0 {
            continue;
        }

        replace_database_family(target_database_path, &source_database_path)?;
        return Ok(Some(source_database_path));
    }

    Ok(None)
}

fn legacy_database_candidates(app: &tauri::App, target_database_path: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    let mut base_dirs = Vec::new();

    if let Ok(path) = app.path().app_local_data_dir() {
        base_dirs.push(path);
    }
    if let Ok(path) = app.path().app_data_dir() {
        if !base_dirs.iter().any(|existing| existing == &path) {
            base_dirs.push(path);
        }
    }

    for base_dir in base_dirs {
        let Some(parent) = base_dir.parent() else {
            continue;
        };

        for legacy_name in LEGACY_APP_DIR_NAMES {
            let candidate = parent.join(legacy_name).join("buyu.db");
            if candidate != target_database_path
                && !candidates.iter().any(|existing| existing == &candidate)
            {
                candidates.push(candidate);
            }
        }
    }

    candidates
}

async fn database_conversation_count(path: &Path) -> anyhow::Result<Option<i64>> {
    if !path.exists() {
        return Ok(None);
    }

    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(false)
        .busy_timeout(Duration::from_secs(5))
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal);

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await?;

    let has_conversation_table = sqlx::query(
        "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'conversations' LIMIT 1",
    )
    .fetch_optional(&pool)
    .await?
    .is_some();

    let count = if has_conversation_table {
        sqlx::query("SELECT COUNT(*) AS count FROM conversations")
            .fetch_one(&pool)
            .await?
            .try_get::<i64, _>("count")?
    } else {
        0
    };

    pool.close().await;
    Ok(Some(count))
}

fn replace_database_family(target_database_path: &Path, source_database_path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = target_database_path.parent() {
        fs::create_dir_all(parent)?;
    }

    for suffix in DATABASE_FAMILY_SUFFIXES {
        let target = database_family_path(target_database_path, suffix);
        if target.exists() {
            fs::remove_file(&target)?;
        }
    }

    for suffix in DATABASE_FAMILY_SUFFIXES {
        let source = database_family_path(source_database_path, suffix);
        if source.exists() {
            let target = database_family_path(target_database_path, suffix);
            fs::copy(source, target)?;
        }
    }

    Ok(())
}

fn database_family_path(path: &Path, suffix: &str) -> PathBuf {
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "buyu.db".to_string());
    path.with_file_name(format!("{file_name}{suffix}"))
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

#[cfg(test)]
mod tests {
    use super::{database_family_path, replace_database_family};
    use std::{fs, path::PathBuf};
    use uuid::Uuid;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("buyu-{name}-{}", Uuid::now_v7()));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn replace_database_family_copies_db_and_sidecars() {
        let root = temp_dir("db-migrate");
        let source = root.join("legacy").join("buyu.db");
        let target = root.join("current").join("buyu.db");

        fs::create_dir_all(source.parent().expect("source parent")).expect("source dir");
        fs::write(&source, b"db").expect("write db");
        fs::write(database_family_path(&source, "-wal"), b"wal").expect("write wal");
        fs::write(database_family_path(&source, "-shm"), b"shm").expect("write shm");

        replace_database_family(&target, &source).expect("replace family");

        assert_eq!(fs::read(&target).expect("read target db"), b"db");
        assert_eq!(
            fs::read(database_family_path(&target, "-wal")).expect("read target wal"),
            b"wal"
        );
        assert_eq!(
            fs::read(database_family_path(&target, "-shm")).expect("read target shm"),
            b"shm"
        );

        fs::remove_dir_all(root).expect("cleanup");
    }
}
