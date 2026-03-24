//! 内嵌的 SQLx 迁移器定义。

use sqlx::migrate::Migrator;

/// 基于 `migrations/` 目录构建的迁移器。
pub static MIGRATOR: Migrator = sqlx::migrate!("./migrations");
