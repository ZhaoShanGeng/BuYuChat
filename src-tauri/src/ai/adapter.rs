//! 使用 AISDK 构建 OpenAI-compatible 适配器。

use async_trait::async_trait;
use aisdk::{core::DynamicModel, providers::OpenAICompatible};
use serde::Deserialize;

use crate::{
    channel_types::config_for,
    error::AppError,
    models::{Channel, RemoteModelInfo},
};

/// 构建 AISDK provider 与远程元数据请求所需的统一渠道配置。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiChannelConfig {
    /// Provider 展示名称。
    pub provider_name: String,
    /// 不带具体接口路径的渠道根地址。
    pub base_url: String,
    /// Provider 使用的 API Key。
    pub api_key: Option<String>,
    /// 对外请求使用的鉴权方式。
    pub auth_type: String,
    /// 模型列表接口路径。
    pub models_endpoint: String,
    /// 聊天接口路径。
    pub chat_endpoint: String,
    /// 当前选中的模型 ID。
    pub model_name: Option<String>,
}

impl AiChannelConfig {
    /// 为聊天生成链路补充当前选中的模型名称。
    pub fn with_model_name(mut self, model_name: impl Into<String>) -> Self {
        self.model_name = Some(model_name.into());
        self
    }
}

impl TryFrom<&Channel> for AiChannelConfig {
    type Error = AppError;

    /// 将渠道资源归一化为 AI 适配层使用的配置。
    fn try_from(channel: &Channel) -> Result<Self, Self::Error> {
        let defaults = config_for(&channel.channel_type)?;

        Ok(Self {
            provider_name: channel.name.clone(),
            base_url: channel.base_url.trim().to_string(),
            api_key: channel.api_key.clone(),
            auth_type: channel
                .auth_type
                .clone()
                .unwrap_or_else(|| defaults.auth_type.to_string()),
            models_endpoint: channel
                .models_endpoint
                .clone()
                .unwrap_or_else(|| defaults.models_endpoint.to_string()),
            chat_endpoint: channel
                .chat_endpoint
                .clone()
                .unwrap_or_else(|| defaults.chat_endpoint.to_string()),
            model_name: None,
        })
    }
}

/// AI 适配层向服务层暴露的远程元数据访问契约。
#[async_trait]
pub trait AiMetadataClient: Send + Sync {
    /// 对模型接口执行连通性探测。
    async fn probe_models_endpoint(
        &self,
        http_client: &reqwest::Client,
        config: &AiChannelConfig,
    ) -> Result<(), AppError>;

    /// 从远程模型接口拉取模型列表。
    async fn fetch_remote_models(
        &self,
        http_client: &reqwest::Client,
        config: &AiChannelConfig,
    ) -> Result<Vec<RemoteModelInfo>, AppError>;
}

/// 根据渠道配置创建 AISDK OpenAI-compatible provider。
#[derive(Debug, Default, Clone, Copy)]
pub struct AiAdapter;

impl AiAdapter {
    /// 基于运行时渠道配置构建 `OpenAICompatible` provider。
    pub fn build_openai_compatible_provider(
        &self,
        config: &AiChannelConfig,
    ) -> Result<OpenAICompatible<DynamicModel>, AppError> {
        let model_name = config.model_name.clone().ok_or_else(|| {
            AppError::validation("VALIDATION_ERROR", "model_name is required for chat generation")
        })?;

        OpenAICompatible::<DynamicModel>::builder()
            .provider_name(config.provider_name.clone())
            .base_url(config.base_url.clone())
            .api_key(config.api_key.clone().unwrap_or_default())
            .model_name(model_name)
            .path(config.chat_endpoint.clone())
            .build()
            .map_err(|error| AppError::internal(format!("failed to build aisdk provider: {error}")))
    }
}

#[async_trait]
impl AiMetadataClient for AiAdapter {
    async fn probe_models_endpoint(
        &self,
        http_client: &reqwest::Client,
        config: &AiChannelConfig,
    ) -> Result<(), AppError> {
        execute_models_request(http_client, config).await.map(|_| ())
    }

