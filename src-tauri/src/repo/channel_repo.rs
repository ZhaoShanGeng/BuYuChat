//! `api_channels` 表的仓储访问实现。

use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::models::{Channel, ChannelPatch, NewChannel};

/// 渠道服务使用的异步仓储契约。
#[async_trait]
pub trait ChannelRepo: Send + Sync {
    /// 插入新渠道并返回持久化结果。
    async fn insert(&self, new_channel: &NewChannel) -> Result<Channel, String>;
    /// 按创建时间倒序列出渠道。
    async fn list(&self, include_disabled: bool) -> Result<Vec<Channel>, String>;
    /// 按 ID 获取单个渠道。
    async fn get(&self, id: &str) -> Result<Option<Channel>, String>;
    /// 对渠道执行部分更新。
    async fn update(&self, id: &str, patch: &ChannelPatch) -> Result<Option<Channel>, String>;
    /// 按 ID 删除渠道。
    async fn delete(&self, id: &str) -> Result<bool, String>;
}

/// 基于 SQLx 的仓储实现。
#[derive(Debug, Clone)]
pub struct SqlxChannelRepo {
    pool: SqlitePool,
}

impl SqlxChannelRepo {
    /// 使用指定连接池创建仓储实例。
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ChannelRepo for SqlxChannelRepo {
    async fn insert(&self, new_channel: &NewChannel) -> Result<Channel, String> {
        sqlx::query(
            r#"
            INSERT INTO api_channels (
                id, name, channel_type, base_url, api_key, auth_type,
                api_keys, models_endpoint, chat_endpoint, stream_endpoint, thinking_tags, enabled, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&new_channel.id)
        .bind(&new_channel.name)
        .bind(&new_channel.channel_type)
        .bind(&new_channel.base_url)
        .bind(&new_channel.api_key)
        .bind(&new_channel.auth_type)
        .bind(&new_channel.api_keys)
        .bind(&new_channel.models_endpoint)
        .bind(&new_channel.chat_endpoint)
        .bind(&new_channel.stream_endpoint)
        .bind(&new_channel.thinking_tags)
        .bind(new_channel.enabled)
        .bind(new_channel.created_at)
        .bind(new_channel.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|error| error.to_string())?;

        self.get(&new_channel.id)
            .await?
            .ok_or_else(|| "inserted channel missing after insert".to_string())
    }

    async fn list(&self, include_disabled: bool) -> Result<Vec<Channel>, String> {
        let query = if include_disabled {
            "SELECT * FROM api_channels ORDER BY created_at DESC, id DESC"
        } else {
            "SELECT * FROM api_channels WHERE enabled = 1 ORDER BY created_at DESC, id DESC"
        };

        sqlx::query_as::<_, Channel>(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|error| error.to_string())
    }

    async fn get(&self, id: &str) -> Result<Option<Channel>, String> {
        sqlx::query_as::<_, Channel>("SELECT * FROM api_channels WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| error.to_string())
    }

    async fn update(&self, id: &str, patch: &ChannelPatch) -> Result<Option<Channel>, String> {
        let result = sqlx::query(
            r#"
            UPDATE api_channels
            SET
                name = COALESCE(?1, name),
                base_url = COALESCE(?2, base_url),
                channel_type = COALESCE(?3, channel_type),
                api_key = COALESCE(?4, api_key),
                api_keys = COALESCE(?5, api_keys),
                auth_type = COALESCE(?6, auth_type),
                models_endpoint = COALESCE(?7, models_endpoint),
                chat_endpoint = COALESCE(?8, chat_endpoint),
                stream_endpoint = COALESCE(?9, stream_endpoint),
                thinking_tags = COALESCE(?10, thinking_tags),
                enabled = COALESCE(?11, enabled),
                updated_at = ?12
            WHERE id = ?13
            "#,
        )
        .bind(&patch.name)
        .bind(&patch.base_url)
        .bind(&patch.channel_type)
        .bind(&patch.api_key)
        .bind(&patch.api_keys)
        .bind(&patch.auth_type)
        .bind(&patch.models_endpoint)
        .bind(&patch.chat_endpoint)
        .bind(&patch.stream_endpoint)
        .bind(&patch.thinking_tags)
        .bind(patch.enabled)
        .bind(patch.updated_at)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|error| error.to_string())?;

        if result.rows_affected() == 0 {
            return Ok(None);
        }

        self.get(id).await
    }

    async fn delete(&self, id: &str) -> Result<bool, String> {
        sqlx::query("DELETE FROM api_channels WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map(|result| result.rows_affected() > 0)
            .map_err(|error| error.to_string())
    }
}
