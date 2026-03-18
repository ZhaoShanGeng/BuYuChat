use async_trait::async_trait;
use reqwest::Client;
use tokio::sync::mpsc;
use tracing::{debug, warn};

use crate::error::{AppError, Result};
use crate::providers::LlmProvider;
use crate::types::{
    ChatRequest, ChatResponse, Message, ModelInfo, StreamEvent, TokenUsage, ToolCall, ToolDef,
};

#[derive(Clone)]
pub struct OpenAiProvider {
    client: Client,
    name: String,
    api_key: Option<String>,
    base_url: String,
    models_path: String,
    chat_path: String,
}

impl OpenAiProvider {
    pub fn new(api_key: Option<String>, base_url: Option<String>) -> Self {
        Self::new_named(
            "openai".to_string(),
            api_key,
            base_url,
            Some("models".to_string()),
            Some("chat/completions".to_string()),
        )
    }

    pub fn new_named(
        name: String,
        api_key: Option<String>,
        base_url: Option<String>,
        models_path: Option<String>,
        chat_path: Option<String>,
    ) -> Self {
        Self {
            client: Client::new(),
            name,
            api_key,
            base_url: base_url.unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
            models_path: models_path.unwrap_or_else(|| "models".to_string()),
            chat_path: chat_path.unwrap_or_else(|| "chat/completions".to_string()),
        }
    }

