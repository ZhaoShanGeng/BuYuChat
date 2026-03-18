use std::sync::Arc;

use reqwest::{header::HeaderMap, Client, Method};
use sqlx::SqlitePool;
use tracing::{debug, info};

use crate::db::{
    custom_channel,
    models::{CustomChannelRow, ProviderConfigRow},
    provider_config,
};
use crate::error::{AppError, Result};
use crate::providers::{self, ProviderRegistry};
use crate::services::keyring::KeyringService;
use crate::types::ModelInfo;

pub struct RawProviderResponse {
    pub url: String,
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

pub struct ProviderService {
    db: SqlitePool,
    providers: Arc<ProviderRegistry>,
    keyring: Arc<KeyringService>,
}

impl ProviderService {
    pub fn new(
        db: SqlitePool,
        providers: Arc<ProviderRegistry>,
        keyring: Arc<KeyringService>,
    ) -> Self {
        Self {
            db,
            providers,
            keyring,
        }
    }

    pub async fn list_configs(&self) -> Result<Vec<ProviderConfigRow>> {
        provider_config::list_all(&self.db).await
    }

    pub async fn list_custom_channels(&self) -> Result<Vec<CustomChannelRow>> {
        custom_channel::list_all(&self.db).await
    }

    pub fn save_api_key(&self, key_id: &str, value: &str) -> Result<()> {
        self.keyring.save(key_id, value)
    }

    pub fn delete_api_key(&self, key_id: &str) -> Result<()> {
        self.keyring.delete(key_id)
    }

    pub async fn get_api_key(&self, provider: &str) -> Result<Option<String>> {
        let existing = provider_config::get_by_provider(&self.db, provider).await?;
        let Some(key_id) = existing.and_then(|config| config.api_key_id) else {
            return Ok(None);
        };

        if !self.keyring.contains(&key_id)? {
            return Ok(None);
        }

        Ok(Some(self.keyring.get(&key_id)?))
    }

    pub async fn get_custom_channel_api_key(&self, id: &str) -> Result<Option<String>> {
        let row = custom_channel::get(&self.db, id).await?;
        let auth: serde_json::Value = serde_json::from_str(&row.auth_json)?;

        if let Some(api_key) = auth["api_key"].as_str() {
            return Ok(Some(api_key.to_string()));
        }

        match auth["key_ref"].as_str() {
            Some(key_id) => self.keyring.get_optional(key_id),
            None => Ok(None),
        }
    }

    pub async fn save_config(
        &self,
        provider: &str,
        api_key: Option<&str>,
        base_url: Option<&str>,
    ) -> Result<()> {
        let existing = provider_config::get_by_provider(&self.db, provider).await?;
        let key_id = format!("provider:{provider}");
        let api_key_id = match api_key {
            Some(value) if !value.trim().is_empty() => {
                self.keyring.save(&key_id, value)?;
                Some(key_id.as_str())
            }
            Some(_) => {
                self.keyring.delete(&key_id)?;
                None
            }
            None => match existing
                .as_ref()
                .and_then(|config| config.api_key_id.as_deref())
            {
                Some(existing_key_id) if self.keyring.contains(existing_key_id)? => {
                    Some(existing_key_id)
                }
                _ => None,
            },
        };

        let existing_base_url = existing
            .as_ref()
            .and_then(|config| config.base_url.as_deref());
        let final_base_url = base_url.or(existing_base_url);

        provider_config::save(&self.db, provider, api_key_id, final_base_url, true).await?;
        if api_key_id.is_some() {
            self.register_provider(provider).await?;
        }
        Ok(())
    }

    pub async fn test_connection(&self, provider: &str) -> Result<()> {
        self.register_provider(provider).await?;
        self.providers.get(provider).await?.health_check().await
    }

    pub async fn list_models(&self, provider: &str) -> Result<Vec<ModelInfo>> {
        self.register_provider(provider).await?;
        self.providers.get(provider).await?.list_models().await
    }

    pub async fn refresh_custom_channel_models(&self, id: &str) -> Result<Vec<ModelInfo>> {
        let provider_name = format!("custom:{id}");
        let models = self.list_models(&provider_name).await?;
        let models_json = serde_json::to_string(&models)?;
        custom_channel::update_models_json(&self.db, id, &models_json).await?;
        Ok(models)
    }

    pub async fn save_custom_channel_models(&self, id: &str, models: &[ModelInfo]) -> Result<()> {
        let models_json = serde_json::to_string(models)?;
        custom_channel::update_models_json(&self.db, id, &models_json).await?;
        Ok(())
    }

    pub async fn create_custom_channel(
        &self,
        name: &str,
        channel_type: &str,
        base_url: &str,
        models_path: &str,
        chat_path: &str,
        stream_path: &str,
        api_key: Option<&str>,
    ) -> Result<CustomChannelRow> {
        info!(channel_name = %name, channel_type = %channel_type, base_url = %base_url, "creating custom channel");
        let row = custom_channel::create(
            &self.db,
            name,
            channel_type,
            base_url,
            &auth_json(api_key),
            &endpoints_json(models_path, chat_path, stream_path),
            &default_request_template_json(),
            &default_response_mapping_json(),
            &default_stream_mapping_json(),
            "[]",
        )
        .await?;

        self.register_provider(&format!("custom:{}", row.id))
            .await?;
        Ok(row)
    }

