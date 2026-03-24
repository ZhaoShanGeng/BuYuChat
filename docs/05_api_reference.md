# 步语 BuYu — API 参考手册

**版本：** 0.2
**阶段：** MVP（P0）

---

## 1. 通用约定

### 1.1 传输层

BuYu 使用 Tauri IPC（非 HTTP）。本文档以 REST 风格描述语义，实际调用方式：

```typescript
import { invoke } from "@tauri-apps/api/core";

// REST: GET /channels → Tauri: invoke("list_channels")
// REST: POST /channels → Tauri: invoke("create_channel", { input })
// REST: PATCH /channels/{id} → Tauri: invoke("update_channel", { id, input })
// REST: DELETE /channels/{id} → Tauri: invoke("delete_channel", { id })
```

**命名映射规则：** `HTTP_METHOD /resource/{id}/action` → `snake_case` 命令名

### 1.2 ID 格式

所有实体使用 **UUID v7**（时间有序），字符串格式：`019587ab-0000-7abc-8def-000000000001`

### 1.3 时间戳

所有 `created_at` / `updated_at` 字段为 **Unix 毫秒时间戳**（integer）。

### 1.4 命名风格

| 层级 | 风格 | 示例 |
|------|------|------|
| Rust 结构体 / 字段 | snake_case | `channel_model_id` |
| Tauri command 名 | snake_case | `create_channel` |
| TypeScript 类型 / 字段 | camelCase | `channelModelId` |
| API 文档 (OpenAPI) | snake_case | `channel_model_id` |
| Tauri serde 序列化 | snake_case → 前端 transport 层转 camelCase | — |

**重要：** MVP 当前由前端 transport 层统一完成 `snake_case → camelCase` 转换；OpenAPI 文档继续使用 snake_case 描述字段语义。

### 1.5 成功响应

| 操作类型 | 状态码 | 响应体 |
|----------|--------|--------|
| 创建 | 201 | 完整资源对象 |
| 读取 | 200 | 资源对象或数组 |
| 更新 | 200 | 更新后的完整资源对象 |
| 删除 | 204 | 无响应体 |
| 动作（send/reroll/cancel） | 200 | 操作结果对象 |

### 1.6 错误响应格式

所有错误统一格式：

```json
{
  "error_code": "NO_CHANNEL",
  "message": "conversation has no channel configured"
}
```

- `error_code`：机器可读，前端用于 i18n 查表翻译为中文
- `message`：英文调试信息，仅用于开发排查

### 1.7 错误码总表

| 错误码 | HTTP 语义 | 触发场景 |
|--------|-----------|---------|
| `VALIDATION_ERROR` | 400 | 入参校验失败（附 field 信息） |
| `INVALID_URL` | 400 | base_url 格式不合法 |
| `NAME_EMPTY` | 400 | name 为空字符串 |
| `CONTENT_EMPTY` | 400 | 消息 content 为空 |
| `NOT_FOUND` | 404 | 资源不存在 |
| `MODEL_ID_CONFLICT` | 409 | 同渠道下 model_id 已存在 |
| `NO_AGENT` | 422 | 会话未绑定 Agent |
| `AGENT_DISABLED` | 422 | Agent 已禁用 |
| `NO_CHANNEL` | 422 | 会话未绑定渠道 |
| `CHANNEL_DISABLED` | 422 | 渠道已禁用 |
| `NO_MODEL` | 422 | 会话未绑定模型 |
| `NOT_LAST_USER_NODE` | 422 | user node 不是末尾楼层（不允许 reroll） |
| `VERSION_NOT_IN_NODE` | 400 | version_id 不属于指定 node |
| `CHANNEL_UNREACHABLE` | 502 | 渠道连通性测试失败 |
| `AI_REQUEST_FAILED` | 502 | AI 服务商返回错误 |
| `INTERNAL_ERROR` | 500 | 未预期的内部错误 |

---

## 2. 渠道资源 (Channels)

### POST /channels — 创建渠道

**Tauri command:** `create_channel`

