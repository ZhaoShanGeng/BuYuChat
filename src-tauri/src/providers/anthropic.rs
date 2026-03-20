use async_trait::async_trait;
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Client,
};
use serde_json::{json, Map, Value};

use crate::domain::api_channels::{ApiChannel, ApiChannelModel};
use crate::domain::messages::{
    MessageRole, ProviderCapabilities, ProviderChatEvent, ProviderChatMessage, ProviderChatRequest,
    ProviderChatResponse, ProviderMessagePart, ProviderMessagePartKind,
};
use crate::support::error::{AppError, Result};
use crate::support::ids;

use super::{common, ChatProvider, ProviderEventCallback};

pub struct AnthropicProvider {
    client: Client,
}

impl AnthropicProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

#[async_trait]
impl ChatProvider for AnthropicProvider {
    fn provider_type(&self) -> &'static str {
        "anthropic"
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            supports_streaming: true,
        }
    }

    async fn test_connection(&self, channel: &ApiChannel) -> Result<()> {
        let _ = self.list_models(channel).await?;
        Ok(())
    }

    async fn list_models(&self, channel: &ApiChannel) -> Result<Vec<ApiChannelModel>> {
        let url = common::build_endpoint_url(
            channel,
            channel.models_endpoint.as_deref().unwrap_or("/v1/models"),
        )?;
        let response = self
            .client
            .get(url)
            .headers(build_headers(channel)?)
            .send()
            .await
            .map_err(|err| AppError::Other(format!("provider request failed: {err}")))?;

        if !response.status().is_success() {
            return Err(AppError::Validation(format!(
                "provider models request failed [{}]: {}",
                response.status().as_u16(),
                response.status()
            )));
        }

        let body = response
            .json::<Value>()
            .await
            .map_err(|err| AppError::Other(format!("failed to decode models response: {err}")))?;
        let items = body
            .get("data")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        let mut models = Vec::with_capacity(items.len());
        for item in items {
            let model_id = item.get("id").and_then(Value::as_str).ok_or_else(|| {
                AppError::Validation("provider model entry missing id".to_string())
            })?;
            models.push(ApiChannelModel {
                id: ids::new_id(),
                channel_id: channel.id.clone(),
                model_id: model_id.to_string(),
                display_name: item
                    .get("display_name")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                model_type: Some("chat".to_string()),
                context_window: item.get("context_window").and_then(Value::as_i64),
                max_output_tokens: item.get("max_output_tokens").and_then(Value::as_i64),
                capabilities_json: item
                    .get("capabilities")
                    .cloned()
                    .unwrap_or_else(|| json!({})),
                pricing_json: item.get("pricing").cloned().unwrap_or_else(|| json!({})),
                default_parameters_json: json!({}),
                sort_order: 0,
                config_json: item,
            });
        }

        Ok(models)
    }

    async fn chat(&self, req: ProviderChatRequest) -> Result<ProviderChatResponse> {
        let url = common::build_endpoint_url(
            &req.api_channel,
            req.api_channel
                .chat_endpoint
                .as_deref()
                .unwrap_or("/v1/messages"),
        )?;
        let body = build_request_body(&req, false)?;

        let response = self
            .client
            .post(url)
            .headers(build_headers(&req.api_channel)?)
            .json(&body)
            .send()
            .await
            .map_err(|err| AppError::Other(format!("provider request failed: {err}")))?;

        let status = response.status();
        let body = response
            .json::<Value>()
            .await
            .map_err(|err| AppError::Other(format!("failed to decode provider response: {err}")))?;

        if !status.is_success() {
            return Err(AppError::Validation(format!(
                "API request failed [{}]: {}",
                status.as_u16(),
                body
            )));
        }

        Ok(parse_chat_response(body))
    }

    async fn chat_stream(
        &self,
        req: ProviderChatRequest,
        on_event: &mut ProviderEventCallback<'_>,
    ) -> Result<ProviderChatResponse> {
        let url = common::build_endpoint_url(
            &req.api_channel,
            req.api_channel
                .stream_endpoint
                .as_deref()
                .or(req.api_channel.chat_endpoint.as_deref())
                .unwrap_or("/v1/messages"),
        )?;
        let body = build_request_body(&req, true)?;

        let mut response = self
            .client
            .post(url)
            .headers(build_headers(&req.api_channel)?)
            .json(&body)
            .send()
            .await
            .map_err(|err| AppError::Other(format!("provider request failed: {err}")))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "<stream error body unavailable>".to_string());
            return Err(AppError::Validation(format!(
                "API streaming request failed [{}]: {}",
                status.as_u16(),
                error_text
            )));
        }

        let mut buffer = String::new();
        let mut final_text = String::new();
        let mut finish_reason = None;
        let mut prompt_tokens = None;
        let mut completion_tokens = None;
        let mut total_tokens = None;
        let mut chunk_count = 0_i64;

        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|err| AppError::Other(format!("failed to read stream chunk: {err}")))?
        {
            buffer.push_str(&String::from_utf8_lossy(&chunk).replace("\r\n", "\n"));
            while let Some(event) = common::take_next_sse_event(&mut buffer) {
                if event.data.trim().is_empty() {
                    continue;
                }

                let payload = serde_json::from_str::<Value>(&event.data).map_err(|err| {
                    AppError::Other(format!("failed to decode provider stream event: {err}"))
                })?;
                chunk_count += 1;

                match payload.get("type").and_then(Value::as_str) {
                    Some("message_start") => {
                        prompt_tokens = payload
                            .get("message")
                            .and_then(|message| message.get("usage"))
                            .and_then(|usage| usage.get("input_tokens"))
                            .and_then(Value::as_i64);
                    }
                    Some("content_block_delta") => {
                        let text = payload
                            .get("delta")
                            .and_then(|delta| delta.get("text"))
                            .and_then(Value::as_str);
                        if let Some(text) = text {
                            final_text.push_str(text);
                            on_event(ProviderChatEvent::Delta {
                                parts: vec![ProviderMessagePart {
                                    kind: ProviderMessagePartKind::Text,
                                    text: Some(text.to_string()),
                                    content: None,
                                    metadata_json: payload.clone(),
                                }],
                                raw_event_json: Some(payload),
                            })?;
                        }
                    }
                    Some("message_delta") => {
                        finish_reason = payload
                            .get("delta")
                            .and_then(|delta| delta.get("stop_reason"))
                            .and_then(Value::as_str)
                            .map(str::to_string)
                            .or(finish_reason);
                        completion_tokens = payload
                            .get("usage")
                            .and_then(|usage| usage.get("output_tokens"))
                            .and_then(Value::as_i64)
                            .or(completion_tokens);
                    }
                    Some("message_stop")
                        if event.event.as_deref() == Some("message_stop")
                            || payload.get("type").and_then(Value::as_str)
                                == Some("message_stop") =>
                    {
                        total_tokens = match (prompt_tokens, completion_tokens) {
                            (Some(input), Some(output)) => Some(input + output),
                            _ => total_tokens,
                        };
                        let raw_event_json = Some(json!({
                            "streamed": true,
                            "chunk_count": chunk_count,
                        }));
                        on_event(ProviderChatEvent::Finished {
                            finish_reason: finish_reason.clone(),
                            prompt_tokens,
                            completion_tokens,
                            total_tokens,
                            raw_event_json: raw_event_json.clone(),
                        })?;
                        return Ok(ProviderChatResponse {
                            parts: vec![ProviderMessagePart {
                                kind: ProviderMessagePartKind::Text,
                                text: Some(final_text),
                                content: None,
                                metadata_json: json!({}),
                            }],
                            finish_reason,
                            prompt_tokens,
                            completion_tokens,
                            total_tokens,
                            raw_response_json: raw_event_json,
                        });
                    }
                    _ => {}
                }
            }
        }

        Err(AppError::Other(
            "provider stream ended before receiving message_stop".to_string(),
        ))
    }
}