    pub async fn update_custom_channel(
        &self,
        id: &str,
        name: &str,
        channel_type: &str,
        base_url: &str,
        models_path: &str,
        chat_path: &str,
        stream_path: &str,
        api_key: Option<&str>,
    ) -> Result<CustomChannelRow> {
        info!(channel_id = %id, channel_name = %name, channel_type = %channel_type, base_url = %base_url, "updating custom channel");
        let existing = custom_channel::get(&self.db, id).await?;

        let row = custom_channel::update(
            &self.db,
            id,
            name,
            channel_type,
            base_url,
            &auth_json(api_key),
            &endpoints_json(models_path, chat_path, stream_path),
            &existing.request_template_json,
            &existing.response_mapping_json,
            &existing.stream_mapping_json,
            &existing.models_json,
            existing.enabled,
        )
        .await?;

        self.register_provider(&format!("custom:{}", row.id))
            .await?;
        Ok(row)
    }

    pub async fn delete_custom_channel(&self, id: &str) -> Result<()> {
        let provider_name = format!("custom:{id}");
        custom_channel::delete(&self.db, id).await?;
        self.providers.deregister(&provider_name).await;
        Ok(())
    }

    pub async fn send_raw_request_direct(
        base_url: &str,
        api_key: Option<&str>,
        method: &str,
        path: &str,
        headers: Vec<(String, String)>,
        body: Option<String>,
    ) -> Result<RawProviderResponse> {
        let url = format!(
            "{}/{}",
            base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        );

        Self::execute_raw_request(url, api_key.map(str::to_string), method, headers, body).await
    }

    async fn execute_raw_request(
        url: String,
        api_key: Option<String>,
        method: &str,
        headers: Vec<(String, String)>,
        body: Option<String>,
    ) -> Result<RawProviderResponse> {
        let method = Method::from_bytes(method.trim().to_uppercase().as_bytes())
            .map_err(|err| AppError::Other(format!("invalid HTTP method: {err}")))?;

        let client = Client::new();
        let mut request = client.request(method, &url);

        if let Some(api_key) = api_key {
            request = request.bearer_auth(api_key);
        }

        let mut header_map = HeaderMap::new();
        for (name, value) in headers {
            let name = reqwest::header::HeaderName::from_bytes(name.as_bytes())
                .map_err(|err| AppError::Other(format!("invalid header name: {err}")))?;
            let value = reqwest::header::HeaderValue::from_str(&value)
                .map_err(|err| AppError::Other(format!("invalid header value: {err}")))?;
            header_map.insert(name, value);
        }
        request = request.headers(header_map);

        if let Some(body) = body {
            request = request.body(body);
        }

        let response = request
            .send()
            .await
            .map_err(|err| AppError::Other(format!("request failed: {err}")))?;

        let status = response.status().as_u16();
        let headers = response
            .headers()
            .iter()
            .map(|(name, value)| {
                (
                    name.to_string(),
                    value.to_str().unwrap_or("<binary>").to_string(),
                )
            })
            .collect::<Vec<_>>();
        let body = response
            .text()
            .await
            .map_err(|err| AppError::Other(format!("failed to read response body: {err}")))?;

        Ok(RawProviderResponse {
            url,
            status,
            headers,
            body,
        })
    }

    async fn register_provider(&self, provider: &str) -> Result<()> {
        debug!(provider = %provider, "registering provider");
        if let Some(id) = provider.strip_prefix("custom:") {
            let row = custom_channel::get(&self.db, id).await?;
            let provider_impl = providers::build_custom_channel_provider(&row, &self.keyring)
                .await?
                .ok_or_else(|| AppError::ProviderNotFound {
                    provider: provider.to_string(),
                })?;
            self.providers.register(provider_impl).await;
            return Ok(());
        }

        let config = provider_config::get_by_provider(&self.db, provider)
            .await?
            .ok_or_else(|| AppError::ProviderNotFound {
                provider: provider.to_string(),
            })?;

        let built = providers::build_provider(&config, &self.keyring).await?;
        let provider_impl = built.ok_or_else(|| AppError::ProviderNotFound {
            provider: provider.to_string(),
        })?;
        self.providers.register(provider_impl).await;
        Ok(())
    }
}

fn auth_json(api_key: Option<&str>) -> String {
    let api_key = api_key
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string);
    serde_json::json!({
        "auth_type": "bearer",
        "api_key": api_key,
    })
    .to_string()
}

fn endpoints_json(models_path: &str, chat_path: &str, stream_path: &str) -> String {
    serde_json::json!({
        "models": models_path,
        "chat": chat_path,
        "stream": stream_path,
    })
    .to_string()
}

fn default_request_template_json() -> String {
    serde_json::json!({
        "model": "{{model}}",
        "messages": "{{messages}}",
        "stream": "{{stream}}",
        "temperature": "{{temperature}}",
        "max_tokens": "{{max_tokens}}",
    })
    .to_string()
}

fn default_response_mapping_json() -> String {
    serde_json::json!({
        "content": "$.choices[0].message.content",
        "finish_reason": "$.choices[0].finish_reason",
    })
    .to_string()
}

fn default_stream_mapping_json() -> String {
    serde_json::json!({
        "delta": "$.choices[0].delta.content",
        "done_sentinel": "[DONE]",
    })
    .to_string()
}
