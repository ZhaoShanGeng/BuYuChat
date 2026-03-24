//! 模型远程拉取的业务编排逻辑。

use sqlx::SqlitePool;

use crate::{
    ai::adapter::{AiAdapter, AiChannelConfig, AiMetadataClient},
    error::AppError,
    models::RemoteModelInfo,
    repo::channel_repo::{ChannelRepo, SqlxChannelRepo},
    services::channel_service::get_with,
};

/// 使用连接池和共享 HTTP 客户端从远程拉取模型列表。
pub async fn fetch_remote_models(
    pool: &SqlitePool,
    http_client: &reqwest::Client,
    channel_id: &str,
) -> Result<Vec<RemoteModelInfo>, AppError> {
    let repo = SqlxChannelRepo::new(pool.clone());
    fetch_remote_models_with(&repo, &AiAdapter, http_client, channel_id).await
}

/// 使用注入的仓储与 AI 适配器从远程拉取模型列表。
pub async fn fetch_remote_models_with<R: ChannelRepo, A: AiMetadataClient>(
    repo: &R,
    adapter: &A,
    http_client: &reqwest::Client,
    channel_id: &str,
) -> Result<Vec<RemoteModelInfo>, AppError> {
    let channel = get_with(repo, channel_id).await?;
    let config = AiChannelConfig::try_from(&channel)?;

    adapter.fetch_remote_models(http_client, &config).await
}
