# 工具调用（核心）与 MCP（可选增强）

> 路径：`src-tauri/src/services/tool/`

当前版本的实现顺序必须固定：

1. 先完成 `ToolRegistry + ToolService + 内置工具 + Tool Loop`
2. 验证工具调用主链路可用
3. 再补 `MCP` 接入

不要一开始就实现完整 MCP 管理器。

## 目录结构

```
tool/
├── mod.rs          # ToolService + ToolRegistry
├── builtin/
│   ├── web_search.rs
│   └── calculator.rs
└── mcp/            # 可选增强
    ├── client.rs   # McpClient
    ├── transport.rs
    └── registry.rs
```

---

## ToolRegistry

```rust
// src-tauri/src/services/tool/mod.rs
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct ToolRegistry {
    tools: RwLock<HashMap<String, RegisteredTool>>,
}

pub struct RegisteredTool {
    pub def: ToolDef,
    pub source: ToolSource,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub enum ToolSource {
    Builtin,
    Mcp { server_id: String },
    User,
}

impl ToolRegistry {
    /// 注册一个工具（已存在时覆盖）
    pub async fn register(&self, tool: RegisteredTool);

    /// 取消注册（MCP Server 断开时调用；非 MCP 版本可暂不实现）
    pub async fn unregister_by_server(&self, server_id: &str);

    /// 获取所有已启用工具的 ToolDef 列表
    pub async fn list_enabled(&self) -> Vec<ToolDef>;

    /// 获取某个助手/对话可用的工具子集
    pub async fn get_for_assistant(&self, tool_ids: &[String]) -> Vec<ToolDef>;

    /// 按名称获取单个工具
    pub async fn get(&self, name: &str) -> Option<RegisteredTool>;
}
```

---

## ToolService（入口）

```rust
pub struct ToolService {
    registry: Arc<ToolRegistry>,
    mcp_manager: Arc<McpManager>,
    db: SqlitePool,
}

impl ToolService {
    /// 应用启动时只注册内置工具
    pub async fn init_builtin(&self) -> Result<()>;

    /// 执行单个工具调用
    pub async fn execute(&self, call: &ToolCall) -> Result<String>;

    /// 获取对话应使用的工具列表
    /// 优先级：助手绑定工具 > 应用默认启用工具
    pub async fn get_tools_for_conv(&self, conv: &ConversationRow) -> Result<Vec<ToolDef>>;

    /// 可选增强：从数据库同步 MCP 工具状态
    pub async fn sync_mcp_from_db(&self) -> Result<()>;
}
```

---

## 内置工具

### web_search

```rust
// src-tauri/src/services/tool/builtin/web_search.rs
pub struct WebSearchTool;

// ToolDef：
// name: "web_search"
// description: "Search the web for up-to-date information"
// parameters:
//   query: string, required
//   engine: enum ["tavily","searxng"], default "tavily"
//   num_results: integer, default 5

impl WebSearchTool {
    pub async fn execute(&self, query: &str, engine: &str, num_results: u32) -> String;
}
```

实现规则：

- `tavily` 作为默认引擎
- `searxng` 作为本地可选引擎
- 返回值必须是可直接注入 LLM 的紧凑 JSON 字符串
- 搜索失败时返回文本结果，不直接抛错中断工具循环

### calculator

```rust
// src-tauri/src/services/tool/builtin/calculator.rs
pub struct CalculatorTool;

// ToolDef：
// name: "calculator"
// description: "Evaluate mathematical expressions"
// parameters:
//   expression: string, required（如 "2^10 + sqrt(144)"）

impl CalculatorTool {
    pub fn execute(&self, expression: &str) -> String;
}
```

实现规则：

- 只支持纯数学表达式
- 返回值始终是字符串
- 失败时返回 `"计算错误：{原因}"`

---

## 工具循环和 ChatService 的边界

`ChatService` 负责：

- 准备 `ChatRequest`
- 决定是否启用工具
- 收发流式事件

