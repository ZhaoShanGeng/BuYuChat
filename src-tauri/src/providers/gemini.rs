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

pub struct GeminiProvider {
    client: Client,
}

impl GeminiProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

#[async_trait]
impl ChatProvider for GeminiProvider {
    fn provider_type(&self) -> &'static str {
        "gemini"
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
            channel
                .models_endpoint
                .as_deref()
                .unwrap_or("/v1beta/models"),
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
            .get("models")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        let mut models = Vec::with_capacity(items.len());
        for item in items {
            let model_name = item.get("name").and_then(Value::as_str).ok_or_else(|| {
                AppError::Validation("provider model entry missing name".to_string())
            })?;
            let model_id = model_name
                .rsplit('/')
                .next()
                .unwrap_or(model_name)
                .to_string();
            models.push(ApiChannelModel {
                id: ids::new_id(),
                channel_id: channel.id.clone(),
                model_id: model_id.clone(),
                display_name: item
                    .get("displayName")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .or(Some(model_id)),
                model_type: Some("chat".to_string()),
                context_window: item
                    .get("inputTokenLimit")
                    .or_else(|| item.get("contextWindow"))
                    .and_then(Value::as_i64),
                max_output_tokens: item.get("outputTokenLimit").and_then(Value::as_i64),
                capabilities_json: item
                    .get("supportedGenerationMethods")
                    .cloned()
                    .map(|methods| json!({ "supported_generation_methods": methods }))
                    .unwrap_or_else(|| json!({})),
                pricing_json: json!({}),
                default_parameters_json: json!({}),
                sort_order: 0,
                config_json: item,
            });
        }

        Ok(models)
    }

    async fn chat(&self, req: ProviderChatRequest) -> Result<ProviderChatResponse> {
        let url = build_generate_url(&req.api_channel, &req.api_channel_model.model_id, false)?;
        let body = build_request_body(&req)?;

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
        let url = build_generate_url(&req.api_channel, &req.api_channel_model.model_id, true)?;
        let body = build_request_body(&req)?;

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

                if let Some(text) = extract_candidate_text(&payload) {
                    final_text.push_str(&text);
                    on_event(ProviderChatEvent::Delta {
                        parts: vec![ProviderMessagePart {
                            kind: ProviderMessagePartKind::Text,
                            text: Some(text),
                            content: None,
                            metadata_json: payload.clone(),
                        }],
                        raw_event_json: Some(payload.clone()),
                    })?;
                }

                if let Some((prompt, completion, total)) = extract_usage(&payload) {
                    prompt_tokens = prompt.or(prompt_tokens);
                    completion_tokens = completion.or(completion_tokens);
                    total_tokens = total.or(total_tokens);
                }

                finish_reason = payload
                    .get("candidates")
                    .and_then(Value::as_array)
                    .and_then(|items| items.first())
                    .and_then(|candidate| candidate.get("finishReason"))
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .or(finish_reason);
            }
        }

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
        Ok(ProviderChatResponse {
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
        })
    }
}

fn build_headers(channel: &ApiChannel) -> Result<HeaderMap> {
    let mut headers = common::build_headers(channel)?;
    if let Some(api_key) = &channel.api_key {
        headers.insert(
            HeaderName::from_static("x-goog-api-key"),
            HeaderValue::from_str(api_key).map_err(|err| {
                AppError::Validation(format!("invalid x-goog-api-key header: {err}"))
            })?,
        );
    }
    Ok(headers)
}

fn build_generate_url(channel: &ApiChannel, model_id: &str, stream: bool) -> Result<reqwest::Url> {
    let endpoint = if stream {
        channel
            .stream_endpoint
            .clone()
            .unwrap_or_else(|| format!("/v1beta/models/{model_id}:streamGenerateContent?alt=sse"))
    } else {
        channel
            .chat_endpoint
            .clone()
            .unwrap_or_else(|| format!("/v1beta/models/{model_id}:generateContent"))
    };
    common::build_endpoint_url(channel, &endpoint.replace("{model}", model_id))
}

fn build_request_body(req: &ProviderChatRequest) -> Result<Value> {
    let mut contents = Vec::new();
    let mut system_parts = Vec::new();

    for message in &req.messages {
        if matches!(message.role, MessageRole::System) {
            for part in &message.parts {
                system_parts.push(json!({ "text": part_to_text(part)? }));
            }
            continue;
        }
        contents.push(message_to_gemini_value(message)?);
    }

    let mut body = Map::new();
    body.insert("contents".to_string(), Value::Array(contents));
    if !system_parts.is_empty() {
        body.insert(
            "systemInstruction".to_string(),
            json!({
                "role": "system",
                "parts": system_parts,
            }),
        );
    }
    if let Some(params) = req
        .request_parameters_json
        .as_object()
        .filter(|item| !item.is_empty())
    {
        body.insert(
            "generationConfig".to_string(),
            Value::Object(params.clone()),
        );
    }

    Ok(Value::Object(body))
}

fn message_to_gemini_value(message: &ProviderChatMessage) -> Result<Value> {
    let role = match message.role {
        MessageRole::Assistant => "model",
        MessageRole::User | MessageRole::Tool | MessageRole::System => "user",
    };
    let parts = message
        .parts
        .iter()
        .map(|part| Ok(json!({ "text": part_to_text(part)? })))
        .collect::<Result<Vec<Value>>>()?;

    Ok(json!({
        "role": role,
        "parts": parts,
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
    let prompt_tokens = body
        .get("usageMetadata")
        .and_then(|usage| usage.get("promptTokenCount"))
        .and_then(Value::as_i64);
    let completion_tokens = body
        .get("usageMetadata")
        .and_then(|usage| usage.get("candidatesTokenCount"))
        .and_then(Value::as_i64);
    let total_tokens = body
        .get("usageMetadata")
        .and_then(|usage| usage.get("totalTokenCount"))
        .and_then(Value::as_i64);

    let text = extract_candidate_text(&body).unwrap_or_default();
    ProviderChatResponse {
        parts: vec![ProviderMessagePart {
            kind: ProviderMessagePartKind::Text,
            text: Some(text),
            content: None,
            metadata_json: json!({}),
        }],
        finish_reason: body
            .get("candidates")
            .and_then(Value::as_array)
            .and_then(|items| items.first())
            .and_then(|candidate| candidate.get("finishReason"))
            .and_then(Value::as_str)
            .map(str::to_string),
        prompt_tokens,
        completion_tokens,
        total_tokens,
        raw_response_json: Some(body),
    }
}

fn extract_candidate_text(payload: &Value) -> Option<String> {
    let parts = payload
        .get("candidates")
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .and_then(|candidate| candidate.get("content"))
        .and_then(|content| content.get("parts"))
        .and_then(Value::as_array)?;

    let text = parts
        .iter()
        .filter_map(|part| part.get("text").and_then(Value::as_str))
        .collect::<String>();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

fn extract_usage(payload: &Value) -> Option<(Option<i64>, Option<i64>, Option<i64>)> {
    payload.get("usageMetadata").map(|usage| {
        (
            usage.get("promptTokenCount").and_then(Value::as_i64),
            usage.get("candidatesTokenCount").and_then(Value::as_i64),
            usage.get("totalTokenCount").and_then(Value::as_i64),
        )
    })
}