    async fn fetch_remote_models(
        &self,
        http_client: &reqwest::Client,
        config: &AiChannelConfig,
    ) -> Result<Vec<RemoteModelInfo>, AppError> {
        let body = execute_models_request(http_client, config).await?;
        let response: OpenAiModelsResponse = serde_json::from_str(&body).map_err(|error| {
            AppError::ai_request_failed(format!(
                "failed to parse remote model response: {error}"
            ))
        })?;

        Ok(response
            .data
            .into_iter()
            .map(|model| RemoteModelInfo {
                model_id: model.id,
                display_name: model.display_name.or(model.name),
                context_window: model.context_window,
            })
            .collect())
    }
}

/// OpenAI-compatible `/v1/models` 响应的顶层结构。
#[derive(Debug, Deserialize)]
struct OpenAiModelsResponse {
    /// 上游返回的模型条目列表。
    data: Vec<OpenAiModelRecord>,
}

/// OpenAI-compatible `/v1/models` 响应中的单条模型记录。
#[derive(Debug, Deserialize)]
struct OpenAiModelRecord {
    /// 模型标识。
    id: String,
    /// 可选展示名称。
    #[serde(default)]
    display_name: Option<String>,
    /// 部分服务商使用 `name` 作为展示名称。
    #[serde(default)]
    name: Option<String>,
    /// 可选上下文窗口大小。
    #[serde(default)]
    context_window: Option<i64>,
}

/// 对模型接口执行统一的 GET 请求并返回原始响应体。
async fn execute_models_request(
    http_client: &reqwest::Client,
    config: &AiChannelConfig,
) -> Result<String, AppError> {
    let mut builder = http_client.get(build_models_endpoint_url(config));

    if let Some((name, value)) = build_auth_header(config)? {
        builder = builder.header(name, value);
    }

    let response = builder.send().await.map_err(|error| {
        AppError::channel_unreachable(format!("failed to reach channel: {error}"))
    })?;
    let status = response.status();
    let body = response.text().await.map_err(|error| {
        AppError::ai_request_failed(format!("failed to read remote response: {error}"))
    })?;

    if status.is_success() {
        return Ok(body);
    }

    Err(AppError::ai_request_failed(format!(
        "remote endpoint returned {status}: {body}"
    )))
}

/// 根据统一配置构造最终的模型接口 URL。
fn build_models_endpoint_url(config: &AiChannelConfig) -> String {
    format!(
        "{}{}",
        config.base_url.trim_end_matches('/'),
        config.models_endpoint
    )
}

