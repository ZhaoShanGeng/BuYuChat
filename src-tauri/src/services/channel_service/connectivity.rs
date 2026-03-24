//! 渠道模型接口连通性测试的编排逻辑。

use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::{
    error::AppError,
    models::{Channel, ChannelTestResult, TestChannelRequest},
    repo::channel_repo::{ChannelRepo, SqlxChannelRepo},
};

use super::{crud::get_with, validation::resolve_config};

/// 服务层与测试共用的连通性探测抽象。
#[async_trait]
pub trait ChannelConnectivityProbe: Send + Sync {
    /// 对归一化请求执行探测。
    async fn get(&self, request: &TestChannelRequest) -> Result<(), String>;
}

/// 基于共享 reqwest 客户端的生产探测器。
#[derive(Debug, Clone)]
pub struct ReqwestConnectivityProbe {
    client: reqwest::Client,
}

impl ReqwestConnectivityProbe {
    /// 使用共享 HTTP 客户端创建探测器。
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl ChannelConnectivityProbe for ReqwestConnectivityProbe {
    /// 发起 GET 探测请求并检查响应状态码。
    async fn get(&self, request: &TestChannelRequest) -> Result<(), String> {
        let mut builder = self.client.get(&request.url);
        if let Some((name, value)) = &request.auth_header {
            builder = builder.header(name, value);
        }

        let response = builder.send().await.map_err(|error| error.to_string())?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("remote endpoint returned {}", response.status()))
        }
    }
}

/// 使用连接池与 HTTP 客户端测试渠道连通性。
pub async fn test_channel(
    pool: &SqlitePool,
    http_client: &reqwest::Client,
    id: &str,
) -> Result<ChannelTestResult, AppError> {
    let repo = SqlxChannelRepo::new(pool.clone());
    let probe = ReqwestConnectivityProbe::new(http_client.clone());
    test_with(&repo, &probe, id).await
}

/// 使用注入的仓储与探测器测试渠道连通性。
pub async fn test_with<R: ChannelRepo, P: ChannelConnectivityProbe>(
    repo: &R,
    probe: &P,
    id: &str,
) -> Result<ChannelTestResult, AppError> {
    let channel = get_with(repo, id).await?;
    let request = build_test_request(&channel)?;
    probe.get(&request).await.map_err(|error| {
        AppError::channel_unreachable(format!("failed to reach channel: {error}"))
    })?;

    Ok(ChannelTestResult {
        success: true,
        message: Some("channel is reachable".to_string()),
    })
}

/// 将渠道配置归一化为最终的探测请求。
pub fn build_test_request(channel: &Channel) -> Result<TestChannelRequest, AppError> {
    let config = resolve_config(Some(&channel.channel_type))?;
    let endpoint = channel
        .models_endpoint
        .clone()
        .unwrap_or_else(|| config.models_endpoint.to_string());
    let auth_type = channel
        .auth_type
        .clone()
        .unwrap_or_else(|| config.auth_type.to_string());

    let auth_header = match auth_type.as_str() {
        "bearer" => channel
            .api_key
            .clone()
            .map(|key| ("Authorization".to_string(), format!("Bearer {key}"))),
        "x_api_key" => channel
            .api_key
            .clone()
            .map(|key| ("x-api-key".to_string(), key)),
        "none" => None,
        other => {
            return Err(AppError::validation(
                "VALIDATION_ERROR",
                format!("unsupported auth_type '{other}'"),
            ));
        }
    };

    Ok(TestChannelRequest {
        url: format!("{}{}", channel.base_url.trim_end_matches('/'), endpoint),
        auth_header,
    })
}