**请求体：**
```json
{
  "name": "My OpenAI",
  "base_url": "https://api.openai.com",
  "api_key": "sk-xxxx",
  "channel_type": "openai_compatible",
  "auth_type": null,
  "models_endpoint": null,
  "chat_endpoint": null,
  "stream_endpoint": null
}
```

必填：`name`（非空）、`base_url`（http/https 开头）。其余选填。

**成功响应 (201)：**
```json
{
  "id": "019587ab-0000-7abc-8def-000000000001",
  "name": "My OpenAI",
  "channel_type": "openai_compatible",
  "base_url": "https://api.openai.com",
  "api_key": "sk-xxxx",
  "auth_type": null,
  "models_endpoint": null,
  "chat_endpoint": null,
  "stream_endpoint": null,
  "enabled": true,
  "created_at": 1735000000000,
  "updated_at": 1735000000000
}
```

**错误响应：**
```json
// 400
{ "error_code": "INVALID_URL", "message": "base_url must start with http:// or https://" }
```

---

### GET /channels — 渠道列表

**Tauri command:** `list_channels`

**参数：** `include_disabled: bool`（默认 true）

**成功响应 (200)：** `Channel[]`，按 `created_at` 降序

---

### GET /channels/{id} — 渠道详情

**Tauri command:** `get_channel`

---

### PATCH /channels/{id} — 更新渠道

**Tauri command:** `update_channel`

只发送需要修改的字段。

```json
// 请求：仅修改名称和启用状态
{ "name": "OpenAI Pro", "enabled": false }

// 响应 200：返回完整的更新后对象
```

---

### DELETE /channels/{id} — 删除渠道

**Tauri command:** `delete_channel`

**响应：** 204（无响应体）

**副作用：**
1. 级联删除 `api_channel_models`
2. `conversations.channel_id` → NULL
3. `conversations.channel_model_id` → NULL

---

### POST /channels/{id}/test — 测试渠道连通性

**Tauri command:** `test_channel`

```json
// 成功响应
{ "success": true, "message": "channel is reachable" }

// 失败响应 502
{ "error_code": "CHANNEL_UNREACHABLE", "message": "failed to reach channel: connection timeout" }
```

---

## 3. 模型资源 (Models)

### POST /channels/{channelId}/models — 添加模型

**Tauri command:** `create_model`

```json
// 请求
{ "model_id": "gpt-4o", "display_name": "GPT-4o" }

// 成功响应 201
{
  "id": "019587ab-0001-7abc-8def-000000000002",
  "channel_id": "019587ab-0000-7abc-8def-000000000001",
  "model_id": "gpt-4o",
  "display_name": "GPT-4o",
  "context_window": null,
  "max_output_tokens": null
}

// 冲突响应 409
{ "error_code": "MODEL_ID_CONFLICT", "message": "model_id 'gpt-4o' already exists in this channel" }
```

---

### GET /channels/{channelId}/models — 模型列表

**Tauri command:** `list_models`

---

### PATCH /channels/{channelId}/models/{id} — 更新模型

**Tauri command:** `update_model`

---

### DELETE /channels/{channelId}/models/{id} — 删除模型

**Tauri command:** `delete_model`

**副作用：** `conversations.channel_model_id` → NULL

---

### POST /channels/{channelId}/models/fetch — 从远程拉取模型列表

**Tauri command:** `fetch_remote_models`

不写库，仅返回供用户勾选。

```json
// 成功响应 200
[
  { "model_id": "gpt-4o", "display_name": "GPT-4o", "context_window": 128000 },
  { "model_id": "gpt-4o-mini", "display_name": null, "context_window": null }
]
```

---

## 4. Agent 资源 (Agents)

### POST /agents — 创建 Agent

**Tauri command:** `create_agent`

```json
// 请求
{ "name": "助手", "system_prompt": "你是一个有帮助的助手" }

// 成功响应 201
{
  "id": "019587ab-0003-...",
  "name": "助手",
  "system_prompt": "你是一个有帮助的助手",
  "avatar_uri": null,
  "enabled": true,
  "created_at": 1735000000000,
  "updated_at": 1735000000000
}
```

---

### GET /agents — Agent 列表

