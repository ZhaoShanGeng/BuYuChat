# Provider 抽象层

> 路径：`src-tauri/src/providers/`

## 目录结构

```
providers/
├── mod.rs          # ProviderRegistry + LlmProvider trait 定义
├── openai.rs       # OpenAI Adapter（基准，其他 compatible 继承）
├── claude.rs       # Anthropic Claude Adapter
├── gemini.rs       # Google Gemini Adapter
├── ollama.rs       # Ollama Adapter（本地）
├── deepseek.rs     # DeepSeek（复用 OpenAI-compatible）
├── custom.rs       # 自定义渠道 Adapter
└── openai_compat.rs # OpenAI-compatible 基类（DeepSeek/Ollama/GLM 复用）
```

---

## LlmProvider Trait

```rust
// src-tauri/src/providers/mod.rs
use async_trait::async_trait;
use tokio::sync::mpsc;
use crate::types::*;
use crate::error::Result;

#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Provider 标识名（小写，与数据库 provider 字段一致）
    /// 例："openai" | "claude" | "gemini" | "ollama" | "deepseek" | "custom:{id}"
    fn name(&self) -> &str;

    /// 返回该 Provider 支持的模型列表
    /// 对于 Ollama：通过 GET /api/tags 动态拉取，失败返回空列表而不是 Err
    /// 对于 OpenAI/Claude/Gemini：返回硬编码的已知模型列表
    async fn list_models(&self) -> Result<Vec<ModelInfo>>;

    /// 非流式对话（用于：工具调用第二轮、自动命名、单次辅助请求）
    /// - 必须等待完整响应后返回
    /// - 超时：30 秒
    async fn chat(&self, req: &ChatRequest) -> Result<ChatResponse>;

    /// 流式对话（主对话路径）
    /// - 通过 tx 发送 StreamEvent::Delta 逐 token 推送
    /// - 流结束时发送 StreamEvent::Done
    /// - 出错时发送 StreamEvent::Error，函数本身也返回 Err
    /// - tx 发送失败（接收方已关闭）时静默退出，不返回错误
    /// - 超时：首 token 60 秒，后续每个 chunk 10 秒
    async fn chat_stream(
        &self,
        req: &ChatRequest,
        tx: mpsc::Sender<StreamEvent>,
    ) -> Result<()>;

    /// 该 Provider 是否支持 Function Calling
    /// 默认 false；OpenAI/Claude/Gemini 返回 true；Ollama 视模型而定
    fn supports_function_calling(&self) -> bool {
        false
    }

    /// 将 ToolDef 列表格式化为该 Provider 的请求格式
    /// 只有 supports_function_calling() == true 时才会被调用
    fn format_tools(&self, tools: &[ToolDef]) -> serde_json::Value;

    /// 检测连通性（可选，用于设置页"测试连接"按钮）
    /// 发送最小化请求验证 API Key 有效性，成功返回 Ok(())
    async fn health_check(&self) -> Result<()>;
}
```

---

## ProviderRegistry

```rust
// src-tauri/src/providers/mod.rs
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ProviderRegistry {
    providers: RwLock<HashMap<String, Arc<dyn LlmProvider>>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self { providers: RwLock::new(HashMap::new()) }
    }

    /// 注册或替换 Provider（应用启动时，以及用户修改配置后调用）
    pub async fn register(&self, provider: Arc<dyn LlmProvider>) {
        let name = provider.name().to_string();
        self.providers.write().await.insert(name, provider);
    }

    /// 按名称取 Provider；找不到返回 AppError::ProviderNotFound
    pub async fn get(&self, name: &str) -> Result<Arc<dyn LlmProvider>> {
        self.providers.read().await
            .get(name)
            .cloned()
            .ok_or_else(|| AppError::ProviderNotFound { provider: name.to_string() })
    }

    /// 列出所有已注册 Provider 的名称
    pub async fn list_names(&self) -> Vec<String> {
        self.providers.read().await.keys().cloned().collect()
    }
}
```

---

## 各 Adapter 实现要点

### OpenAI Adapter（基准）

```rust
// src-tauri/src/providers/openai.rs
pub struct OpenAiProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,    // 默认 "https://api.openai.com/v1"
}
```

**chat() 实现步骤：**
1. 将 `ChatRequest.messages` 按 OpenAI 格式序列化（role/content/tool_calls/tool_call_id）
2. 若有 `system_prompt`，插入 `{"role":"system","content":"..."}` 到 messages[0]
3. POST `/chat/completions`，`stream: false`
4. 解析 `choices[0].message.content` 和 `choices[0].message.tool_calls`
5. 解析 `usage.prompt_tokens / completion_tokens`
6. 返回 `ChatResponse`

**chat_stream() 实现步骤：**
1. 同上序列化，`stream: true`
2. `reqwest` 获取字节流，通过 `eventsource-stream` 解析 SSE 事件
3. 每个 `data: {...}` 事件：提取 `choices[0].delta.content` → 发 `StreamEvent::Delta`
4. 若 delta 中有 `tool_calls`：累积所有 delta 的工具调用片段直到 `finish_reason == "tool_calls"` → 发 `StreamEvent::ToolCall`（完整的 ToolCall）
5. 收到 `data: [DONE]` → 发 `StreamEvent::Done`