`ToolService` 负责：

- 提供 `ToolDef`
- 执行具体工具
- 把工具结果转换成 `role=tool` 的标准消息内容

不要把对话编排逻辑放进工具模块。

---

## MCP 客户端（可选增强）

只有在内置工具已经稳定后，才实现这一节。

### McpManager

```rust
// src-tauri/src/services/tool/mcp/client.rs
pub struct McpManager {
    clients: RwLock<HashMap<String, Arc<McpClient>>>,  // server_id -> client
    registry: Arc<ToolRegistry>,
    db: SqlitePool,
}

impl McpManager {
    /// 启动所有 enabled=true 的 MCP Server
    pub async fn start_all(&self) -> Result<()>;

    /// 启动单个 MCP Server 并注册其工具到 ToolRegistry
    pub async fn start_server(&self, server_id: &str) -> Result<()>;

    /// 停止单个 MCP Server（发送 shutdown 通知，取消注册其工具）
    pub async fn stop_server(&self, server_id: &str) -> Result<()>;

    /// 调用 MCP 工具
    pub async fn call_tool(&self, server_id: &str, call: &ToolCall) -> Result<String>;

    /// 获取 Server 当前连接状态
    pub async fn server_status(&self, server_id: &str) -> McpStatus;
}

#[derive(Debug, Serialize)]
pub enum McpStatus {
    Connected,
    Disconnected,
    Error { message: String },
}
```

### McpClient（单个 Server）

```rust
pub struct McpClient {
    transport: Box<dyn McpTransport>,
    tools: Vec<ToolDef>,    // 从 Server 发现的工具列表
}

impl McpClient {
    /// 建立连接并发送 initialize 握手
    pub async fn connect(config: &McpServerRow) -> Result<Self>;

    /// 列出 Server 提供的工具（发送 tools/list 请求）
    pub async fn list_tools(&self) -> Result<Vec<ToolDef>>;

    /// 调用工具（发送 tools/call 请求）
    /// 返回工具执行结果（文本内容）
    pub async fn call_tool(&self, name: &str, arguments: &serde_json::Value) -> Result<String>;

    /// 保活（每 30 秒发送 ping）
    pub async fn start_keepalive(self: Arc<Self>);
}
```

### McpTransport Trait

```rust
#[async_trait]
pub trait McpTransport: Send + Sync {
    /// 发送 JSON-RPC 请求，等待响应
    async fn request(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value>;

    /// 发送通知（无需等待响应，如 shutdown）
    async fn notify(&self, method: &str, params: serde_json::Value) -> Result<()>;
}

pub struct StdioTransport {
    stdin: tokio::io::BufWriter<tokio::process::ChildStdin>,
    stdout: tokio::io::BufReader<tokio::process::ChildStdout>,
    pending: Mutex<HashMap<u64, oneshot::Sender<serde_json::Value>>>,
    // 后台 reader 任务 dispatch 响应到对应 oneshot channel
}

pub struct SseTransport {
    client: reqwest::Client,
    base_url: String,
    // POST endpoint: {base_url}/message
    // SSE stream: GET {base_url}/sse
}
```

**JSON-RPC 帧格式（MCP 标准）：**
```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"web_search","arguments":{"query":"rust"}}}
{"jsonrpc":"2.0","id":1,"result":{"content":[{"type":"text","text":"..."}]}}
```

### MCP Server 重连机制

```
连接断开 → 等待 retry_delay（初始 1s）→ 重试连接
retry_delay 指数退避，上限 60s。
连续失败 3 次后，标记 server 状态为 McpStatus::Error，停止重试，等待用户手动重启。
```

---

## 本轮交付验收标准

- AI 可以通过文档先实现 `web_search` 和 `calculator`
- Provider 能正确接收 `ToolDef`
- 工具调用时，前端能收到 `tool_call` 事件
- 工具结果能够回注到下一轮模型请求
- 不实现 MCP 时，核心工具链路也能完整运行
