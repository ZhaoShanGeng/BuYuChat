//! `api_channel_models` 表的仓储访问实现。

use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::models::{ChannelModel, ChannelModelPatch, NewChannelModel};

/// 模型服务使用的异步仓储契约。
#[async_trait]
pub trait ModelRepo: Send + Sync {
    /// 判断渠道是否存在。
    async fn channel_exists(&self, channel_id: &str) -> Result<bool, String>;
    /// 判断同渠道下是否已存在相同的 model_id。
    async fn model_id_exists(&self, channel_id: &str, model_id: &str) -> Result<bool, String>;
    /// 插入新模型并返回持久化结果。
    async fn insert(&self, new_model: &NewChannelModel) -> Result<ChannelModel, String>;
    /// 按渠道列出模型列表。
    async fn list_by_channel(&self, channel_id: &str) -> Result<Vec<ChannelModel>, String>;
    /// 按渠道与模型记录 ID 获取单个模型。
    async fn get_by_channel_and_id(
        &self,
        channel_id: &str,
        id: &str,
    ) -> Result<Option<ChannelModel>, String>;
    /// 对指定模型执行部分更新。
    async fn update(
        &self,
        channel_id: &str,
        id: &str,
        patch: &ChannelModelPatch,
    ) -> Result<Option<ChannelModel>, String>;
    /// 按渠道与模型记录 ID 删除模型。
    async fn delete(&self, channel_id: &str, id: &str) -> Result<bool, String>;
}

/// 基于 SQLx 的模型仓储实现。
#[derive(Debug, Clone)]
pub struct SqlxModelRepo {
    pool: SqlitePool,
}

impl SqlxModelRepo {
    /// 使用指定连接池创建模型仓储实例。
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ModelRepo for SqlxModelRepo {
    async fn channel_exists(&self, channel_id: &str) -> Result<bool, String> {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM api_channels WHERE id = ?1")
            .bind(channel_id)
            .fetch_one(&self.pool)
            .await
            .map(|count| count > 0)
            .map_err(|error| error.to_string())
    }

    async fn model_id_exists(&self, channel_id: &str, model_id: &str) -> Result<bool, String> {
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM api_channel_models WHERE channel_id = ?1 AND model_id = ?2",
        )
        .bind(channel_id)
        .bind(model_id)
        .fetch_one(&self.pool)
        .await
        .map(|count| count > 0)
        .map_err(|error| error.to_string())
    }

    async fn insert(&self, new_model: &NewChannelModel) -> Result<ChannelModel, String> {
        sqlx::query(
            r#"
            INSERT INTO api_channel_models (
                id, channel_id, model_id, display_name, context_window, max_output_tokens, temperature, top_p
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
        )
        .bind(&new_model.id)
        .bind(&new_model.channel_id)
        .bind(&new_model.model_id)
        .bind(&new_model.display_name)
        .bind(new_model.context_window)
        .bind(new_model.max_output_tokens)
        .bind(&new_model.temperature)
        .bind(&new_model.top_p)
        .execute(&self.pool)
        .await
        .map_err(|error| error.to_string())?;

        self.get_by_channel_and_id(&new_model.channel_id, &new_model.id)
            .await?
            .ok_or_else(|| "inserted model missing after insert".to_string())
    }

    async fn list_by_channel(&self, channel_id: &str) -> Result<Vec<ChannelModel>, String> {
        sqlx::query_as::<_, ChannelModel>(
            r#"
            SELECT * FROM api_channel_models
            WHERE channel_id = ?1
            ORDER BY model_id ASC, id ASC
            "#,
        )
        .bind(channel_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| error.to_string())
    }

    async fn get_by_channel_and_id(
        &self,
        channel_id: &str,
        id: &str,
    ) -> Result<Option<ChannelModel>, String> {
        sqlx::query_as::<_, ChannelModel>(
            "SELECT * FROM api_channel_models WHERE channel_id = ?1 AND id = ?2",
        )
        .bind(channel_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| error.to_string())
    }

    async fn update(
        &self,
        channel_id: &str,
        id: &str,
        patch: &ChannelModelPatch,
    ) -> Result<Option<ChannelModel>, String> {
        let display_name_provided = patch.display_name.is_some();
        let context_window_provided = patch.context_window.is_some();
        let max_output_tokens_provided = patch.max_output_tokens.is_some();
        let temperature_provided = patch.temperature.is_some();
        let top_p_provided = patch.top_p.is_some();

        let result = sqlx::query(
            r#"
            UPDATE api_channel_models
            SET
                display_name = CASE WHEN ?1 THEN ?2 ELSE display_name END,
                context_window = CASE WHEN ?3 THEN ?4 ELSE context_window END,
                max_output_tokens = CASE WHEN ?5 THEN ?6 ELSE max_output_tokens END,
                temperature = CASE WHEN ?7 THEN ?8 ELSE temperature END,
                top_p = CASE WHEN ?9 THEN ?10 ELSE top_p END
            WHERE channel_id = ?11 AND id = ?12
            "#,
        )
        .bind(display_name_provided)
        .bind(patch.display_name.as_ref().cloned().flatten())
        .bind(context_window_provided)
        .bind(patch.context_window.flatten())
        .bind(max_output_tokens_provided)
        .bind(patch.max_output_tokens.flatten())
        .bind(temperature_provided)
        .bind(patch.temperature.as_ref().cloned().flatten())
        .bind(top_p_provided)
        .bind(patch.top_p.as_ref().cloned().flatten())
        .bind(channel_id)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|error| error.to_string())?;

        if result.rows_affected() == 0 {
            return Ok(None);
        }

        self.get_by_channel_and_id(channel_id, id).await
    }

    async fn delete(&self, channel_id: &str, id: &str) -> Result<bool, String> {
        sqlx::query("DELETE FROM api_channel_models WHERE channel_id = ?1 AND id = ?2")
            .bind(channel_id)
            .bind(id)
            .execute(&self.pool)
            .await
            .map(|result| result.rows_affected() > 0)
            .map_err(|error| error.to_string())
    }
}
