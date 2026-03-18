use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::mpsc;
use tracing::debug;

use crate::db::models::CustomChannelRow;
use crate::error::Result;
use crate::providers::{openai::OpenAiProvider, LlmProvider};
use crate::services::keyring::KeyringService;
use crate::types::{ChatRequest, ChatResponse, ModelInfo, StreamEvent, ToolDef};

#[derive(Clone)]
pub struct CustomChannelProvider {
    inner: OpenAiProvider,
}

impl CustomChannelProvider {
    pub fn from_row(row: CustomChannelRow, keyring: &Arc<KeyringService>) -> Result<Self> {
        let auth: serde_json::Value = serde_json::from_str(&row.auth_json)?;
        let endpoints: serde_json::Value = serde_json::from_str(&row.endpoints_json)?;
        let api_key = match auth["api_key"].as_str() {
            Some(value) => Some(value.to_string()),
            None => auth["key_ref"]
                .as_str()
                .map(|key_ref| keyring.get_optional(key_ref))
                .transpose()?
                .flatten(),
        };

        debug!(
            provider = %format!("custom:{}", row.id),
            channel_name = %row.name,
            channel_type = %row.channel_type,
            base_url = %row.base_url,
            models_path = %endpoints["models"].as_str().unwrap_or("models"),
            chat_path = %endpoints["chat"].as_str().unwrap_or("chat/completions"),
            api_key_present = api_key.as_ref().map(|value| !value.is_empty()).unwrap_or(false),
            api_key_len = api_key.as_ref().map(|value| value.len()).unwrap_or(0),
            "built custom channel provider"
        );

        Ok(Self {
            inner: OpenAiProvider::new_named(
                format!("custom:{}", row.id),
                api_key,
                Some(row.base_url),
                endpoints["models"].as_str().map(str::to_string),
                endpoints["chat"].as_str().map(str::to_string),
            ),
        })
    }
}

#[async_trait]
impl LlmProvider for CustomChannelProvider {
    fn name(&self) -> &str {
        self.inner.name()
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        self.inner.list_models().await
    }

    async fn chat(&self, req: &ChatRequest) -> Result<ChatResponse> {
        self.inner.chat(req).await
    }

    async fn chat_stream(&self, req: &ChatRequest, tx: mpsc::Sender<StreamEvent>) -> Result<()> {
        self.inner.chat_stream(req, tx).await
    }

    fn supports_function_calling(&self) -> bool {
        self.inner.supports_function_calling()
    }

    fn format_tools(&self, tools: &[ToolDef]) -> serde_json::Value {
        self.inner.format_tools(tools)
    }

    async fn health_check(&self) -> Result<()> {
        self.inner.health_check().await
    }
}
