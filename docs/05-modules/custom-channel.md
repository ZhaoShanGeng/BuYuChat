# 自定义渠道系统

> 路径：`src-tauri/src/providers/custom.rs`

用户可以任意接入 HTTP LLM API，通过**渠道配置 JSON + Handlebars 模板 + JSONPath 响应映射**实现，无需写代码。

---

## CustomChannelAdapter

```rust
// src-tauri/src/providers/custom.rs
pub struct CustomChannelAdapter {
    config: CustomChannelConfig,   // 从数据库反序列化
    client: reqwest::Client,
    api_key: Option<String>,       // 从 keyring 取出的明文（不落盘）
    // name 格式："custom:{channel_id}"，如 "custom:abc123"
}

impl CustomChannelAdapter {
    /// 从数据库行和 keyring 构造 Adapter
    /// 若 auth_json 中 key_ref 不为空，从 keyring 读取 api_key
    pub fn from_config(row: CustomChannelRow, keyring: &KeyringService) -> Result<Self>;
}
```

`CustomChannelAdapter` 实现 `LlmProvider` trait，以下说明各方法：

---

## LlmProvider 实现

### name()

```rust
fn name(&self) -> &str {
    // 返回 "custom:{id}"，id 为数据库的 custom_channels.id
    &self.name_str  // 预计算存在结构体字段
}
```

### list_models()

```rust
async fn list_models(&self) -> Result<Vec<ModelInfo>> {
    // 直接从 config.models_json 反序列化，不发网络请求
    // 用户在设置页手动填写模型列表
    // 若为空，返回 [ModelInfo { id: "default", name: "Default", ... }]
}
```

### chat()

**步骤：**
1. 用 Handlebars 渲染 `request_template_json`，注入以下变量：
   ```
   {{model}}       → ChatRequest.model
   {{messages}}    → JSON 序列化的 messages 数组（标准格式）
   {{stream}}      → false
   {{temperature}} → params.temperature（若为 None 则不渲染该字段）
   {{max_tokens}}  → params.max_tokens
   {{API_KEY}}     → keyring 取得的 api_key
   {{custom.*}}    → params.custom 中的字段
   ```
2. POST 到 `{base_url}{endpoints.chat}`，带上认证头（见下文认证）
3. 解析响应 JSON，用 JSONPath 提取字段：
   - `content = jsonpath(resp, config.response_mapping.content)`
   - `finish_reason = jsonpath(resp, config.response_mapping.finish_reason)`
4. 返回 `ChatResponse`

### chat_stream()

**步骤：**
1. 同 chat()，但 `{{stream}}` 渲染为 `true`，POST 到 `endpoints.stream`
2. 根据 `stream_protocol` 处理响应：

   **SSE（`data: {...}\n\n`）：**
   ```
   每行 data: 字段 → 解析 JSON → 用 stream_mapping.delta 提取 delta 文本
   data: [DONE] → 发送 StreamEvent::Done
   ```

   **NDJSON（`{...}\n`）：**
   ```
   每行 → 解析 JSON → 用 stream_mapping.delta 提取 delta 文本
   stream_mapping.done_signal 字段值为 true → 发送 StreamEvent::Done
   ```
3. 每次获得 delta 文本 → `tx.send(StreamEvent::Delta { text })`

### supports_function_calling()

```rust
fn supports_function_calling(&self) -> bool {
    // 默认 false；用户可在渠道配置中通过 extra 字段设置 "supports_tools": true
    self.config.extra.get("supports_tools")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}
```

### format_tools()

```rust
fn format_tools(&self, tools: &[ToolDef]) -> serde_json::Value {
    // 使用 OpenAI 格式（大多数兼容 API 支持）
    // 若渠道需要其他格式，由用户在 request_template 中手动处理
    serde_json::json!({
        "tools": tools.iter().map(|t| serde_json::json!({
            "type": "function",
            "function": { "name": t.name, "description": t.description, "parameters": t.parameters }
        })).collect::<Vec<_>>()
    })
}
```

---

## 认证处理