fn build_headers(channel: &ApiChannel) -> Result<HeaderMap> {
    let mut headers = common::build_headers(channel)?;
    headers.insert(
        HeaderName::from_static("anthropic-version"),
        HeaderValue::from_str(
            channel
                .config_json
                .get("anthropic_version")
                .and_then(Value::as_str)
                .unwrap_or("2023-06-01"),
        )
        .map_err(|err| AppError::Validation(format!("invalid anthropic-version header: {err}")))?,
    );
    Ok(headers)
}

fn build_request_body(req: &ProviderChatRequest, stream: bool) -> Result<Value> {
    let mut system_lines = Vec::new();
    let mut messages = Vec::new();

    for message in &req.messages {
        if matches!(message.role, MessageRole::System) {
            for part in &message.parts {
                system_lines.push(part_to_text(part)?);
            }
            continue;
        }
        messages.push(message_to_anthropic_value(message)?);
    }

    let mut body = Map::new();
    body.insert(
        "model".to_string(),
        Value::String(req.api_channel_model.model_id.clone()),
    );
    body.insert(
        "max_tokens".to_string(),
        req.request_parameters_json
            .get("max_tokens")
            .cloned()
            .or_else(|| req.api_channel_model.max_output_tokens.map(Value::from))
            .unwrap_or_else(|| Value::from(1024)),
    );
    body.insert("messages".to_string(), Value::Array(messages));
    body.insert("stream".to_string(), Value::Bool(stream));

    if !system_lines.is_empty() {
        body.insert(
            "system".to_string(),
            Value::String(system_lines.join("\n\n")),
        );
    }

    if let Some(params) = req.request_parameters_json.as_object() {
        for (key, value) in params {
            if key != "max_tokens" {
                body.insert(key.clone(), value.clone());
            }
        }
    }

    Ok(Value::Object(body))
}

