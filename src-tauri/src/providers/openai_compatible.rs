use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};

use crate::domain::api_channels::{ApiChannel, ApiChannelModel};
use crate::domain::messages::{
    MessageRole, ProviderCapabilities, ProviderChatEvent, ProviderChatMessage, ProviderChatRequest,
    ProviderChatResponse, ProviderMessagePart, ProviderMessagePartKind,
};
use crate::support::error::{AppError, Result};
use crate::support::ids;

use super::{common, ChatProvider, ProviderEventCallback};

pub struct OpenAiCompatibleProvider {
    client: Client,
}

impl OpenAiCompatibleProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

#[async_trait]
impl ChatProvider for OpenAiCompatibleProvider {
    fn provider_type(&self) -> &'static str {
        "openai_compatible"
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
            channel.models_endpoint.as_deref().unwrap_or("/models"),
        )?;
        let response = self
            .client
            .get(url)
            .headers(common::build_headers(channel)?)
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
                    .get("name")
                    .or_else(|| item.get("display_name"))
                    .and_then(Value::as_str)
                    .map(str::to_string),
                model_type: item.get("type").and_then(Value::as_str).map(str::to_string),
                context_window: item.get("context_window").and_then(Value::as_i64),
                max_output_tokens: item
                    .get("max_output_tokens")
                    .or_else(|| item.get("max_tokens"))
                    .and_then(Value::as_i64),
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
                .unwrap_or("/chat/completions"),
        )?;

        let messages = req
            .messages
            .iter()
            .map(message_to_openai_value)
            .collect::<Result<Vec<_>>>()?;

        let body = common::build_chat_request_body(
            &req.api_channel_model.model_id,
            messages,
            &req.request_parameters_json,
            false,
        );

        let response = self
            .client
            .post(url)
            .headers(common::build_headers(&req.api_channel)?)
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

        let choice = body
            .get("choices")
            .and_then(Value::as_array)
            .and_then(|choices| choices.first())
            .ok_or_else(|| {
                AppError::Validation("provider response missing choices[0]".to_string())
            })?;

        let message = choice
            .get("message")
            .ok_or_else(|| AppError::Validation("provider response missing message".to_string()))?;

        Ok(ProviderChatResponse {
            parts: parse_response_parts(message)?,
            finish_reason: choice
                .get("finish_reason")
                .and_then(Value::as_str)
                .map(str::to_string),
            prompt_tokens: body
                .get("usage")
                .and_then(|usage| usage.get("prompt_tokens"))
                .and_then(Value::as_i64),
            completion_tokens: body
                .get("usage")
                .and_then(|usage| usage.get("completion_tokens"))
                .and_then(Value::as_i64),
            total_tokens: body
                .get("usage")
                .and_then(|usage| usage.get("total_tokens"))
                .and_then(Value::as_i64),
            raw_response_json: Some(body),
        })
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
                .unwrap_or("/chat/completions"),
        )?;

        let messages = req
            .messages
            .iter()
            .map(message_to_openai_value)
            .collect::<Result<Vec<_>>>()?;
        let mut body = common::build_chat_request_body(
            &req.api_channel_model.model_id,
            messages,
            &req.request_parameters_json,
            true,
        );
        if let Some(obj) = body.as_object_mut() {
            obj.entry("stream_options".to_string())
                .or_insert_with(|| json!({ "include_usage": true }));
        }

        let mut response = self
            .client
            .post(url)
            .headers(common::build_headers(&req.api_channel)?)
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
                if event.data.trim() == "[DONE]" {
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

                let payload = serde_json::from_str::<Value>(&event.data).map_err(|err| {
                    AppError::Other(format!("failed to decode provider stream event: {err}"))
                })?;
                chunk_count += 1;

                if let Some(usage) = payload.get("usage") {
                    prompt_tokens = usage.get("prompt_tokens").and_then(Value::as_i64);
                    completion_tokens = usage.get("completion_tokens").and_then(Value::as_i64);
                    total_tokens = usage.get("total_tokens").and_then(Value::as_i64);
                }

                let mut delta_parts = Vec::new();
                if let Some(choice) = payload
                    .get("choices")
                    .and_then(Value::as_array)
                    .and_then(|choices| choices.first())
                {
                    finish_reason = choice
                        .get("finish_reason")
                        .and_then(Value::as_str)
                        .map(str::to_string)
                        .or(finish_reason);

                    if let Some(delta) = choice.get("delta") {
                        delta_parts = parse_stream_delta_parts(delta)?;
                        for part in &delta_parts {
                            if let Some(text) = part.text.clone().or_else(|| {
                                part.content
                                    .as_ref()
                                    .and_then(|content| content.text_content.clone())
                            }) {
                                final_text.push_str(&text);
                            }
                        }
                    }
                }

                if !delta_parts.is_empty() {
                    on_event(ProviderChatEvent::Delta {
                        parts: delta_parts,
                        raw_event_json: Some(payload),
                    })?;
                }
            }
        }

        Err(AppError::Other(
            "provider stream ended before receiving [DONE]".to_string(),
        ))
    }
}