```rust
// 根据 auth_json.type 设置 HTTP 请求头
match auth.auth_type.as_str() {
    "bearer" => {
        headers.insert("Authorization", format!("Bearer {}", api_key));
    }
    "api_key" => {
        // 在 query param 中传入：?key={api_key}
        // 或自定义 header：headers.insert(&auth.header_name, api_key)
    }
    "none" => {}  // 无需认证（内网部署）
    _ => return Err(AppError::Other(format!("Unknown auth type: {}", auth.auth_type)))
}
```

---

## 渠道配置结构体（Rust 侧反序列化）

```rust
#[derive(Debug, Deserialize)]
pub struct CustomChannelConfig {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub auth: AuthConfig,
    pub endpoints: Endpoints,
    pub stream_protocol: StreamProtocol,
    pub request_template: serde_json::Value,  // Handlebars 模板 JSON
    pub response_mapping: ResponseMapping,
    pub stream_mapping: StreamMapping,
    pub models: Vec<SimpleModel>,
    pub extra: serde_json::Value,  // 扩展字段（如 "supports_tools": true）
}

#[derive(Debug, Deserialize)]
pub struct AuthConfig {
    pub auth_type: String,     // "bearer" | "api_key" | "none"
    pub key_ref: Option<String>,   // keyring 中的 key 标识
    pub header_name: Option<String>,  // api_key 模式的 header 名
}

#[derive(Debug, Deserialize)]
pub struct Endpoints {
    pub chat: String,
    pub stream: String,  // 可与 chat 相同
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StreamProtocol {
    Sse,
    Ndjson,
}

#[derive(Debug, Deserialize)]
pub struct ResponseMapping {
    pub content: String,        // JSONPath，如 "$.choices[0].message.content"
    pub finish_reason: String,  // JSONPath，如 "$.choices[0].finish_reason"
}

#[derive(Debug, Deserialize)]
pub struct StreamMapping {
    pub delta: String,          // JSONPath，如 "$.choices[0].delta.content"
    pub done_sentinel: Option<String>,  // SSE done 信号（如 "[DONE]"）；NDJSON 用 done_flag
    pub done_flag: Option<String>,      // NDJSON done 标志 JSONPath（如 "$.done"）
}

#[derive(Debug, Deserialize, serde::Serialize)]
pub struct SimpleModel {
    pub id: String,
    pub name: String,
}
```

---

## 内置预设（openai-compatible）

这是最常用的预设，适用于 DeepSeek、Moonshot、Qwen、本地 vllm 等：

```json
{
  "name": "OpenAI-Compatible",
  "base_url": "https://api.example.com",
  "auth": { "auth_type": "bearer", "key_ref": "my_api_key" },
  "endpoints": { "chat": "/v1/chat/completions", "stream": "/v1/chat/completions" },
  "stream_protocol": "sse",
  "request_template": {
    "model": "{{model}}",
    "messages": "{{messages}}",
    "stream": "{{stream}}",
    "temperature": "{{temperature}}",
    "max_tokens": "{{max_tokens}}"
  },
  "response_mapping": {
    "content": "$.choices[0].message.content",
    "finish_reason": "$.choices[0].finish_reason"
  },
  "stream_mapping": {
    "delta": "$.choices[0].delta.content",
    "done_sentinel": "[DONE]"
  },
  "models": [{ "id": "your-model", "name": "Your Model" }]
}
```

## CRUD 接口（Tauri 命令对应）

```rust
// db::custom_channel 模块
pub async fn list_all(db: &SqlitePool) -> Result<Vec<CustomChannelRow>>;
pub async fn list_enabled(db: &SqlitePool) -> Result<Vec<CustomChannelRow>>;
pub async fn get(db: &SqlitePool, id: &str) -> Result<CustomChannelRow>;
pub async fn create(db: &SqlitePool, row: CustomChannelRow) -> Result<CustomChannelRow>;
pub async fn update(db: &SqlitePool, id: &str, row: CustomChannelRow) -> Result<()>;
pub async fn delete(db: &SqlitePool, id: &str) -> Result<()>;
// 删除后：调用 ProviderRegistry.deregister("custom:{id}")
```
