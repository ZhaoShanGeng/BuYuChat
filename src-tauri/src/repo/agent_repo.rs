//! `agents` 表的仓储访问实现。

use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::models::{Agent, AgentPatch, NewAgent};

/// Agent 服务使用的异步仓储契约。
#[async_trait]
pub trait AgentRepo: Send + Sync {
    /// 插入新 Agent 并返回持久化结果。
    async fn insert(&self, new_agent: &NewAgent) -> Result<Agent, String>;
    /// 按创建时间倒序列出 Agent。
    async fn list(&self, include_disabled: bool) -> Result<Vec<Agent>, String>;
    /// 按 ID 获取单个 Agent。
    async fn get(&self, id: &str) -> Result<Option<Agent>, String>;
    /// 对 Agent 执行部分更新。
    async fn update(&self, id: &str, patch: &AgentPatch) -> Result<Option<Agent>, String>;
    /// 按 ID 删除 Agent。
    async fn delete(&self, id: &str) -> Result<bool, String>;
}

/// 基于 SQLx 的 Agent 仓储实现。
#[derive(Debug, Clone)]
pub struct SqlxAgentRepo {
    pool: SqlitePool,
}

impl SqlxAgentRepo {
    /// 使用指定连接池创建 Agent 仓储实例。
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AgentRepo for SqlxAgentRepo {
    async fn insert(&self, new_agent: &NewAgent) -> Result<Agent, String> {
        sqlx::query(
            r#"
            INSERT INTO agents (
                id, name, system_prompt, avatar_uri, enabled, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
        )
        .bind(&new_agent.id)
        .bind(&new_agent.name)
        .bind(&new_agent.system_prompt)
        .bind(&new_agent.avatar_uri)
        .bind(new_agent.enabled)
        .bind(new_agent.created_at)
        .bind(new_agent.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|error| error.to_string())?;

        self.get(&new_agent.id)
            .await?
            .ok_or_else(|| "inserted agent missing after insert".to_string())
    }

    async fn list(&self, include_disabled: bool) -> Result<Vec<Agent>, String> {
        let query = if include_disabled {
            "SELECT * FROM agents ORDER BY created_at DESC, id DESC"
        } else {
            "SELECT * FROM agents WHERE enabled = 1 ORDER BY created_at DESC, id DESC"
        };

        sqlx::query_as::<_, Agent>(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|error| error.to_string())
    }

    async fn get(&self, id: &str) -> Result<Option<Agent>, String> {
        sqlx::query_as::<_, Agent>("SELECT * FROM agents WHERE id = ?1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| error.to_string())
    }

    async fn update(&self, id: &str, patch: &AgentPatch) -> Result<Option<Agent>, String> {
        let system_prompt_provided = patch.system_prompt.is_some();

        let result = sqlx::query(
            r#"
            UPDATE agents
            SET
                name = COALESCE(?1, name),
                system_prompt = CASE WHEN ?2 THEN ?3 ELSE system_prompt END,
                enabled = COALESCE(?4, enabled),
                updated_at = ?5
            WHERE id = ?6
            "#,
        )
        .bind(&patch.name)
        .bind(system_prompt_provided)
        .bind(patch.system_prompt.clone().flatten())
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
        sqlx::query("DELETE FROM agents WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map(|result| result.rows_affected() > 0)
            .map_err(|error| error.to_string())
    }
}
