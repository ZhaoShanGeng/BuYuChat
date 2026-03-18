# 统一错误类型

## AppError（顶层错误枚举）

```rust
// src-tauri/src/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    // --- Provider 层 ---
    #[error("Provider '{provider}' not found or not enabled")]
    ProviderNotFound { provider: String },

    #[error("API request failed [{status}]: {body}")]
    ApiError { status: u16, body: String },

    #[error("API key not found for provider '{provider}'")]
    ApiKeyNotFound { provider: String },

    #[error("Stream error: {0}")]
    StreamError(String),

    #[error("Request timeout after {secs}s")]
    Timeout { secs: u64 },

    // --- 数据库层 ---
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Record not found: {entity} id={id}")]
    NotFound { entity: &'static str, id: String },

    // --- 工具调用 ---
    #[error("Tool '{name}' not found in registry")]
    ToolNotFound { name: String },

    #[error("Tool execution failed '{name}': {reason}")]
    ToolExecFailed { name: String, reason: String },

    #[error("Tool call loop exceeded max iterations ({max})")]
    ToolLoopExceeded { max: u8 },

    // --- MCP ---
    #[error("MCP server '{server}' not connected")]
    McpNotConnected { server: String },

    #[error("MCP protocol error: {0}")]
    McpProtocol(String),

    // --- RAG ---
    #[error("Document parse failed '{name}': {reason}")]
    DocumentParseFailed { name: String, reason: String },

    #[error("Embedding failed: {0}")]
    EmbeddingFailed(String),

    #[error("Vector store error: {0}")]
    VectorStore(String),

    // --- 密钥管理 ---
    #[error("Keyring error: {0}")]
    Keyring(#[from] keyring::Error),

    // --- 序列化 ---
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    // --- IO ---
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    // --- 自定义渠道 ---
    #[error("Template render error: {0}")]
    TemplateRender(String),

    #[error("JSONPath extraction failed '{path}': {reason}")]
    JsonPathFailed { path: String, reason: String },

    // --- 通用 ---
    #[error("{0}")]
    Other(String),
}

/// 给 Tauri command 用：将 AppError 序列化为 String 返回前端
impl serde::Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
```

## 使用原则

- **所有 Tauri command** 的返回类型为 `Result<T, AppError>`（Tauri 会自动序列化错误给前端）
- **服务层方法** 统一返回 `crate::error::Result<T>`
- **不在底层 swallow 错误**：底层遇到错误用 `?` 上传，在 command 层统一 log 后返回给前端
- **前端错误展示**：前端收到 `Err(string)` 时，直接 toast 展示该字符串