fn message_to_openai_value(message: &ProviderChatMessage) -> Result<Value> {
    let content = if message.parts.len() == 1
        && matches!(message.parts[0].kind, ProviderMessagePartKind::Text)
    {
        Value::String(
            message.parts[0]
                .text
                .clone()
                .or_else(|| {
                    message.parts[0]
                        .content
                        .as_ref()
                        .and_then(|content| content.text_content.clone())
                })
                .unwrap_or_default(),
        )
    } else {
        let mut items = Vec::with_capacity(message.parts.len());
        for part in &message.parts {
            items.push(part_to_openai_value(part)?);
        }
        Value::Array(items)
    };

    Ok(json!({
        "role": role_to_provider(message.role),
        "content": content,
    }))
}

fn role_to_provider(role: MessageRole) -> &'static str {
    match role {
        MessageRole::System => "system",
        MessageRole::User => "user",
        MessageRole::Assistant => "assistant",
        MessageRole::Tool => "tool",
    }
}

fn part_to_openai_value(part: &ProviderMessagePart) -> Result<Value> {
    match part.kind {
        ProviderMessagePartKind::Text => Ok(json!({
            "type": "text",
            "text": part
                .text
                .clone()
                .or_else(|| part.content.as_ref().and_then(|content| content.text_content.clone()))
                .unwrap_or_default(),
        })),
        ProviderMessagePartKind::ImageRef => {
            let url = part
                .content
                .as_ref()
                .and_then(|content| content.primary_storage_uri.clone())
                .ok_or_else(|| {
                    AppError::Validation("image_ref part requires primary_storage_uri".to_string())
                })?;
            Ok(json!({
                "type": "image_url",
                "image_url": { "url": url },
            }))
        }
        ProviderMessagePartKind::AudioRef => Ok(json!({
            "type": "text",
            "text": common::binary_part_text_fallback(part, "audio")?,
        })),
        ProviderMessagePartKind::VideoRef => Ok(json!({
            "type": "text",
            "text": common::binary_part_text_fallback(part, "video")?,
        })),
        ProviderMessagePartKind::FileRef => Ok(json!({
            "type": "text",
            "text": common::binary_part_text_fallback(part, "file")?,
        })),
        ProviderMessagePartKind::JsonPayload
        | ProviderMessagePartKind::ToolRequest
        | ProviderMessagePartKind::ToolResponse
        | ProviderMessagePartKind::RagExcerpt
        | ProviderMessagePartKind::McpPayload
        | ProviderMessagePartKind::PluginPayload
        | ProviderMessagePartKind::ReasoningTrace
        | ProviderMessagePartKind::ProviderSignature => {
            let text = common::part_text(part)
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
                })?;
            Ok(json!({
                "type": "text",
                "text": text,
            }))
        }
    }
}

fn parse_response_parts(message: &Value) -> Result<Vec<ProviderMessagePart>> {
    let content = message.get("content");
    match content {
        None | Some(Value::Null) => Ok(vec![ProviderMessagePart {
            kind: ProviderMessagePartKind::Text,
            text: Some(String::new()),
            content: None,
            metadata_json: json!({}),
        }]),
        Some(Value::String(text)) => Ok(vec![ProviderMessagePart {
            kind: ProviderMessagePartKind::Text,
            text: Some(text.clone()),
            content: None,
            metadata_json: json!({}),
        }]),
        Some(Value::Array(items)) => {
            let mut parts = Vec::new();
            for item in items {
                if let Some(text) = item
                    .get("text")
                    .or_else(|| item.get("content"))
                    .and_then(Value::as_str)
                {
                    parts.push(ProviderMessagePart {
                        kind: ProviderMessagePartKind::Text,
                        text: Some(text.to_string()),
                        content: None,
                        metadata_json: item.clone(),
                    });
                }
            }
            if parts.is_empty() {
                return Err(AppError::Validation(
                    "provider response array content did not contain textual items".to_string(),
                ));
            }
            Ok(parts)
        }
        Some(other) => Err(AppError::Validation(format!(
            "unsupported provider response content shape: {other}"
        ))),
    }
}

fn parse_stream_delta_parts(delta: &Value) -> Result<Vec<ProviderMessagePart>> {
    if let Some(text) = delta.get("content").and_then(Value::as_str) {
        return Ok(vec![ProviderMessagePart {
            kind: ProviderMessagePartKind::Text,
            text: Some(text.to_string()),
            content: None,
            metadata_json: json!({}),
        }]);
    }

    if let Some(items) = delta.get("content").and_then(Value::as_array) {
        let mut parts = Vec::new();
        for item in items {
            if let Some(text) = item
                .get("text")
                .or_else(|| item.get("content"))
                .and_then(Value::as_str)
            {
                parts.push(ProviderMessagePart {
                    kind: ProviderMessagePartKind::Text,
                    text: Some(text.to_string()),
                    content: None,
                    metadata_json: item.clone(),
                });
            }
        }
        return Ok(parts);
    }

    Ok(Vec::new())
}
