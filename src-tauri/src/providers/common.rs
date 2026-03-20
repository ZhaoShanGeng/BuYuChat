use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION},
    Url,
};
use serde_json::{Map, Value};

use crate::domain::api_channels::ApiChannel;
use crate::domain::messages::ProviderMessagePart;
use crate::support::error::{AppError, Result};

#[derive(Debug, Clone)]
pub struct SseEvent {
    pub event: Option<String>,
    pub data: String,
}

pub fn build_endpoint_url(channel: &ApiChannel, endpoint: &str) -> Result<Url> {
    let base = channel.base_url.trim_end_matches('/');
    let endpoint = if endpoint.starts_with('/') {
        endpoint.to_string()
    } else {
        format!("/{endpoint}")
    };
    Url::parse(&format!("{base}{endpoint}"))
        .map_err(|err| AppError::Validation(format!("invalid provider url: {err}")))
}

pub fn build_headers(channel: &ApiChannel) -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    if let Some(api_key) = &channel.api_key {
        match channel.auth_type.as_str() {
            "bearer" => {
                headers.insert(
                    AUTHORIZATION,
                    HeaderValue::from_str(&format!("Bearer {api_key}")).map_err(|err| {
                        AppError::Validation(format!("invalid authorization header: {err}"))
                    })?,
                );
            }
            "x-api-key" => {
                headers.insert(
                    HeaderName::from_static("x-api-key"),
                    HeaderValue::from_str(api_key).map_err(|err| {
                        AppError::Validation(format!("invalid x-api-key header: {err}"))
                    })?,
                );
            }
            "none" => {}
            other => {
                return Err(AppError::Validation(format!(
                    "unsupported auth_type '{other}'"
                )));
            }
        }
    }

    if let Some(config) = channel.config_json.as_object() {
        if let Some(extra_headers) = config.get("headers").and_then(Value::as_object) {
            for (name, value) in extra_headers {
                if let Some(value) = value.as_str() {
                    headers.insert(
                        HeaderName::from_bytes(name.as_bytes()).map_err(|err| {
                            AppError::Validation(format!(
                                "invalid custom header name '{name}': {err}"
                            ))
                        })?,
                        HeaderValue::from_str(value).map_err(|err| {
                            AppError::Validation(format!(
                                "invalid custom header value for '{name}': {err}"
                            ))
                        })?,
                    );
                }
            }
        }
    }

    Ok(headers)
}

pub fn build_chat_request_body(
    model_id: &str,
    messages: Vec<Value>,
    request_parameters_json: &Value,
    stream: bool,
) -> Value {
    let mut body = Map::new();
    body.insert("model".to_string(), Value::String(model_id.to_string()));
    body.insert("messages".to_string(), Value::Array(messages));
    body.insert("stream".to_string(), Value::Bool(stream));

    if let Some(obj) = request_parameters_json.as_object() {
        for (key, value) in obj {
            body.insert(key.clone(), value.clone());
        }
    }

    Value::Object(body)
}

pub fn take_next_sse_event(buffer: &mut String) -> Option<SseEvent> {
    let marker = "\n\n";
    let idx = buffer.find(marker)?;
    let raw_event = buffer[..idx].to_string();
    *buffer = buffer[idx + marker.len()..].to_string();

    let mut event_name = None;
    let mut data_lines = Vec::new();
    for line in raw_event.lines() {
        if let Some(value) = line.strip_prefix("event:") {
            event_name = Some(value.trim().to_string());
        } else if let Some(value) = line.strip_prefix("data:") {
            data_lines.push(value.trim().to_string());
        }
    }

    Some(SseEvent {
        event: event_name,
        data: data_lines.join("\n"),
    })
}

pub fn part_text(part: &ProviderMessagePart) -> Option<String> {
    part.text.clone().or_else(|| {
        part.content
            .as_ref()
            .and_then(|content| content.text_content.clone())
    })
}

pub fn binary_part_text_fallback(part: &ProviderMessagePart, label: &str) -> Result<String> {
    if let Some(text) = part_text(part) {
        return Ok(text);
    }

    let content = part.content.as_ref().ok_or_else(|| {
        AppError::Validation(format!(
            "{label} part requires content or explicit text representation"
        ))
    })?;

    let mut lines = vec![format!("[{label}]")];
    if let Some(uri) = &content.primary_storage_uri {
        lines.push(format!("uri: {uri}"));
    }
    if let Some(mime) = &content.mime_type {
        lines.push(format!("mime: {mime}"));
    }
    lines.push(format!("size_bytes: {}", content.size_bytes));
    if let Some(preview) = &content.preview_text {
        lines.push(format!("preview: {preview}"));
    }
    if let Some(metadata) = content
        .config_json
        .as_object()
        .filter(|item| !item.is_empty())
    {
        lines.push(format!(
            "metadata: {}",
            serde_json::Value::Object(metadata.clone())
        ));
    }
    Ok(lines.join("\n"))
}