/// 根据统一配置构造远程请求使用的鉴权头。
fn build_auth_header(config: &AiChannelConfig) -> Result<Option<(&'static str, String)>, AppError> {
    match config.auth_type.as_str() {
        "bearer" => Ok(config
            .api_key
            .clone()
            .map(|key| ("Authorization", format!("Bearer {key}")))),
        "x_api_key" => Ok(config.api_key.clone().map(|key| ("x-api-key", key))),
        "none" => Ok(None),
        other => Err(AppError::validation(
            "VALIDATION_ERROR",
            format!("unsupported auth_type '{other}'"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use crate::ai::adapter::AiMetadataClient;
    use wiremock::{
        matchers::{header, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use super::{AiAdapter, AiChannelConfig};

    /// 构造一个默认的 AI 渠道配置样本。
    fn sample_config(base_url: String) -> AiChannelConfig {
        AiChannelConfig {
            provider_name: "BuYu".to_string(),
            base_url,
            api_key: Some("sk-test".to_string()),
            auth_type: "bearer".to_string(),
            models_endpoint: "/v1/models".to_string(),
            chat_endpoint: "/v1/chat/completions".to_string(),
            model_name: Some("gpt-4o-mini".to_string()),
        }
    }

    /// AI 生成使用的 OpenAI-compatible provider 应能按统一配置正常构建。
    #[test]
    fn build_openai_compatible_provider_can_be_built() {
        let provider = AiAdapter
            .build_openai_compatible_provider(&sample_config("https://api.openai.com".to_string()))
            .unwrap();

        assert_eq!(provider.settings.provider_name, "BuYu");
        assert_eq!(provider.settings.base_url, "https://api.openai.com/");
        assert_eq!(provider.settings.api_key, "sk-test");
        assert_eq!(
            provider.settings.path.as_deref(),
            Some("/v1/chat/completions")
        );
    }

    /// 连通性探测应使用默认模型接口与 Bearer 鉴权头。
    #[tokio::test]
    async fn probe_models_endpoint_uses_models_path_and_auth_header() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1/models"))
            .and(header("authorization", "Bearer sk-test"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        AiAdapter
            .probe_models_endpoint(&reqwest::Client::new(), &sample_config(server.uri()))
            .await
            .unwrap();
    }

    /// 渠道资源应能归一化为 AI 适配层配置并补齐默认值。
    #[test]
    fn ai_channel_config_try_from_channel_applies_defaults() {
        let config = AiChannelConfig::try_from(&crate::models::Channel {
            id: "channel-1".to_string(),
            name: "OpenAI".to_string(),
            channel_type: "openai_compatible".to_string(),
            base_url: "https://api.openai.com".to_string(),
            api_key: Some("sk-test".to_string()),
            auth_type: None,
            models_endpoint: None,
            chat_endpoint: None,
            stream_endpoint: None,
            enabled: true,
            created_at: 100,
            updated_at: 100,
        })
        .unwrap();

        assert_eq!(config.auth_type, "bearer");
        assert_eq!(config.models_endpoint, "/v1/models");
        assert_eq!(config.chat_endpoint, "/v1/chat/completions");
    }

    /// 远程拉取模型应能解析 OpenAI-compatible 响应。
    #[tokio::test]
    async fn fetch_remote_models_parses_openai_compatible_response() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1/models"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "data": [
                        { "id": "gpt-4o", "display_name": "GPT-4o", "context_window": 128000 },
                        { "id": "gpt-4o-mini" }
                    ]
                })),
            )
            .mount(&server)
            .await;

        let models = AiAdapter
            .fetch_remote_models(&reqwest::Client::new(), &sample_config(server.uri()))
            .await
            .unwrap();

        assert_eq!(models.len(), 2);
        assert_eq!(models[0].model_id, "gpt-4o");
        assert_eq!(models[0].display_name.as_deref(), Some("GPT-4o"));
        assert_eq!(models[0].context_window, Some(128_000));
        assert_eq!(models[1].model_id, "gpt-4o-mini");
    }

    /// 远程非成功状态应映射为 AI_REQUEST_FAILED。
    #[tokio::test]
    async fn fetch_remote_models_maps_non_success_status_to_ai_request_failed() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1/models"))
            .respond_with(ResponseTemplate::new(503).set_body_string("upstream unavailable"))
            .mount(&server)
            .await;

        let error = AiAdapter
            .fetch_remote_models(&reqwest::Client::new(), &sample_config(server.uri()))
            .await
            .unwrap_err();

        assert_eq!(
            error,
            crate::error::AppError::ai_request_failed(
                "remote endpoint returned 503 Service Unavailable: upstream unavailable"
            )
        );
    }

    /// 网络连接失败应映射为 CHANNEL_UNREACHABLE。
    #[tokio::test]
    async fn probe_models_endpoint_maps_transport_errors_to_channel_unreachable() {
        let error = AiAdapter
            .probe_models_endpoint(
                &reqwest::Client::new(),
                &sample_config("http://127.0.0.1:1".to_string()),
            )
            .await
            .unwrap_err();

        assert_eq!(error.error_code, "CHANNEL_UNREACHABLE");
    }
}
