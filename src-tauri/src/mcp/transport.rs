use async_trait::async_trait;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, ChildStdout, Command},
};

use crate::{
    error::AppError,
    mcp::protocol::{McpRequest, McpResponse},
};

#[async_trait]
pub trait McpTransport: Send {
    async fn send_request(&mut self, request: &McpRequest) -> Result<McpResponse, AppError>;
}

pub struct StdioTransport {
    _child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl StdioTransport {
    pub async fn spawn(program: &str, args: &[String]) -> Result<Self, AppError> {
        let mut child = Command::new(program)
            .args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .map_err(|error| AppError::internal(format!("failed to spawn MCP process: {error}")))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| AppError::internal("failed to capture MCP process stdin".to_string()))?;
        let stdout = child.stdout.take().ok_or_else(|| {
            AppError::internal("failed to capture MCP process stdout".to_string())
        })?;

        Ok(Self {
            _child: child,
            stdin,
            stdout: BufReader::new(stdout),
        })
    }
}

#[async_trait]
impl McpTransport for StdioTransport {
    async fn send_request(&mut self, request: &McpRequest) -> Result<McpResponse, AppError> {
        let payload = serde_json::to_string(request).map_err(|error| {
            AppError::internal(format!("failed to serialize MCP request: {error}"))
        })?;
        self.stdin
            .write_all(format!("{payload}\n").as_bytes())
            .await
            .map_err(|error| {
                AppError::channel_unreachable(format!("failed to write MCP stdin: {error}"))
            })?;
        self.stdin.flush().await.map_err(|error| {
            AppError::channel_unreachable(format!("failed to flush MCP stdin: {error}"))
        })?;

        let mut line = String::new();
        self.stdout.read_line(&mut line).await.map_err(|error| {
            AppError::channel_unreachable(format!("failed to read MCP stdout: {error}"))
        })?;
        serde_json::from_str::<McpResponse>(line.trim()).map_err(|error| {
            AppError::ai_request_failed(format!("failed to parse MCP stdio response: {error}"))
        })
    }
}

pub struct StreamableHttpTransport {
    client: reqwest::Client,
    endpoint: String,
    session_id: Option<String>,
}

impl StreamableHttpTransport {
    pub fn new(client: reqwest::Client, endpoint: impl Into<String>) -> Self {
        Self {
            client,
            endpoint: endpoint.into(),
            session_id: None,
        }
    }

    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }
}

#[async_trait]
impl McpTransport for StreamableHttpTransport {
    async fn send_request(&mut self, request: &McpRequest) -> Result<McpResponse, AppError> {
        let mut builder = self.client.post(&self.endpoint).json(request);
        if let Some(session_id) = &self.session_id {
            builder = builder.header("mcp-session-id", session_id);
        }

        let response = builder.send().await.map_err(|error| {
            AppError::channel_unreachable(format!("failed to reach MCP HTTP endpoint: {error}"))
        })?;
        let status = response.status();
        let body = response.text().await.map_err(|error| {
            AppError::ai_request_failed(format!("failed to read MCP HTTP response: {error}"))
        })?;

        if !status.is_success() {
            return Err(AppError::ai_request_failed(format!(
                "MCP HTTP endpoint returned {status}: {body}"
            )));
        }

        serde_json::from_str::<McpResponse>(&body).map_err(|error| {
            AppError::ai_request_failed(format!("failed to parse MCP HTTP response: {error}"))
        })
    }
}
