pub mod custom;
pub mod openai;

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use sqlx::SqlitePool;
use tokio::sync::{mpsc, RwLock};

use crate::db::{models::ProviderConfigRow, provider_config};
use crate::error::{AppError, Result};
use crate::services::keyring::KeyringService;
use crate::types::{ChatRequest, ChatResponse, ModelInfo, StreamEvent, ToolDef};

#[async_trait]
pub trait LlmProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn list_models(&self) -> Result<Vec<ModelInfo>>;
    async fn chat(&self, req: &ChatRequest) -> Result<ChatResponse>;
    async fn chat_stream(&self, req: &ChatRequest, tx: mpsc::Sender<StreamEvent>) -> Result<()>;
    fn supports_function_calling(&self) -> bool {
        false
    }
    fn format_tools(&self, _tools: &[ToolDef]) -> serde_json::Value {
        serde_json::Value::Null
    }
    async fn health_check(&self) -> Result<()>;
}

pub struct ProviderRegistry {
    providers: RwLock<HashMap<String, Arc<dyn LlmProvider>>>,
}

impl ProviderRegistry {
    pub fn new_shared() -> Arc<Self> {
        Arc::new(Self {
            providers: RwLock::new(HashMap::new()),
        })
    }

    pub async fn register(&self, provider: Arc<dyn LlmProvider>) {
        self.providers
            .write()
            .await
            .insert(provider.name().to_string(), provider);
    }

    pub async fn deregister(&self, name: &str) {
        self.providers.write().await.remove(name);
    }

    pub async fn get(&self, name: &str) -> Result<Arc<dyn LlmProvider>> {
        self.providers
            .read()
            .await
            .get(name)
            .cloned()
            .ok_or_else(|| AppError::ProviderNotFound {
                provider: name.to_string(),
            })
    }
}

pub async fn init_enabled_providers(
    registry: &Arc<ProviderRegistry>,
    db: &SqlitePool,
    keyring: &Arc<KeyringService>,
) -> Result<()> {
    let configs = provider_config::list_enabled(db).await?;
    for config in configs {
        if let Some(provider) = build_provider(&config, keyring).await? {
            registry.register(provider).await;
        }
    }

    let custom_channels = crate::db::custom_channel::list_enabled(db).await?;
    for row in custom_channels {
        if let Some(provider) = build_custom_channel_provider(&row, keyring).await? {
            registry.register(provider).await;
        }
    }
    Ok(())
}

pub async fn build_provider(
    config: &ProviderConfigRow,
    keyring: &Arc<KeyringService>,
) -> Result<Option<Arc<dyn LlmProvider>>> {
    let api_key = match &config.api_key_id {
        Some(key_id) => keyring.get_optional(key_id)?,
        None => None,
    };

    match config.provider.as_str() {
        "openai" => Ok(Some(Arc::new(openai::OpenAiProvider::new(
            api_key,
            config.base_url.clone(),
        )))),
        _ => Ok(None),
    }
}

pub async fn build_custom_channel_provider(
    row: &crate::db::models::CustomChannelRow,
    keyring: &Arc<KeyringService>,
) -> Result<Option<Arc<dyn LlmProvider>>> {
    Ok(Some(Arc::new(custom::CustomChannelProvider::from_row(
        row.clone(),
        keyring,
    )?)))
}
