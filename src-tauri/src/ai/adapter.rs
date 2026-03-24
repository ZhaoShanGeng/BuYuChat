//! 使用 AISDK 构建 OpenAI-compatible 适配器。

use aisdk::{core::DynamicModel, providers::OpenAICompatible};

use crate::error::AppError;

/// 构建 AISDK provider 所需的最小渠道配置。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiChannelConfig {
    /// Provider 展示名称。
    pub provider_name: String,
    /// 不带具体接口路径的渠道根地址。
    pub base_url: String,
    /// Provider 使用的 API Key。
    pub api_key: String,
    /// 当前选中的模型 ID。
    pub model_name: String,
    /// 可选的聊天接口覆盖路径。
    pub path: Option<String>,
}

/// 根据渠道配置创建 AISDK OpenAI-compatible provider。
#[derive(Debug, Default, Clone, Copy)]
pub struct AiAdapter;

impl AiAdapter {
    /// 基于运行时渠道配置构建 `OpenAICompatible` provider。
    pub fn openai_compatible_provider(
        &self,
        config: &AiChannelConfig,
    ) -> Result<OpenAICompatible<DynamicModel>, AppError> {
        OpenAICompatible::<DynamicModel>::builder()
            .provider_name(config.provider_name.clone())
            .base_url(config.base_url.clone())
            .api_key(config.api_key.clone())
            .model_name(config.model_name.clone())
            .path(
                config
                    .path
                    .clone()
                    .unwrap_or_else(|| "/v1/chat/completions".to_string()),
            )
            .build()
            .map_err(|error| AppError::internal(format!("failed to build aisdk provider: {error}")))
    }
}

#[cfg(test)]
mod tests {
    use super::{AiAdapter, AiChannelConfig};

    /// 校验 AISDK provider 能按渠道配置正常构建。
    #[test]
    fn openai_compatible_provider_can_be_built() {
        let provider = AiAdapter
            .openai_compatible_provider(&AiChannelConfig {
                provider_name: "BuYu".to_string(),
                base_url: "https://api.openai.com".to_string(),
                api_key: "sk-test".to_string(),
                model_name: "gpt-4o-mini".to_string(),
                path: Some("/v1/chat/completions".to_string()),
            })
            .unwrap();

        assert_eq!(provider.settings.provider_name, "BuYu");
        assert_eq!(provider.settings.base_url, "https://api.openai.com/");
        assert_eq!(provider.settings.api_key, "sk-test");
        assert_eq!(
            provider.settings.path.as_deref(),
            Some("/v1/chat/completions")
        );
    }
}
