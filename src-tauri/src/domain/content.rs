use serde::{Deserialize, Serialize};

use crate::domain::common::Id;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentStorageKind {
    Inline,
    Chunked,
    FileRef,
}

impl ContentStorageKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Inline => "inline",
            Self::Chunked => "chunked",
            Self::FileRef => "file_ref",
        }
    }

    pub fn parse(value: &str) -> crate::support::error::Result<Self> {
        match value {
            "inline" => Ok(Self::Inline),
            "chunked" => Ok(Self::Chunked),
            "file_ref" => Ok(Self::FileRef),
            _ => Err(crate::support::error::AppError::Validation(format!(
                "unsupported content storage kind '{value}'"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentType {
    Text,
    Markdown,
    Json,
    Image,
    Audio,
    Video,
    File,
    Html,
    Binary,
    ToolRequest,
    ToolResponse,
    RagExcerpt,
    McpPayload,
    PluginPayload,
    PluginState,
    ReasoningTrace,
    ProviderSignature,
}

impl ContentType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Markdown => "markdown",
            Self::Json => "json",
            Self::Image => "image",
            Self::Audio => "audio",
            Self::Video => "video",
            Self::File => "file",
            Self::Html => "html",
            Self::Binary => "binary",
            Self::ToolRequest => "tool_request",
            Self::ToolResponse => "tool_response",
            Self::RagExcerpt => "rag_excerpt",
            Self::McpPayload => "mcp_payload",
            Self::PluginPayload => "plugin_payload",
            Self::PluginState => "plugin_state",
            Self::ReasoningTrace => "reasoning_trace",
            Self::ProviderSignature => "provider_signature",
        }
    }

    pub fn parse(value: &str) -> crate::support::error::Result<Self> {
        match value {
            "text" => Ok(Self::Text),
            "markdown" => Ok(Self::Markdown),
            "json" => Ok(Self::Json),
            "image" => Ok(Self::Image),
            "audio" => Ok(Self::Audio),
            "video" => Ok(Self::Video),
            "file" => Ok(Self::File),
            "html" => Ok(Self::Html),
            "binary" => Ok(Self::Binary),
            "tool_request" => Ok(Self::ToolRequest),
            "tool_response" => Ok(Self::ToolResponse),
            "rag_excerpt" => Ok(Self::RagExcerpt),
            "mcp_payload" => Ok(Self::McpPayload),
            "plugin_payload" => Ok(Self::PluginPayload),
            "plugin_state" => Ok(Self::PluginState),
            "reasoning_trace" => Ok(Self::ReasoningTrace),
            "provider_signature" => Ok(Self::ProviderSignature),
            _ => Err(crate::support::error::AppError::Validation(format!(
                "unsupported content type '{value}'"
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredContent {
    pub content_id: Id,
    pub content_type: ContentType,
    pub storage_kind: ContentStorageKind,
    pub mime_type: Option<String>,
    pub size_bytes: u64,
    pub preview_text: Option<String>,
    pub primary_storage_uri: Option<String>,
    pub text_content: Option<String>,
    pub chunk_count: u32,
    pub sha256: Option<String>,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentWriteInput {
    pub content_type: ContentType,
    pub mime_type: Option<String>,
    pub text_content: Option<String>,
    pub source_file_path: Option<String>,
    pub primary_storage_uri: Option<String>,
    pub size_bytes_hint: Option<u64>,
    pub preview_text: Option<String>,
    pub config_json: serde_json::Value,
}