    fn endpoint(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    fn map_messages(&self, req: &ChatRequest) -> Vec<serde_json::Value> {
        let mut items = Vec::new();
        if let Some(system_prompt) = &req.system_prompt {
            items.push(serde_json::json!({
                "role": "system",
                "content": system_prompt,
            }));
        }
        items.extend(req.messages.iter().map(map_message));
        items
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    fn name(&self) -> &str {
        &self.name
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        let url = self.endpoint(&self.models_path);
        debug!(
            provider = self.name(),
            %url,
            api_key_present = self.api_key.as_ref().map(|value| !value.is_empty()).unwrap_or(false),
            api_key_len = self.api_key.as_ref().map(|value| value.len()).unwrap_or(0),
            "listing models"
        );

        let mut request = self.client.get(&url);
        if let Some(api_key) = self.api_key.as_deref().filter(|value| !value.is_empty()) {
            request = request.bearer_auth(api_key);
        }

        let response = request
            .send()
            .await
            .map_err(|err| AppError::Other(format!("request failed: {err}")))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|err| AppError::Other(format!("failed to read response body: {err}")))?;

        if !status.is_success() {
            warn!(
                provider = self.name(),
                %url,
                status = status.as_u16(),
                "list_models failed"
            );
            return Err(AppError::ApiError {
                status: status.as_u16(),
                body,
            });
        }

        let value: serde_json::Value = serde_json::from_str(&body)?;
        let items = value["data"]
            .as_array()
            .or_else(|| value.as_array())
            .ok_or_else(|| AppError::Other("models response missing data array".to_string()))?;

        Ok(items
            .iter()
            .filter_map(|item| {
                let id = item["id"].as_str()?.to_string();
                let name = item["name"]
                    .as_str()
                    .map(str::to_string)
                    .unwrap_or_else(|| id.clone());
                let context_length = item["context_length"]
                    .as_u64()
                    .or_else(|| item["max_context_length"].as_u64())
                    .map(|value| value as u32);

                Some(ModelInfo {
                    id,
                    name,
                    context_length,
                    supports_vision: item["supports_vision"].as_bool().unwrap_or(false),
                    supports_function_calling: item["supports_function_calling"]
                        .as_bool()
                        .unwrap_or(true),
                })
            })
            .collect())
    }

    async fn chat(&self, req: &ChatRequest) -> Result<ChatResponse> {
        let mapped_messages = self.map_messages(req);
        let first_role = mapped_messages
            .first()
            .and_then(|value| value.get("role"))
            .and_then(|value| value.as_str())
            .unwrap_or("none")
            .to_string();
        let mut payload = serde_json::json!({
            "model": req.model,
            "messages": mapped_messages,
            "stream": false,
        });

        if let Some(temperature) = req.params.temperature {
            payload["temperature"] = serde_json::json!(temperature);
        }
        if let Some(top_p) = req.params.top_p {
            payload["top_p"] = serde_json::json!(top_p);
        }
        if let Some(max_tokens) = req.params.max_tokens {
            payload["max_tokens"] = serde_json::json!(max_tokens);
        }

        let url = self.endpoint(&self.chat_path);
        debug!(
            provider = self.name(),
            %url,
            model = %req.model,
            history_messages = req.messages.len(),
            payload_messages = payload["messages"].as_array().map(|items| items.len()).unwrap_or(0),
            payload_first_role = %first_role,
            system_prompt_present = req.system_prompt.as_ref().map(|value| !value.trim().is_empty()).unwrap_or(false),
            system_prompt_len = req.system_prompt.as_ref().map(|value| value.len()).unwrap_or(0),
            api_key_present = self.api_key.as_ref().map(|value| !value.is_empty()).unwrap_or(false),
            api_key_len = self.api_key.as_ref().map(|value| value.len()).unwrap_or(0),
            "sending chat request"
        );

        let mut request = self.client.post(&url);
        if let Some(api_key) = self.api_key.as_deref().filter(|value| !value.is_empty()) {
            request = request.bearer_auth(api_key);
        }

        let response = request
            .json(&payload)
            .send()
            .await
            .map_err(|err| AppError::Other(format!("request failed: {err}")))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|err| AppError::Other(format!("failed to read response body: {err}")))?;

        if !status.is_success() {
            warn!(
                provider = self.name(),
                %url,
                model = %req.model,
                status = status.as_u16(),
                "chat request failed"
            );
            return Err(AppError::ApiError {
                status: status.as_u16(),
                body,
            });
        }

        let value: serde_json::Value = serde_json::from_str(&body)?;
        let choice = value["choices"]
            .as_array()
            .and_then(|choices| choices.first())
            .ok_or_else(|| AppError::Other("OpenAI response missing choices".to_string()))?;

        let content = choice["message"]["content"]
            .as_str()
            .unwrap_or_default()
            .to_string();
        let finish_reason = choice["finish_reason"].as_str().map(str::to_string);
        let tool_calls = choice["message"]["tool_calls"]
            .as_array()
            .map(|calls| {
                calls
                    .iter()
                    .map(|call| ToolCall {
                        id: call["id"].as_str().unwrap_or_default().to_string(),
                        name: call["function"]["name"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string(),
                        arguments: parse_arguments(&call["function"]["arguments"]),
                    })
                    .collect::<Vec<_>>()
            })
            .filter(|calls| !calls.is_empty());

        let usage = value["usage"].as_object().map(|usage| TokenUsage {
            prompt_tokens: usage
                .get("prompt_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or_default() as u32,
            completion_tokens: usage
                .get("completion_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or_default() as u32,
            total_tokens: usage
                .get("total_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or_default() as u32,
        });

        Ok(ChatResponse {
            content,
            tool_calls,
            usage,
            finish_reason,
        })
    }

    async fn chat_stream(&self, req: &ChatRequest, tx: mpsc::Sender<StreamEvent>) -> Result<()> {
        let response = self.chat(req).await?;
        if !response.content.is_empty() {
            tx.send(StreamEvent::Delta {
                text: response.content.clone(),
            })
            .await
            .ok();
        }
        tx.send(StreamEvent::Done {
            usage: response.usage.clone(),
            finish_reason: response
                .finish_reason
                .clone()
                .unwrap_or_else(|| "stop".to_string()),
        })
        .await
        .ok();
        Ok(())
    }

    fn supports_function_calling(&self) -> bool {
        true
    }

    fn format_tools(&self, tools: &[ToolDef]) -> serde_json::Value {
        serde_json::json!({
            "tools": tools.iter().map(|tool| {
                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": tool.name,
                        "description": tool.description,
                        "parameters": tool.parameters,
                    }
                })
            }).collect::<Vec<_>>(),
            "tool_choice": "auto"
        })
    }

    async fn health_check(&self) -> Result<()> {
        self.list_models().await?;
        Ok(())
    }
}

fn map_message(message: &Message) -> serde_json::Value {
    serde_json::json!({
        "role": message.role.as_str(),
        "content": message.content.as_text(),
    })
}

fn parse_arguments(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::String(text) => serde_json::from_str(text).unwrap_or_else(|_| {
            serde_json::json!({
                "raw": text
            })
        }),
        other => other.clone(),
    }
}
