use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
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

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Database migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("Record not found: {entity} id={id}")]
    NotFound { entity: &'static str, id: String },

    #[error("Tool '{name}' not found in registry")]
    ToolNotFound { name: String },

    #[error("Tool execution failed '{name}': {reason}")]
    ToolExecFailed { name: String, reason: String },

    #[error("Tool call loop exceeded max iterations ({max})")]
    ToolLoopExceeded { max: u8 },

    #[error("MCP server '{server}' not connected")]
    McpNotConnected { server: String },

    #[error("MCP protocol error: {0}")]
    McpProtocol(String),

    #[error("Document parse failed '{name}': {reason}")]
    DocumentParseFailed { name: String, reason: String },

    #[error("Embedding failed: {0}")]
    EmbeddingFailed(String),

    #[error("Vector store error: {0}")]
    VectorStore(String),

    #[error("Keyring error: {0}")]
    Keyring(#[from] keyring::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Tauri error: {0}")]
    Tauri(#[from] tauri::Error),

    #[error("Template render error: {0}")]
    TemplateRender(String),

    #[error("JSONPath extraction failed '{path}': {reason}")]
    JsonPathFailed { path: String, reason: String },

    #[error("{0}")]
    Other(String),
}

impl serde::Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
