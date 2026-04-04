//! 原生 OpenAI-compatible AI 客户端适配层。

use std::{collections::VecDeque, sync::OnceLock};

use async_trait::async_trait;
use dashmap::DashMap;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::{sync::mpsc, task::JoinHandle};

use crate::{
    channel_types::config_for,
    error::{AppError, AppErrorDetails},
    models::{
        Channel, FileAttachment, ImageAttachment, PromptMessage, RemoteModelInfo, ToolCallDelta,
        ToolCallRecord,
    },
};

static API_KEY_ROTATION: OnceLock<DashMap<String, usize>> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReasoningEffort {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiChannelConfig {
    pub provider_name: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub api_keys: Vec<String>,
    pub auth_type: String,
    pub models_endpoint: String,
    pub chat_endpoint: String,
    pub stream_endpoint: String,
    pub model_name: Option<String>,
    pub temperature: Option<String>,
    pub top_p: Option<String>,
}

impl AiChannelConfig {
    pub fn with_model_name(mut self, model_name: impl Into<String>) -> Self {
        self.model_name = Some(model_name.into());
        self
    }

    pub fn with_sampling(mut self, temperature: Option<String>, top_p: Option<String>) -> Self {
        self.temperature = temperature;
        self.top_p = top_p;
        self
    }

    pub fn rotated_api_key(&self) -> Option<String> {
        let mut keys = self
            .api_keys
            .iter()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();

        if keys.is_empty() {
            if let Some(api_key) = self
                .api_key
                .clone()
                .filter(|value| !value.trim().is_empty())
            {
                keys.push(api_key);
            }
        }

        if keys.is_empty() {
            return None;
        }

        if keys.len() == 1 {
            return keys.into_iter().next();
        }

        let rotation = API_KEY_ROTATION.get_or_init(DashMap::new);
        let key = format!("{}|{}", self.provider_name, self.base_url);
        let mut entry = rotation.entry(key).or_insert(0);
        let current = *entry;
        *entry = current.wrapping_add(1);
        Some(keys[current % keys.len()].clone())
    }
}

impl TryFrom<&Channel> for AiChannelConfig {
    type Error = AppError;

    fn try_from(channel: &Channel) -> Result<Self, Self::Error> {
        let defaults = config_for(&channel.channel_type)?;
        let api_keys = channel
            .api_keys
            .as_deref()
            .and_then(|value| serde_json::from_str::<Vec<String>>(value).ok())
            .unwrap_or_default();

        Ok(Self {
            provider_name: channel.name.clone(),
            base_url: channel.base_url.trim().to_string(),
            api_key: channel.api_key.clone(),
            api_keys,
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
            stream_endpoint: channel
                .stream_endpoint
                .clone()
                .unwrap_or_else(|| defaults.stream_endpoint.to_string()),
            model_name: None,
            temperature: None,
            top_p: None,
        })
    }
}

#[async_trait]
pub trait AiMetadataClient: Send + Sync {
    async fn probe_models_endpoint(
        &self,
        http_client: &reqwest::Client,
        config: &AiChannelConfig,
    ) -> Result<(), AppError>;

    async fn fetch_remote_models(
        &self,
        http_client: &reqwest::Client,
        config: &AiChannelConfig,
    ) -> Result<Vec<RemoteModelInfo>, AppError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiChatCompletion {
    pub text: String,
    pub thinking: String,
    pub images: Vec<ImageAttachment>,
    pub files: Vec<FileAttachment>,
    pub tool_calls: Vec<ToolCallRecord>,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub finish_reason: String,
    pub model: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AiStreamChunk {
    Text(String),
    Reasoning(String),
    ToolCallDelta(ToolCallDelta),
    Failed(String),
    Incomplete(String),
}

pub struct AiStreamHandle {
    receiver: mpsc::Receiver<AiStreamChunk>,
    pub model: String,
    finalizer: JoinHandle<Result<AiChatCompletion, AppError>>,
}

impl AiStreamHandle {
    pub async fn next_chunk(&mut self) -> Option<AiStreamChunk> {
        self.receiver.recv().await
    }

    pub async fn finish(self) -> Result<AiChatCompletion, AppError> {
        self.finalizer
            .await
            .map_err(|error| AppError::internal(format!("stream finalizer task failed: {error}")))?
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct AiAdapter;

#[derive(Debug, Clone)]
struct RequestDebugContext {
    request_url: String,
    request_method: String,
    request_body: Option<String>,
}

#[derive(Debug)]
struct ChatRequestResponse {
    response: reqwest::Response,
    debug: RequestDebugContext,
}

impl AiAdapter {
    pub async fn generate_chat(
        &self,
        config: &AiChannelConfig,
        messages: &[PromptMessage],
        max_output_tokens: Option<i64>,
        reasoning_effort: Option<ReasoningEffort>,
        tools: Option<&[serde_json::Value]>,
    ) -> Result<AiChatCompletion, AppError> {
        let response = send_chat_request(
            &reqwest::Client::new(),
            config,
            messages,
            max_output_tokens,
            reasoning_effort,
            false,
            false,
            tools,
        )
        .await?;
        parse_chat_completion_response(response.response, response.debug, config).await
    }

    pub async fn stream_chat(
        &self,
        config: &AiChannelConfig,
        messages: &[PromptMessage],
        max_output_tokens: Option<i64>,
        reasoning_effort: Option<ReasoningEffort>,
        tools: Option<&[serde_json::Value]>,
    ) -> Result<AiStreamHandle, AppError> {
        let response = send_chat_request(
            &reqwest::Client::new(),
            config,
            messages,
            max_output_tokens,
            reasoning_effort,
            true,
            true,
            tools,
        )
        .await?;
        build_stream_handle(response.response, response.debug, config.clone())
    }

    pub async fn finish_stream(
        &self,
        handle: AiStreamHandle,
    ) -> Result<AiChatCompletion, AppError> {
        handle.finish().await
    }
}

#[async_trait]
impl AiMetadataClient for AiAdapter {
    async fn probe_models_endpoint(
        &self,
        http_client: &reqwest::Client,
        config: &AiChannelConfig,
    ) -> Result<(), AppError> {
        execute_models_request(http_client, config)
            .await
            .map(|_| ())
    }

    async fn fetch_remote_models(
        &self,
        http_client: &reqwest::Client,
        config: &AiChannelConfig,
    ) -> Result<Vec<RemoteModelInfo>, AppError> {
        let body = execute_models_request(http_client, config).await?;
        let response: OpenAiModelsResponse = serde_json::from_str(&body).map_err(|error| {
            AppError::ai_request_failed(format!("failed to parse remote model response: {error}"))
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

fn build_stream_handle(
    response: reqwest::Response,
    debug: RequestDebugContext,
    config: AiChannelConfig,
) -> Result<AiStreamHandle, AppError> {
    let (sender, receiver) = mpsc::channel(64);
    let model = config.model_name.clone().unwrap_or_default();
    let initial_model = model.clone();
    let finalizer = tokio::spawn(async move {
        let mut bytes = response.bytes_stream();
        let mut pending = String::new();
        let mut accumulated_text = String::new();
        let mut accumulated_thinking = String::new();
        let mut accumulated_images = Vec::new();
        let mut accumulated_files = Vec::new();
        let mut usage = CompatibleChatUsage::default();
        let mut finish_reason = None;
        let mut tool_calls = ToolCallAccumulator::default();
        let mut response_model = initial_model.clone();

        while let Some(chunk) = bytes.next().await {
            let chunk = chunk.map_err(|error| {
                AppError::ai_request_failed(format!("failed to read stream bytes: {error}"))
                    .with_details(AppErrorDetails {
                        request_url: Some(debug.request_url.clone()),
                        request_method: Some(debug.request_method.clone()),
                        request_body: debug.request_body.clone(),
                        response_status: None,
                        response_body: None,
                        raw_message: None,
                    })
            })?;
            pending.push_str(&String::from_utf8_lossy(&chunk));

            while let Some(event) = take_next_sse_event(&mut pending) {
                let payload = parse_sse_payload(&event);
                if payload.is_empty() {
                    continue;
                }
                if payload == "[DONE]" {
                    return Ok(AiChatCompletion {
                        text: accumulated_text,
                        thinking: accumulated_thinking,
                        images: accumulated_images,
                        files: accumulated_files,
                        tool_calls: tool_calls.finish(),
                        prompt_tokens: usage.prompt_tokens as i64,
                        completion_tokens: usage.completion_tokens as i64,
                        finish_reason: finish_reason.unwrap_or_else(|| "stop".to_string()),
                        model: response_model,
                    });
                }

                let chunk: CompatibleChatCompletionChunk =
                    serde_json::from_str(&payload).map_err(|error| {
                        AppError::ai_request_failed(format!(
                            "failed to parse stream event payload: {error}"
                        ))
                        .with_details(AppErrorDetails {
                            request_url: Some(debug.request_url.clone()),
                            request_method: Some(debug.request_method.clone()),
                            request_body: debug.request_body.clone(),
                            response_status: None,
                            response_body: None,
                            raw_message: Some(payload.clone()),
                        })
                    })?;

                if let Some(model_name) = chunk.model.clone().filter(|value| !value.is_empty()) {
                    response_model = model_name;
                }
                if let Some(chunk_usage) = chunk.usage.clone() {
                    usage = chunk_usage;
                }

                for choice in chunk.choices {
                    if let Some(reason) = choice.finish_reason.clone() {
                        finish_reason = Some(reason);
                    }

                    if let Some(text) = choice.delta.text_delta() {
                        accumulated_text.push_str(&text);
                        let _ = sender.send(AiStreamChunk::Text(text)).await;
                    }

                    if let Some(thinking) = choice.delta.reasoning_delta() {
                        accumulated_thinking.push_str(&thinking);
                        let _ = sender.send(AiStreamChunk::Reasoning(thinking)).await;
                    }

                    accumulated_images.extend(choice.delta.image_attachments());
                    accumulated_files.extend(choice.delta.file_attachments());

                    for delta in choice.delta.tool_call_deltas() {
                        tool_calls.apply(&delta);
                        let _ = sender.send(AiStreamChunk::ToolCallDelta(delta)).await;
                    }
                }
            }
        }

        if !pending.trim().is_empty() {
            let _ = sender
                .send(AiStreamChunk::Incomplete(
                    "stream ended before a complete SSE event was received".to_string(),
                ))
                .await;
        }

        Ok(AiChatCompletion {
            text: accumulated_text,
            thinking: accumulated_thinking,
            images: accumulated_images,
            files: accumulated_files,
            tool_calls: tool_calls.finish(),
            prompt_tokens: usage.prompt_tokens as i64,
            completion_tokens: usage.completion_tokens as i64,
            finish_reason: finish_reason.unwrap_or_else(|| "stop".to_string()),
            model: response_model,
        })
    });

    Ok(AiStreamHandle {
        receiver,
        model,
        finalizer,
    })
}

async fn send_chat_request(
    http_client: &reqwest::Client,
    config: &AiChannelConfig,
    messages: &[PromptMessage],
    max_output_tokens: Option<i64>,
    reasoning_effort: Option<ReasoningEffort>,
    stream: bool,
    use_stream_endpoint: bool,
    tools: Option<&[serde_json::Value]>,
) -> Result<ChatRequestResponse, AppError> {
    let model_name = config.model_name.clone().ok_or_else(|| {
        AppError::validation(
            "VALIDATION_ERROR",
            "model_name is required for chat generation",
        )
    })?;
    let tools_value = tools.filter(|t| !t.is_empty()).map(|t| t.to_vec());
    let request = CompatibleChatCompletionRequest {
        model: model_name,
        messages: to_compatible_messages(messages)?,
        max_completion_tokens: normalize_max_output_tokens(max_output_tokens),
        temperature: parse_optional_f32(config.temperature.as_deref())?,
        top_p: parse_optional_f32(config.top_p.as_deref())?,
        reasoning_effort: reasoning_effort.map(|value| match value {
            ReasoningEffort::Low => "low".to_string(),
            ReasoningEffort::Medium => "medium".to_string(),
            ReasoningEffort::High => "high".to_string(),
        }),
        stream: Some(stream),
        stream_options: stream.then_some(StreamOptions {
            include_usage: true,
        }),
        tools: tools_value,
    };
    let request_body = serde_json::to_string_pretty(&request).map_err(|error| {
        AppError::internal(format!("failed to serialize chat request: {error}"))
    })?;
    let request_url = if use_stream_endpoint {
        build_stream_endpoint_url(config)
    } else {
        build_chat_endpoint_url(config)
    };
    let debug = RequestDebugContext {
        request_url: request_url.clone(),
        request_method: "POST".to_string(),
        request_body: Some(request_body.clone()),
    };

    let mut builder = http_client.post(request_url.clone()).json(&request);

    if let Some((name, value)) = build_auth_header(config)? {
        builder = builder.header(name, value);
    }

    let response = builder.send().await.map_err(|error| {
        AppError::channel_unreachable(format!("failed to reach channel: {error}")).with_details(
            AppErrorDetails {
                request_url: Some(request_url.clone()),
                request_method: Some("POST".to_string()),
                request_body: Some(request_body.clone()),
                response_status: None,
                response_body: None,
                raw_message: Some(error.to_string()),
            },
        )
    })?;
    let status = response.status();

    if status.is_success() {
        return Ok(ChatRequestResponse { response, debug });
    }

    let body = response.text().await.map_err(|error| {
        AppError::ai_request_failed(format!("failed to read remote response: {error}"))
            .with_details(AppErrorDetails {
                request_url: Some(request_url.clone()),
                request_method: Some("POST".to_string()),
                request_body: Some(request_body.clone()),
                response_status: Some(status.as_u16() as i64),
                response_body: None,
                raw_message: Some(error.to_string()),
            })
    })?;
    Err(
        AppError::ai_request_failed(format!("remote endpoint returned {status}: {body}"))
            .with_details(AppErrorDetails {
                request_url: Some(request_url),
                request_method: Some("POST".to_string()),
                request_body: Some(request_body),
                response_status: Some(status.as_u16() as i64),
                response_body: Some(body),
                raw_message: None,
            }),
    )
}

async fn parse_chat_completion_response(
    response: reqwest::Response,
    debug: RequestDebugContext,
    config: &AiChannelConfig,
) -> Result<AiChatCompletion, AppError> {
    let body = response.text().await.map_err(|error| {
        AppError::ai_request_failed(format!("failed to read remote response: {error}"))
            .with_details(AppErrorDetails {
                request_url: Some(debug.request_url.clone()),
                request_method: Some(debug.request_method.clone()),
                request_body: debug.request_body.clone(),
                response_status: None,
                response_body: None,
                raw_message: Some(error.to_string()),
            })
    })?;
    let parsed: CompatibleChatCompletionResponse =
        serde_json::from_str(&body).map_err(|error| {
            AppError::ai_request_failed(format!(
                "failed to parse compatible chat response: {error}"
            ))
            .with_details(AppErrorDetails {
                request_url: Some(debug.request_url.clone()),
                request_method: Some(debug.request_method.clone()),
                request_body: debug.request_body.clone(),
                response_status: None,
                response_body: Some(body.clone()),
                raw_message: None,
            })
        })?;

    let message = parsed
        .choices
        .first()
        .map(|choice| choice.message.clone())
        .unwrap_or_default();

    Ok(AiChatCompletion {
        text: message.text_content(),
        thinking: message.reasoning_content(),
        images: message.image_attachments(),
        files: message.file_attachments(),
        tool_calls: message.tool_calls.unwrap_or_default(),
        prompt_tokens: parsed
            .usage
            .as_ref()
            .map(|value| value.prompt_tokens)
            .unwrap_or(0) as i64,
        completion_tokens: parsed
            .usage
            .as_ref()
            .map(|value| value.completion_tokens)
            .unwrap_or(0) as i64,
        finish_reason: parsed
            .choices
            .first()
            .and_then(|choice| choice.finish_reason.clone())
            .unwrap_or_else(|| "stop".to_string()),
        model: parsed
            .model
            .or_else(|| config.model_name.clone())
            .unwrap_or_default(),
    })
}

#[derive(Debug, Deserialize)]
struct OpenAiModelsResponse {
    data: Vec<OpenAiModelRecord>,
}

#[derive(Debug, Deserialize)]
struct OpenAiModelRecord {
    id: String,
    #[serde(default)]
    display_name: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    context_window: Option<i64>,
}

fn to_compatible_messages(
    messages: &[PromptMessage],
) -> Result<Vec<CompatibleChatMessage>, AppError> {
    let mut result = Vec::new();

    for message in messages {
        let role = match message.role.as_str() {
            "system" | "user" | "assistant" | "tool" => message.role.clone(),
            other => {
                return Err(AppError::internal(format!(
                    "unsupported prompt role '{other}'"
                )))
            }
        };

        if role == "tool" {
            for tool_result in &message.tool_results {
                result.push(CompatibleChatMessage {
                    role: "tool".to_string(),
                    content: CompatibleChatMessageContent::Text(tool_result.content.clone()),
                    tool_calls: None,
                    tool_call_id: Some(tool_result.tool_call_id.clone()),
                    name: Some(tool_result.name.clone()),
                });
            }
            continue;
        }

        let mut parts = Vec::new();
        if !message.content.is_empty() {
            parts.push(CompatibleChatContentPart::Text {
                text: message.content.clone(),
            });
        }
        for image in &message.images {
            parts.push(CompatibleChatContentPart::ImageUrl {
                image_url: CompatibleImageUrl {
                    url: image
                        .url
                        .clone()
                        .filter(|value| !value.trim().is_empty())
                        .unwrap_or_else(|| {
                            format!("data:{};base64,{}", image.mime_type, image.base64)
                        }),
                },
            });
        }
        for file in &message.files {
            parts.push(CompatibleChatContentPart::File {
                file: CompatibleFileContent {
                    filename: file.name.clone(),
                    mime_type: file.mime_type.clone(),
                    data: format!("data:{};base64,{}", file.mime_type, file.base64),
                },
            });
        }

        result.push(CompatibleChatMessage {
            role,
            content: compatible_message_content(message, parts),
            tool_calls: (!message.tool_calls.is_empty())
                .then(|| to_compatible_tool_calls(&message.tool_calls)),
            tool_call_id: None,
            name: None,
        });

        for tool_result in &message.tool_results {
            result.push(CompatibleChatMessage {
                role: "tool".to_string(),
                content: CompatibleChatMessageContent::Text(tool_result.content.clone()),
                tool_calls: None,
                tool_call_id: Some(tool_result.tool_call_id.clone()),
                name: Some(tool_result.name.clone()),
            });
        }
    }

    Ok(result)
}

fn compatible_message_content(
    message: &PromptMessage,
    parts: Vec<CompatibleChatContentPart>,
) -> CompatibleChatMessageContent {
    if parts.is_empty() {
        return CompatibleChatMessageContent::Text(String::new());
    }

    if parts.len() == 1
        && matches!(parts.first(), Some(CompatibleChatContentPart::Text { .. }))
        && message.images.is_empty()
        && message.files.is_empty()
    {
        return CompatibleChatMessageContent::Text(message.content.clone());
    }

    CompatibleChatMessageContent::Parts(parts)
}

fn to_compatible_tool_calls(tool_calls: &[ToolCallRecord]) -> Vec<CompatibleToolCall> {
    tool_calls
        .iter()
        .map(|tool_call| CompatibleToolCall {
            id: tool_call.id.clone(),
            kind: "function",
            function: CompatibleToolFunctionCall {
                name: tool_call.name.clone(),
                arguments: tool_call.arguments_json.clone(),
            },
        })
        .collect()
}

fn normalize_max_output_tokens(max_output_tokens: Option<i64>) -> Option<u32> {
    max_output_tokens
        .filter(|value| *value > 0)
        .and_then(|value| u32::try_from(value).ok())
}

fn parse_optional_f32(value: Option<&str>) -> Result<Option<f32>, AppError> {
    match value.map(str::trim).filter(|value| !value.is_empty()) {
        Some(value) => value.parse::<f32>().map(Some).map_err(|error| {
            AppError::validation(
                "VALIDATION_ERROR",
                format!("invalid sampling parameter '{value}': {error}"),
            )
        }),
        None => Ok(None),
    }
}

fn build_chat_endpoint_url(config: &AiChannelConfig) -> String {
    format!(
        "{}{}",
        config.base_url.trim_end_matches('/'),
        config.chat_endpoint
    )
}

fn build_stream_endpoint_url(config: &AiChannelConfig) -> String {
    format!(
        "{}{}",
        config.base_url.trim_end_matches('/'),
        config.stream_endpoint
    )
}

async fn execute_models_request(
    http_client: &reqwest::Client,
    config: &AiChannelConfig,
) -> Result<String, AppError> {
    let request_url = build_models_endpoint_url(config);
    let mut builder = http_client.get(request_url.clone());

    if let Some((name, value)) = build_auth_header(config)? {
        builder = builder.header(name, value);
    }

    let response = builder.send().await.map_err(|error| {
        AppError::channel_unreachable(format!("failed to reach channel: {error}")).with_details(
            AppErrorDetails {
                request_url: Some(request_url.clone()),
                request_method: Some("GET".to_string()),
                request_body: None,
                response_status: None,
                response_body: None,
                raw_message: Some(error.to_string()),
            },
        )
    })?;
    let status = response.status();
    let body = response.text().await.map_err(|error| {
        AppError::ai_request_failed(format!("failed to read remote response: {error}"))
            .with_details(AppErrorDetails {
                request_url: Some(request_url.clone()),
                request_method: Some("GET".to_string()),
                request_body: None,
                response_status: Some(status.as_u16() as i64),
                response_body: None,
                raw_message: Some(error.to_string()),
            })
    })?;

    if status.is_success() {
        return Ok(body);
    }

    Err(
        AppError::ai_request_failed(format!("remote endpoint returned {status}: {body}"))
            .with_details(AppErrorDetails {
                request_url: Some(request_url),
                request_method: Some("GET".to_string()),
                request_body: None,
                response_status: Some(status.as_u16() as i64),
                response_body: Some(body),
                raw_message: None,
            }),
    )
}

fn build_models_endpoint_url(config: &AiChannelConfig) -> String {
    format!(
        "{}{}",
        config.base_url.trim_end_matches('/'),
        config.models_endpoint
    )
}

fn build_auth_header(config: &AiChannelConfig) -> Result<Option<(&'static str, String)>, AppError> {
    let api_key = config.rotated_api_key();

    match config.auth_type.as_str() {
        "bearer" => Ok(api_key.map(|key| ("Authorization", format!("Bearer {key}")))),
        "x_api_key" => Ok(api_key.map(|key| ("x-api-key", key))),
        "none" => Ok(None),
        other => Err(AppError::validation(
            "VALIDATION_ERROR",
            format!("unsupported auth_type '{other}'"),
        )),
    }
}

fn take_next_sse_event(buffer: &mut String) -> Option<String> {
    if let Some(index) = buffer.find("\n\n") {
        let event = buffer[..index].replace("\r\n", "\n");
        *buffer = buffer[index + 2..].to_string();
        return Some(event);
    }

    if let Some(index) = buffer.find("\r\n\r\n") {
        let event = buffer[..index].replace("\r\n", "\n");
        *buffer = buffer[index + 4..].to_string();
        return Some(event);
    }

    None
}

fn parse_sse_payload(event: &str) -> String {
    event
        .lines()
        .filter_map(|line| line.strip_prefix("data:"))
        .map(str::trim_start)
        .collect::<Vec<_>>()
        .join("\n")
}

#[derive(Debug, Clone, Serialize)]
struct CompatibleChatCompletionRequest {
    model: String,
    messages: Vec<CompatibleChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_completion_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream_options: Option<StreamOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize)]
struct StreamOptions {
    include_usage: bool,
}

#[derive(Debug, Clone, Serialize)]
struct CompatibleChatMessage {
    role: String,
    content: CompatibleChatMessageContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<CompatibleToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct CompatibleToolCall {
    id: String,
    #[serde(rename = "type")]
    kind: &'static str,
    function: CompatibleToolFunctionCall,
}

#[derive(Debug, Clone, Serialize)]
struct CompatibleToolFunctionCall {
    name: String,
    arguments: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
enum CompatibleChatMessageContent {
    Text(String),
    Parts(Vec<CompatibleChatContentPart>),
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum CompatibleChatContentPart {
    Text { text: String },
    ImageUrl { image_url: CompatibleImageUrl },
    File { file: CompatibleFileContent },
}

#[derive(Debug, Clone, Serialize)]
struct CompatibleImageUrl {
    url: String,
}

#[derive(Debug, Clone, Serialize)]
struct CompatibleFileContent {
    filename: String,
    mime_type: String,
    data: String,
}

#[derive(Debug, Clone, Deserialize)]
struct CompatibleChatCompletionResponse {
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    choices: Vec<CompatibleChatChoice>,
    #[serde(default)]
    usage: Option<CompatibleChatUsage>,
}

#[derive(Debug, Clone, Deserialize)]
struct CompatibleChatChoice {
    #[serde(default)]
    message: CompatibleChatResponseMessage,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct CompatibleChatResponseMessage {
    #[serde(default)]
    content: Option<CompatibleResponseContent>,
    #[serde(default)]
    reasoning_content: Option<String>,
    #[serde(default)]
    reasoning: Option<String>,
    #[serde(default)]
    tool_calls: Option<Vec<ToolCallRecord>>,
}

impl CompatibleChatResponseMessage {
    fn text_content(&self) -> String {
        match &self.content {
            Some(CompatibleResponseContent::Text(value)) => value.clone(),
            Some(CompatibleResponseContent::Parts(parts)) => parts
                .iter()
                .filter_map(|part| match part {
                    CompatibleResponseContentPart::Text { text } => Some(text.as_str()),
                    _ => None,
                })
                .collect::<String>(),
            None => String::new(),
        }
    }

    fn reasoning_content(&self) -> String {
        self.reasoning_content
            .clone()
            .or_else(|| self.reasoning.clone())
            .unwrap_or_default()
    }

    fn image_attachments(&self) -> Vec<ImageAttachment> {
        match &self.content {
            Some(CompatibleResponseContent::Parts(parts)) => parts
                .iter()
                .filter_map(CompatibleResponseContentPart::as_image_attachment)
                .collect(),
            _ => Vec::new(),
        }
    }

    fn file_attachments(&self) -> Vec<FileAttachment> {
        match &self.content {
            Some(CompatibleResponseContent::Parts(parts)) => parts
                .iter()
                .filter_map(CompatibleResponseContentPart::as_file_attachment)
                .collect(),
            _ => Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum CompatibleResponseContent {
    Text(String),
    Parts(Vec<CompatibleResponseContentPart>),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum CompatibleResponseContentPart {
    Text { text: String },
    ImageUrl { image_url: serde_json::Value },
    File { file: serde_json::Value },
}

impl CompatibleResponseContentPart {
    fn as_image_attachment(&self) -> Option<ImageAttachment> {
        match self {
            Self::ImageUrl { image_url } => parse_image_attachment_from_value(image_url),
            _ => None,
        }
    }

    fn as_file_attachment(&self) -> Option<FileAttachment> {
        match self {
            Self::File { file } => parse_file_attachment_from_value(file),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
struct CompatibleChatUsage {
    #[serde(default)]
    prompt_tokens: u32,
    #[serde(default)]
    completion_tokens: u32,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct CompatibleChatCompletionChunk {
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    choices: Vec<CompatibleChunkChoice>,
    #[serde(default)]
    usage: Option<CompatibleChatUsage>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct CompatibleChunkChoice {
    #[serde(default)]
    delta: CompatibleChunkDelta,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct CompatibleChunkDelta {
    #[serde(default)]
    content: Option<CompatibleResponseContent>,
    #[serde(default)]
    reasoning_content: Option<String>,
    #[serde(default)]
    reasoning: Option<String>,
    #[serde(default)]
    tool_calls: Vec<CompatibleToolCallDelta>,
}

impl CompatibleChunkDelta {
    fn text_delta(&self) -> Option<String> {
        match &self.content {
            Some(CompatibleResponseContent::Text(value)) if !value.is_empty() => {
                Some(value.clone())
            }
            Some(CompatibleResponseContent::Parts(parts)) => {
                let text = parts
                    .iter()
                    .filter_map(|part| match part {
                        CompatibleResponseContentPart::Text { text } => Some(text.as_str()),
                        _ => None,
                    })
                    .collect::<String>();
                (!text.is_empty()).then_some(text)
            }
            _ => None,
        }
    }

    fn reasoning_delta(&self) -> Option<String> {
        self.reasoning_content
            .clone()
            .or_else(|| self.reasoning.clone())
            .filter(|value| !value.is_empty())
    }

    fn image_attachments(&self) -> Vec<ImageAttachment> {
        match &self.content {
            Some(CompatibleResponseContent::Parts(parts)) => parts
                .iter()
                .filter_map(CompatibleResponseContentPart::as_image_attachment)
                .collect(),
            _ => Vec::new(),
        }
    }

    fn file_attachments(&self) -> Vec<FileAttachment> {
        match &self.content {
            Some(CompatibleResponseContent::Parts(parts)) => parts
                .iter()
                .filter_map(CompatibleResponseContentPart::as_file_attachment)
                .collect(),
            _ => Vec::new(),
        }
    }

    fn tool_call_deltas(&self) -> Vec<ToolCallDelta> {
        self.tool_calls
            .iter()
            .filter_map(|delta| {
                let arguments_delta = delta
                    .function
                    .as_ref()
                    .and_then(|value| value.arguments.clone())
                    .unwrap_or_default();
                let id = delta.id.clone();
                let name = delta.function.as_ref().and_then(|value| value.name.clone());
                if arguments_delta.is_empty() && id.is_none() && name.is_none() {
                    return None;
                }
                Some(ToolCallDelta {
                    id,
                    name,
                    arguments_delta,
                    index: delta.index.unwrap_or(0),
                })
            })
            .collect()
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
struct CompatibleToolCallDelta {
    #[serde(default)]
    index: Option<usize>,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    function: Option<CompatibleToolFunctionDelta>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct CompatibleToolFunctionDelta {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    arguments: Option<String>,
}

#[derive(Debug, Default)]
struct ToolCallAccumulator {
    ordered: VecDeque<(usize, ToolCallRecord)>,
}

impl ToolCallAccumulator {
    fn apply(&mut self, delta: &ToolCallDelta) {
        if let Some((_, existing)) = self
            .ordered
            .iter_mut()
            .find(|(index, _)| *index == delta.index)
        {
            if let Some(id) = &delta.id {
                existing.id = id.clone();
            }
            if let Some(name) = &delta.name {
                existing.name = name.clone();
            }
            existing.arguments_json.push_str(&delta.arguments_delta);
            return;
        }

        self.ordered.push_back((
            delta.index,
            ToolCallRecord {
                id: delta.id.clone().unwrap_or_default(),
                name: delta.name.clone().unwrap_or_default(),
                arguments_json: delta.arguments_delta.clone(),
            },
        ));
    }

    fn finish(self) -> Vec<ToolCallRecord> {
        self.ordered.into_iter().map(|(_, value)| value).collect()
    }
}

fn parse_image_attachment_from_value(value: &serde_json::Value) -> Option<ImageAttachment> {
    let url = value
        .get("url")
        .or_else(|| value.get("image_url"))
        .or_else(|| value.get("data"))
        .or_else(|| value.get("b64_json"))
        .and_then(serde_json::Value::as_str)?;

    if let Some((mime_type, base64)) = parse_data_url(url) {
        return Some(ImageAttachment {
            base64,
            mime_type,
            url: None,
        });
    }

    if let Some(mime_type) = infer_remote_image_mime_type(url, value) {
        return Some(ImageAttachment {
            base64: String::new(),
            mime_type,
            url: Some(url.to_string()),
        });
    }

    None
}

fn parse_file_attachment_from_value(value: &serde_json::Value) -> Option<FileAttachment> {
    let name = value
        .get("filename")
        .or_else(|| value.get("name"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("file")
        .to_string();

    if let Some(data) = value.get("data").and_then(serde_json::Value::as_str) {
        if let Some((mime_type, base64)) = parse_data_url(data) {
            return Some(FileAttachment {
                name,
                base64,
                mime_type,
            });
        }

        let mime_type = value
            .get("mime_type")
            .or_else(|| value.get("mimeType"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or("application/octet-stream")
            .to_string();
        return Some(FileAttachment {
            name,
            base64: data.to_string(),
            mime_type,
        });
    }

    None
}

fn parse_data_url(value: &str) -> Option<(String, String)> {
    let rest = value.strip_prefix("data:")?;
    let (meta, data) = rest.split_once(',')?;
    if !meta.ends_with(";base64") {
        return None;
    }

    let mime_type = meta
        .strip_suffix(";base64")
        .filter(|v| !v.is_empty())
        .unwrap_or("application/octet-stream")
        .to_string();
    Some((mime_type, data.to_string()))
}

fn infer_remote_image_mime_type(url: &str, value: &serde_json::Value) -> Option<String> {
    let trimmed = url.trim();
    if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
        return None;
    }

    if let Some(mime_type) = value
        .get("mime_type")
        .or_else(|| value.get("mimeType"))
        .or_else(|| value.get("content_type"))
        .and_then(serde_json::Value::as_str)
        .filter(|mime| mime.starts_with("image/"))
    {
        return Some(mime_type.to_string());
    }

    let lower = trimmed.to_ascii_lowercase();
    for (suffix, mime_type) in [
        (".png", "image/png"),
        (".jpg", "image/jpeg"),
        (".jpeg", "image/jpeg"),
        (".webp", "image/webp"),
        (".gif", "image/gif"),
        (".bmp", "image/bmp"),
        (".svg", "image/svg+xml"),
    ] {
        if lower.contains(suffix) {
            return Some(mime_type.to_string());
        }
    }

    Some("image/*".to_string())
}

#[cfg(test)]
mod tests {
    use crate::ai::adapter::AiMetadataClient;
    use wiremock::{
        matchers::{header, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use super::{
        compatible_message_content, parse_image_attachment_from_value, to_compatible_messages,
        AiAdapter, AiChannelConfig, CompatibleChatContentPart, CompatibleChatMessageContent,
    };

    fn sample_config(base_url: String) -> AiChannelConfig {
        AiChannelConfig {
            provider_name: "BuYu".to_string(),
            base_url,
            api_key: Some("sk-test".to_string()),
            api_keys: Vec::new(),
            auth_type: "bearer".to_string(),
            models_endpoint: "/v1/models".to_string(),
            chat_endpoint: "/v1/chat/completions".to_string(),
            stream_endpoint: "/v1/chat/completions".to_string(),
            model_name: Some("gpt-4o-mini".to_string()),
            temperature: None,
            top_p: None,
        }
    }

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

    #[test]
    fn ai_channel_config_try_from_channel_applies_defaults() {
        let config = AiChannelConfig::try_from(&crate::models::Channel {
            id: "channel-1".to_string(),
            name: "OpenAI".to_string(),
            channel_type: "openai_compatible".to_string(),
            base_url: "https://api.openai.com".to_string(),
            api_key: Some("sk-test".to_string()),
            api_keys: Some("[\"sk-a\",\"sk-b\"]".to_string()),
            auth_type: None,
            models_endpoint: None,
            chat_endpoint: None,
            stream_endpoint: None,
            thinking_tags: None,
            enabled: true,
            created_at: 100,
            updated_at: 100,
        })
        .unwrap();

        assert_eq!(config.auth_type, "bearer");
        assert_eq!(config.models_endpoint, "/v1/models");
        assert_eq!(config.chat_endpoint, "/v1/chat/completions");
        assert_eq!(config.stream_endpoint, "/v1/chat/completions");
        assert_eq!(
            config.api_keys,
            vec!["sk-a".to_string(), "sk-b".to_string()]
        );
    }

    #[tokio::test]
    async fn fetch_remote_models_parses_openai_compatible_response() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1/models"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    { "id": "gpt-4o", "display_name": "GPT-4o", "context_window": 128000 },
                    { "id": "gpt-4o-mini" }
                ]
            })))
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

    #[test]
    fn assistant_tool_call_message_without_text_serializes_as_empty_string_content() {
        let messages = vec![crate::models::PromptMessage {
            role: "assistant".to_string(),
            content: String::new(),
            images: Vec::new(),
            files: Vec::new(),
            tool_calls: vec![crate::models::ToolCallRecord {
                id: "call-1".to_string(),
                name: "fetch".to_string(),
                arguments_json: "{\"url\":\"https://example.com\"}".to_string(),
            }],
            tool_results: Vec::new(),
        }];

        let compatible = to_compatible_messages(&messages).unwrap();
        assert_eq!(compatible.len(), 1);
        assert!(matches!(
            compatible[0].content,
            CompatibleChatMessageContent::Text(ref value) if value.is_empty()
        ));
    }

    #[test]
    fn assistant_tool_call_message_uses_openai_compatible_tool_call_shape() {
        let messages = vec![crate::models::PromptMessage {
            role: "assistant".to_string(),
            content: String::new(),
            images: Vec::new(),
            files: Vec::new(),
            tool_calls: vec![crate::models::ToolCallRecord {
                id: "call-1".to_string(),
                name: "fetch".to_string(),
                arguments_json: "{\"url\":\"https://example.com\"}".to_string(),
            }],
            tool_results: Vec::new(),
        }];

        let compatible = to_compatible_messages(&messages).unwrap();
        let value = serde_json::to_value(&compatible[0]).unwrap();
        let tool_call = &value["tool_calls"][0];

        assert_eq!(tool_call["id"], "call-1");
        assert_eq!(tool_call["type"], "function");
        assert_eq!(tool_call["function"]["name"], "fetch");
        assert_eq!(
            tool_call["function"]["arguments"],
            "{\"url\":\"https://example.com\"}"
        );
        assert!(tool_call.get("arguments_json").is_none());
        assert!(tool_call.get("name").is_none());
    }

    #[test]
    fn empty_parts_fallback_uses_empty_text_content() {
        let message = crate::models::PromptMessage {
            role: "assistant".to_string(),
            content: String::new(),
            images: Vec::new(),
            files: Vec::new(),
            tool_calls: Vec::new(),
            tool_results: Vec::new(),
        };

        assert!(matches!(
            compatible_message_content(&message, Vec::<CompatibleChatContentPart>::new()),
            CompatibleChatMessageContent::Text(ref value) if value.is_empty()
        ));
    }

    #[test]
    fn parse_image_attachment_supports_remote_image_urls() {
        let attachment = parse_image_attachment_from_value(&serde_json::json!({
            "url": "https://cdn.example.com/generated/demo.webp"
        }))
        .unwrap();

        assert_eq!(attachment.base64, "");
        assert_eq!(attachment.mime_type, "image/webp");
        assert_eq!(
            attachment.url.as_deref(),
            Some("https://cdn.example.com/generated/demo.webp")
        );
    }
}
