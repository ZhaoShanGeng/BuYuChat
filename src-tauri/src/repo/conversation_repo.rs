//! `conversations` 表的仓储访问实现。

use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::models::{
    Conversation, ConversationPatch, ConversationSummary, NewConversation,
};

/// 会话服务使用的异步仓储契约。
#[async_trait]
pub trait ConversationRepo: Send + Sync {
    /// 插入新会话并返回持久化结果。
    async fn insert(&self, new_conversation: &NewConversation) -> Result<Conversation, String>;
    /// 按归档状态列出会话列表项。
    async fn list(&self, archived: bool) -> Result<Vec<ConversationSummary>, String>;
    /// 按 ID 获取单个会话。
    async fn get(&self, id: &str) -> Result<Option<Conversation>, String>;
    /// 对会话执行部分更新。
    async fn update(
        &self,
        id: &str,
        patch: &ConversationPatch,
    ) -> Result<Option<Conversation>, String>;
    /// 按 ID 删除会话。
    async fn delete(&self, id: &str) -> Result<bool, String>;
    /// 更新会话的 `updated_at`。
    async fn touch_updated_at(&self, id: &str, updated_at: i64) -> Result<bool, String>;
}

/// 基于 SQLx 的会话仓储实现。
#[derive(Debug, Clone)]
pub struct SqlxConversationRepo {
    pool: SqlitePool,
}

impl SqlxConversationRepo {
    /// 使用指定连接池创建会话仓储实例。
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ConversationRepo for SqlxConversationRepo {
    async fn insert(&self, new_conversation: &NewConversation) -> Result<Conversation, String> {
        sqlx::query(
            r#"
            INSERT INTO conversations (
                id, title, agent_id, channel_id, channel_model_id,
                archived, pinned, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
        )
        .bind(&new_conversation.id)
        .bind(&new_conversation.title)
        .bind(&new_conversation.agent_id)
        .bind(&new_conversation.channel_id)
        .bind(&new_conversation.channel_model_id)
        .bind(new_conversation.archived)
        .bind(new_conversation.pinned)
        .bind(new_conversation.created_at)
        .bind(new_conversation.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|error| error.to_string())?;

        self.get(&new_conversation.id)
            .await?
            .ok_or_else(|| "inserted conversation missing after insert".to_string())
    }

    async fn list(&self, archived: bool) -> Result<Vec<ConversationSummary>, String> {
        sqlx::query_as::<_, ConversationSummary>(
            r#"
            SELECT
                id, title, agent_id, channel_id, channel_model_id,
                archived, pinned, updated_at
            FROM conversations
            WHERE archived = ?1
            ORDER BY pinned DESC, updated_at DESC, id DESC
            "#,
        )
        .bind(archived)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| error.to_string())
    }

    async fn get(&self, id: &str) -> Result<Option<Conversation>, String> {
        sqlx::query_as::<_, Conversation>("SELECT * FROM conversations WHERE id = ?1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| error.to_string())
    }

    async fn update(
        &self,
        id: &str,
        patch: &ConversationPatch,
    ) -> Result<Option<Conversation>, String> {
        let agent_id_provided = patch.agent_id.is_some();
        let channel_id_provided = patch.channel_id.is_some();
        let channel_model_id_provided = patch.channel_model_id.is_some();

        let result = sqlx::query(
            r#"
            UPDATE conversations
            SET
                title = COALESCE(?1, title),
                agent_id = CASE WHEN ?2 THEN ?3 ELSE agent_id END,
                channel_id = CASE WHEN ?4 THEN ?5 ELSE channel_id END,
                channel_model_id = CASE WHEN ?6 THEN ?7 ELSE channel_model_id END,
                archived = COALESCE(?8, archived),
                pinned = COALESCE(?9, pinned),
                updated_at = ?10
            WHERE id = ?11
            "#,
        )
        .bind(&patch.title)
        .bind(agent_id_provided)
        .bind(patch.agent_id.clone().flatten())
        .bind(channel_id_provided)
        .bind(patch.channel_id.clone().flatten())
        .bind(channel_model_id_provided)
        .bind(patch.channel_model_id.clone().flatten())
        .bind(patch.archived)
        .bind(patch.pinned)
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
        sqlx::query("DELETE FROM conversations WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map(|result| result.rows_affected() > 0)
            .map_err(|error| error.to_string())
    }

    async fn touch_updated_at(&self, id: &str, updated_at: i64) -> Result<bool, String> {
        sqlx::query("UPDATE conversations SET updated_at = ?1 WHERE id = ?2")
            .bind(updated_at)
            .bind(id)
            .execute(&self.pool)
            .await
            .map(|result| result.rows_affected() > 0)
            .map_err(|error| error.to_string())
    }
}