**Tauri command:** `list_agents`

**参数：** `include_disabled: bool`（默认 true）

---

### PATCH /agents/{id} — 更新 Agent

**Tauri command:** `update_agent`

system_prompt 修改后对所有绑定会话的**下一条消息**立即生效（实时读取，不做快照）。

---

### DELETE /agents/{id} — 删除 Agent

**Tauri command:** `delete_agent`

**副作用：** `conversations.agent_id` → NULL

---

## 5. 会话资源 (Conversations)

### POST /conversations — 创建会话

**Tauri command:** `create_conversation`

```json
// 请求（可选）
{ "title": "关于 Rust 的讨论", "agent_id": "...", "channel_id": "...", "channel_model_id": "..." }

// 最小请求（全部使用默认值）
{}

// 成功响应 201
{
  "id": "019587ab-0004-...",
  "title": "新会话",
  "agent_id": null,
  "channel_id": null,
  "channel_model_id": null,
  "archived": false,
  "pinned": false,
  "created_at": 1735000000000,
  "updated_at": 1735000000000
}
```

---

### GET /conversations — 会话列表

**Tauri command:** `list_conversations`

**参数：** `archived: bool`（默认 false）

**排序：** 置顶优先（`pinned DESC`），再按 `updated_at DESC`

---

### PATCH /conversations/{id} — 更新会话

**Tauri command:** `update_conversation`

用于：重命名、绑定 Agent/渠道/模型、归档、置顶。

```json
// 绑定配置
{ "agent_id": "...", "channel_id": "...", "channel_model_id": "..." }

// 归档
{ "archived": true }

// 置顶
{ "pinned": true }
```

---

### DELETE /conversations/{id} — 删除会话

**Tauri command:** `delete_conversation`

级联删除所有 `message_nodes` 和 `message_versions`。

---

## 6. 消息资源 (Messages)

### GET /conversations/{id}/messages — 消息列表

**Tauri command:** `list_messages`

返回所有 node 及其**全部 version 的元数据**。**只有 active version 包含 content**，非 active version 的 `content` 字段为 `null`。切换版本时通过 `GET /versions/{versionId}/content` 按需加载。

```json
// 成功响应 200
[
  {
    "id": "node-001",
    "conversation_id": "conv-001",
    "role": "user",
    "order_key": "0000001735000000000-0-a3f9",
    "active_version_id": "ver-001",
    "versions": [
      {
        "id": "ver-001",
        "node_id": "node-001",
        "content": "用 Rust 写一个快速排序",
        "status": "committed",
        "model_name": null,
        "prompt_tokens": null,
        "completion_tokens": null,
        "finish_reason": null,
        "created_at": 1735000001000
      }
    ],
    "created_at": 1735000001000
  },
  {
    "id": "node-002",
    "role": "assistant",
    "order_key": "0000001735000000000-1-a3f9",
    "active_version_id": "ver-002",
    "versions": [
      {
        "id": "ver-002",
        "node_id": "node-002",
        "content": "当然，这是 Rust 快速排序的实现...",
        "status": "committed",
        "model_name": "gpt-4o",
        "prompt_tokens": 23,
        "completion_tokens": 312,
        "finish_reason": "stop",
        "created_at": 1735000002000
      },
      {
        "id": "ver-003",
        "node_id": "node-002",
        "content": null,
        "status": "committed",
        "model_name": "gpt-4o",
        "prompt_tokens": 25,
        "completion_tokens": 280,
        "finish_reason": "stop",
        "created_at": 1735000003000
      }
    ],
    "created_at": 1735000001000
  }
]
```

> 注意 `ver-003`（非 active）的 `content` 为 `null`，需要时再调用 `/versions/ver-003/content` 加载。

---

### GET /versions/{versionId}/content — 按需加载版本内容

**Tauri command:** `get_version_content`

用于版本切换时加载非 active 版本的完整内容。

```json
// 成功响应 200
{
  "version_id": "ver-003",
  "content": "这是另一个版本的完整内容...",
  "content_type": "text/plain"
}
```

---

### PUT /conversations/{id}/nodes/{nodeId}/active-version — 切换版本

