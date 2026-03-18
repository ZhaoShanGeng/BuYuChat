# 跨模块共用类型

> 所有类型定义在 `src-tauri/src/types.rs`，各模块通过 `use crate::types::*` 引入。

## 消息与对话类型

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── Role ──────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[serde(rename_all = "snake_case")]
pub enum Role {
    User,
    Assistant,
    Tool,
    System,
}

// ── 消息内容 ───────────────────────────────────────────

/// 单条内容片段（多模态）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    Text { text: String },
    Image { url: String, mime_type: Option<String> }, // base64 data URL 或远程 URL
}

/// 消息内容：纯文本（多数情况）或多模态
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

impl MessageContent {
    /// 提取纯文本，多模态时拼接所有 Text 片段
    pub fn as_text(&self) -> String {
        match self {
            MessageContent::Text(s) => s.clone(),
            MessageContent::Parts(parts) => parts
                .iter()
                .filter_map(|p| if let ContentPart::Text { text } = p { Some(text.as_str()) } else { None })
                .collect::<Vec<_>>()
                .join(""),
        }
    }
}

// ── 工具调用相关 ───────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,           // LLM 给的随机 ID，tool message 回传时需对应
    pub name: String,         // 工具名
    pub arguments: serde_json::Value, // 已解析的 JSON 参数
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value, // JSON Schema object
}

// ── 消息（Provider 层使用） ────────────────────────────

/// Provider 发送 / 接收时使用的标准消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: MessageContent,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_call_id: Option<String>,
    pub tool_result: Option<String>,
}

// ── 请求 / 响应 ────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelParams {
    pub temperature: Option<f32>,          // 0.0 – 2.0
    pub top_p: Option<f32>,                // 0.0 – 1.0
    pub max_tokens: Option<u32>,
    pub frequency_penalty: Option<f32>,    // -2.0 – 2.0，OpenAI 支持
    pub presence_penalty: Option<f32>,     // -2.0 – 2.0，OpenAI 支持
    /// 任意自定义参数透传给 API，如 `{"repetition_penalty": 1.1}`
    pub custom: HashMap<String, serde_json::Value>,
}

impl Default for ModelParams {
    fn default() -> Self {
        Self {
            temperature: None,
            top_p: None,
            max_tokens: None,
            frequency_penalty: None,
            presence_penalty: None,
            custom: HashMap::new(),
        }
    }
}

/// 发送给 Provider 的完整请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    /// 对于支持独立 system 字段的 Provider（Claude），单独传递；
    /// 对于 OpenAI，视情况插入到 messages[0] 或合并
    pub system_prompt: Option<String>,
    pub params: ModelParams,
    pub tools: Option<Vec<ToolDef>>, // None = 不启用工具
    pub stream: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub usage: Option<TokenUsage>,
    pub finish_reason: Option<String>, // "stop" | "tool_calls" | "length"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

// ── 流式事件 ───────────────────────────────────────────

/// Provider 通过 mpsc::Sender<StreamEvent> 发送，ChatService 中转给 Tauri emit
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    /// 流式文本增量
    Delta { text: String },
    /// LLM 请求工具调用（非流式工具调用补全时也用此）
    ToolCall { call: ToolCall },
    /// 流结束（正常）
    Done { usage: Option<TokenUsage>, finish_reason: String },
    /// 流错误（可恢复，前端显示错误提示）
    Error { message: String },
}

// ── 模型信息 ───────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,           // 如 "gpt-4o"
    pub name: String,         // 显示名，如 "GPT-4o"
    pub context_length: Option<u32>,
    pub supports_vision: bool,
    pub supports_function_calling: bool,
}

// ── RAG 引用 ───────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    pub index: u32,           // 对应 LLM 回复中的 [n] 标记
    pub document_id: String,
    pub document_name: String,
    pub chunk_index: u32,
    pub page_number: Option<u32>,
    pub snippet: String,      // 被引用的原文片段（前 200 字符）
}

// ── IPC 通用包装 ───────────────────────────────────────

/// 分页请求（前端列表接口通用）
#[derive(Debug, Deserialize)]
pub struct PageRequest {
    pub page: u32,
    pub per_page: u32,
}

/// 分页响应
#[derive(Debug, Serialize)]
pub struct PageResponse<T> {
    pub items: Vec<T>,
    pub total: u32,
    pub page: u32,
    pub per_page: u32,
}
```

## 数据库映射结构体

> 这些结构体用于 sqlx 查询结果映射，和 `types.rs` 中的业务类型分开。
> 路径：`src-tauri/src/db/models.rs`

```rust
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct ConversationRow {
    pub id: String,
    pub title: String,
    pub model_id: String,
    pub provider: String,
    pub assistant_id: Option<String>,
    pub system_prompt: Option<String>,
    pub pinned: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct MessageRow {
    pub id: String,
    pub conversation_id: String,
    pub parent_message_id: Option<String>,
    pub version_group_id: String,
    pub version_index: i64,
    pub is_active: bool,
    pub role: String,
    pub content: Option<String>,
    pub content_parts: Option<String>,
    pub tool_calls: Option<String>,
    pub tool_call_id: Option<String>,
    pub citations_json: Option<String>,
    pub tokens_used: Option<i64>,
    pub created_at: i64,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct AssistantRow {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub category: String,
    pub system_prompt: String,
    pub model_id: Option<String>,
    pub provider: Option<String>,
    pub tools_json: Option<String>,
    pub knowledge_base_ids: Option<String>,
    pub params_json: Option<String>,
    pub builtin: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct ProviderConfigRow {
    pub id: String,
    pub provider: String,
    pub api_key_id: Option<String>,
    pub base_url: Option<String>,
    pub extra_json: Option<String>,
    pub enabled: bool,
    pub updated_at: i64,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct ParamPresetRow {
    pub id: String,
    pub name: String,
    pub provider: Option<String>,
    pub params_json: String,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct DocumentRow {
    pub id: String,
    pub name: String,
    pub source_type: String,
    pub path_or_url: String,
    pub chunk_count: i64,
    pub status: String,
    pub error_msg: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct ToolRow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub schema_json: String,
    pub source: String,
    pub mcp_server_id: Option<String>,
    pub enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct McpServerRow {
    pub id: String,
    pub name: String,
    pub transport: String,
    pub command: Option<String>,
    pub args_json: Option<String>,
    pub env_json: Option<String>,
    pub url: Option<String>,
    pub enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct CustomChannelRow {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub auth_json: String,
    pub endpoints_json: String,
    pub stream_protocol: String,
    pub request_template_json: String,
    pub response_mapping_json: String,
    pub stream_mapping_json: String,
    pub models_json: String,
    pub enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
}
```

## 约束说明

- `MessageRow` 必须包含 `version_group_id` 和 `is_active`。
- 当前版本不保留 `starred`、`branch_id`、`prompt_versions_json` 等扩展字段。