**format_tools()：**
```json
{
  "tools": [
    {
      "type": "function",
      "function": {
        "name": "web_search",
        "description": "...",
        "parameters": { /* JSON Schema */ }
      }
    }
  ],
  "tool_choice": "auto"
}
```

---

### OpenAI-Compatible 基类

```rust
// src-tauri/src/providers/openai_compat.rs
pub struct OpenAiCompatProvider {
    inner: OpenAiProvider,  // 复用 OpenAI 全部逻辑
    name: String,           // 如 "deepseek"
}
```

DeepSeek、GLM、Moonshot 等只需修改 `base_url`，其余全部复用 `OpenAiProvider`。

---

### Claude Adapter

```rust
pub struct ClaudeProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,  // 默认 "https://api.anthropic.com/v1"
}
```

**关键差异：**

| 方面 | 处理方式 |
|------|---------|
| system 字段 | 独立字段 `"system": "..."` 而非 messages[0] |
| 请求端点 | POST `/messages`（不是 `/chat/completions`） |
| 请求头 | 需额外加 `anthropic-version: 2023-06-01` |
| 流式事件 | SSE 事件类型为 `content_block_delta`；delta 字段为 `delta.text` |
| 工具格式 | `"tools": [{"name":..., "description":..., "input_schema": {...}}]` |
| 工具调用响应 | 返回 `content` 数组，其中有 `{"type":"tool_use","id":...,"name":...,"input":{...}}` |
| 工具结果格式 | role=`user`，content 为 `[{"type":"tool_result","tool_use_id":...,"content":"..."}]` |
| stop_reason | `"tool_use"`（对应 OpenAI 的 `"tool_calls"`） |

---

### Gemini Adapter

```rust
pub struct GeminiProvider {
    client: reqwest::Client,
    api_key: String,    // 通过 ?key=xxx 传在 query param
    base_url: String,   // 默认 "https://generativelanguage.googleapis.com/v1beta"
}
```

**关键差异：**

| 方面 | 处理方式 |
|------|---------|
| 消息格式 | `contents: [{role: "user"/"model", parts: [{text: "..."}]}]` |
| system_prompt | `systemInstruction: {parts: [{text: "..."}]}` 字段 |
| 流式端点 | `POST /models/{model}:streamGenerateContent?alt=sse` |
| 工具格式 | `tools: [{functionDeclarations: [{name, description, parameters}]}]` |
| 工具调用 | `parts` 中出现 `functionCall: {name, args}` |
| 工具结果 | role=`user`，parts 含 `functionResponse: {name, response: {...}}` |

---

### Ollama Adapter

```rust
pub struct OllamaProvider {
    client: reqwest::Client,
    base_url: String,  // 默认 "http://localhost:11434"
}
```

**list_models() 实现：**
- GET `{base_url}/api/tags`
- 解析 `models[].name` 字段，返回 `ModelInfo` 列表
- 失败时日志警告并返回空列表（不影响其他 Provider）

**chat() / chat_stream()：**
- 端点：POST `{base_url}/api/chat`（兼容 OpenAI 消息格式）
- 流式：NDJSON，每行一个 JSON 对象，`message.content` 为 delta，`done: true` 表示结束
- 无需 API Key（请求头无 Authorization）

**supports_function_calling()：**
- 根据模型名判断：`model.contains("llama3")` 或 `model.contains("qwen2.5")` 等返回 true
- 保守策略：默认返回 false，用户可通过设置手动启用

---

## 应用启动时 Provider 初始化

```rust
// src-tauri/src/app.rs（在 setup 函数中调用）
pub async fn init_providers(
    registry: &ProviderRegistry,
    db: &SqlitePool,
    keyring: &KeyringService,
) -> Result<()> {
    let configs = db::provider_config::list_enabled(db).await?;
    for config in configs {
        let api_key = match &config.api_key_id {
            Some(key_id) => Some(keyring.get(key_id)?),
            None => None,
        };
        let base_url = config.base_url.clone();
        let provider: Arc<dyn LlmProvider> = match config.provider.as_str() {
            "openai"    => Arc::new(OpenAiProvider::new(api_key.unwrap_or_default(), base_url)),
            "claude"    => Arc::new(ClaudeProvider::new(api_key.unwrap_or_default(), base_url)),
            "gemini"    => Arc::new(GeminiProvider::new(api_key.unwrap_or_default(), base_url)),
            "ollama"    => Arc::new(OllamaProvider::new(base_url.unwrap_or_default())),
            "deepseek"  => Arc::new(OpenAiCompatProvider::new("deepseek", api_key.unwrap_or_default(), base_url)),
            _           => continue,
        };
        registry.register(provider).await;
    }
    // 加载自定义渠道
    let channels = db::custom_channel::list_enabled(db).await?;
    for ch in channels {
        let adapter = Arc::new(CustomChannelAdapter::from_config(ch, keyring)?);
        registry.register(adapter).await;
    }
    Ok(())
}
```
