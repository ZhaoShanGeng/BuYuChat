use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;

use crate::domain::api_channels::{ApiChannel, ApiChannelModel};
use crate::domain::messages::{
    ProviderCapabilities, ProviderChatEvent, ProviderChatRequest, ProviderChatResponse,
};
use crate::support::error::{AppError, Result};

pub mod anthropic;
pub mod common;
pub mod gemini;
pub mod openai_compatible;

pub type ProviderEventCallback<'a> = dyn FnMut(ProviderChatEvent) -> Result<()> + Send + 'a;

#[async_trait]
pub trait ChatProvider: Send + Sync {
    fn provider_type(&self) -> &'static str;

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities::default()
    }

    async fn test_connection(&self, channel: &ApiChannel) -> Result<()>;

    async fn list_models(&self, channel: &ApiChannel) -> Result<Vec<ApiChannelModel>>;

    async fn chat(&self, req: ProviderChatRequest) -> Result<ProviderChatResponse>;

    async fn chat_stream(
        &self,
        req: ProviderChatRequest,
        on_event: &mut ProviderEventCallback<'_>,
    ) -> Result<ProviderChatResponse> {
        let response = self.chat(req).await?;
        if !response.parts.is_empty() {
            on_event(ProviderChatEvent::Delta {
                parts: response.parts.clone(),
                raw_event_json: response.raw_response_json.clone(),
            })?;
        }
        on_event(ProviderChatEvent::Finished {
            finish_reason: response.finish_reason.clone(),
            prompt_tokens: response.prompt_tokens,
            completion_tokens: response.completion_tokens,
            total_tokens: response.total_tokens,
            raw_event_json: response.raw_response_json.clone(),
        })?;
        Ok(response)
    }
}

#[derive(Clone, Default)]
pub struct ProviderRegistry {
    providers: HashMap<String, Arc<dyn ChatProvider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register(Arc::new(anthropic::AnthropicProvider::new()));
        registry.register(Arc::new(gemini::GeminiProvider::new()));
        registry.register(Arc::new(openai_compatible::OpenAiCompatibleProvider::new()));
        registry
    }

    pub fn register(&mut self, provider: Arc<dyn ChatProvider>) {
        self.providers
            .insert(provider.provider_type().to_string(), provider);
    }

    pub fn get(&self, provider_type: &str) -> Result<Arc<dyn ChatProvider>> {
        self.providers.get(provider_type).cloned().ok_or_else(|| {
            AppError::Validation(format!("unsupported provider type '{provider_type}'"))
        })
    }
}