fn message_to_anthropic_value(message: &ProviderChatMessage) -> Result<Value> {
    let role = match message.role {
        MessageRole::Assistant => "assistant",
        MessageRole::User | MessageRole::Tool | MessageRole::System => "user",
    };
    let content = message
        .parts
        .iter()
        .map(part_to_anthropic_value)
        .collect::<Result<Vec<_>>>()?;
    Ok(json!({
        "role": role,
        "content": content,
    }))
}

fn part_to_anthropic_value(part: &ProviderMessagePart) -> Result<Value> {
    Ok(json!({
        "type": "text",
        "text": part_to_text(part)?,
    }))
}

fn part_to_text(part: &ProviderMessagePart) -> Result<String> {
    match part.kind {
        ProviderMessagePartKind::Text => Ok(common::part_text(part).unwrap_or_default()),
        ProviderMessagePartKind::ImageRef => common::binary_part_text_fallback(part, "image"),
        ProviderMessagePartKind::AudioRef => common::binary_part_text_fallback(part, "audio"),
        ProviderMessagePartKind::VideoRef => common::binary_part_text_fallback(part, "video"),
        ProviderMessagePartKind::FileRef => common::binary_part_text_fallback(part, "file"),
        ProviderMessagePartKind::JsonPayload
        | ProviderMessagePartKind::ToolRequest
        | ProviderMessagePartKind::ToolResponse
        | ProviderMessagePartKind::RagExcerpt
        | ProviderMessagePartKind::McpPayload
        | ProviderMessagePartKind::PluginPayload
        | ProviderMessagePartKind::ReasoningTrace
        | ProviderMessagePartKind::ProviderSignature => common::part_text(part)
            .or_else(|| {
                part.content
                    .as_ref()
                    .and_then(|content| content.preview_text.clone())
            })
            .ok_or_else(|| {
                AppError::Validation(format!(
                    "provider part {:?} requires textual representation",
                    part.kind
                ))
            }),
    }
}

fn parse_chat_response(body: Value) -> ProviderChatResponse {
    let mut parts = Vec::new();
    if let Some(items) = body.get("content").and_then(Value::as_array) {
        for item in items {
            if let Some(text) = item.get("text").and_then(Value::as_str) {
                parts.push(ProviderMessagePart {
                    kind: ProviderMessagePartKind::Text,
                    text: Some(text.to_string()),
                    content: None,
                    metadata_json: item.clone(),
                });
            }
        }
    }
    if parts.is_empty() {
        parts.push(ProviderMessagePart {
            kind: ProviderMessagePartKind::Text,
            text: Some(String::new()),
            content: None,
            metadata_json: json!({}),
        });
    }

    let prompt_tokens = body
        .get("usage")
        .and_then(|usage| usage.get("input_tokens"))
        .and_then(Value::as_i64);
    let completion_tokens = body
        .get("usage")
        .and_then(|usage| usage.get("output_tokens"))
        .and_then(Value::as_i64);

    ProviderChatResponse {
        parts,
        finish_reason: body
            .get("stop_reason")
            .and_then(Value::as_str)
            .map(str::to_string),
        prompt_tokens,
        completion_tokens,
        total_tokens: match (prompt_tokens, completion_tokens) {
            (Some(input), Some(output)) => Some(input + output),
            _ => None,
        },
        raw_response_json: Some(body),
    }
}
