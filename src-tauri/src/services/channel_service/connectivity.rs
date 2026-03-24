//! 渠道模型接口连通性测试的编排逻辑。

use sqlx::SqlitePool;

use crate::{
    ai::adapter::{AiAdapter, AiChannelConfig, AiMetadataClient},
    error::AppError,
    models::ChannelTestResult,
    repo::channel_repo::{ChannelRepo, SqlxChannelRepo},
};

use super::crud::get_with;

/// 使用连接池与 HTTP 客户端测试渠道连通性。
pub async fn test_channel(
    pool: &SqlitePool,
    http_client: &reqwest::Client,
    id: &str,
) -> Result<ChannelTestResult, AppError> {
    let repo = SqlxChannelRepo::new(pool.clone());
    test_with(&repo, &AiAdapter, http_client, id).await
}

/// 使用注入的仓储与 AI 适配器测试渠道连通性。
pub async fn test_with<R: ChannelRepo, A: AiMetadataClient>(
    repo: &R,
    adapter: &A,
    http_client: &reqwest::Client,
    id: &str,
) -> Result<ChannelTestResult, AppError> {
    let channel = get_with(repo, id).await?;
    let config = AiChannelConfig::try_from(&channel)?;
    adapter.probe_models_endpoint(http_client, &config).await?;

    Ok(ChannelTestResult {
        success: true,
        message: Some("channel is reachable".to_string()),
    })
}