**Tauri command:** `set_active_version`

立即写库，不做 debounce。

```json
// 请求
{ "version_id": "ver-003" }

// 成功响应 200（无响应体）
```

---

### DELETE /conversations/{id}/nodes/{nodeId}/versions/{versionId} — 删除版本

**Tauri command:** `delete_version`

```json
// 成功响应 200
{
  "node_deleted": false,
  "new_active_version_id": "ver-001"
}

// 最后一个版本，node 一起删除
{
  "node_deleted": true,
  "new_active_version_id": null
}
```

---

## 7. AI 生成控制 (Generation)

### POST /conversations/{id}/send — 发送消息

**Tauri command:** `send_message`

**参数说明：**

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `content` | string | — | 必填，用户消息正文 |
| `stream` | bool | true | 流式返回（通过 Tauri Channel 推送 chunk） |
| `dry_run` | bool | false | 仅组装 prompt，不调用 AI，不创建消息 |

**正常调用示例：**
```json
// 请求
{ "content": "用 Rust 写一个快速排序" }

// 响应 200
{
  "user_node_id": "node-010",
  "user_version_id": "ver-011",
  "assistant_node_id": "node-012",
  "assistant_version_id": "ver-013"
}
```

**dry_run 调用示例：**
```json
// 请求
{ "content": "测试问题", "dry_run": true }

// 响应 200
{
  "messages": [
    { "role": "system", "content": "你是一个有帮助的助手" },
    { "role": "user", "content": "你好" },
    { "role": "assistant", "content": "你好！有什么可以帮你的？" },
    { "role": "user", "content": "测试问题" }
  ],
  "total_tokens_estimate": 156,
  "model": "gpt-4o"
}
```

**Tauri Channel 事件流（stream=true 时）：**
```json
{ "type": "chunk",     "conversation_id": "...", "node_id": "...", "version_id": "...", "delta": "当然" }
{ "type": "chunk",     "conversation_id": "...", "node_id": "...", "version_id": "...", "delta": "，这是..." }
{ "type": "completed", "conversation_id": "...", "node_id": "...", "version_id": "...", "prompt_tokens": 23, "completion_tokens": 312, "finish_reason": "stop", "model": "gpt-4o" }
```

**业务错误 (422)：**
```json
{ "error_code": "NO_CHANNEL", "message": "conversation has no channel configured" }
```

---

### POST /conversations/{id}/nodes/{nodeId}/reroll — Reroll

**Tauri command:** `reroll`

**行为差异：**

| node.role | 行为 |
|-----------|------|
| assistant | 在同 node 新建 version，开始生成 |
| user | 复制当前 active version 内容为新 user version → 创建新 assistant node → 开始生成 |

```json
// assistant reroll 响应 200
{
  "new_user_version_id": null,
  "assistant_node_id": "node-012",
  "assistant_version_id": "ver-020"
}

// user reroll 响应 200
{
  "new_user_version_id": "ver-030",
  "assistant_node_id": "node-031",
  "assistant_version_id": "ver-032"
}
```

---

### POST /generations/{versionId}/cancel — 取消生成

**Tauri command:** `cancel_generation`

幂等：version 不存在或已终结时同样返回 200。

**副作用：** 触发 CancellationToken → 后台写库 `status=cancelled` → 推送 `generation:cancelled` 事件

---

## 8. Tauri Channel 事件规范

所有 AI 生成事件通过 Tauri `Channel<GenerationEvent>` 推送（有序、有 back-pressure），不使用全局广播。

| 事件类型 | 触发时机 | 关键字段 |
|----------|---------|---------|
| `chunk` | 流式 chunk 到达 | `delta` |
| `completed` | 生成正常完成 | `prompt_tokens`, `completion_tokens`, `finish_reason`, `model` |
| `failed` | 生成失败 | `error` |
| `cancelled` | 用户取消 | — |
| `empty_rollback` | AI 返回空内容，自动回滚 | `node_deleted`, `fallback_version_id` |

**前端路由规则：** 按 `conversation_id` 路由到对应会话视图，按 `version_id` 匹配具体楼层。
