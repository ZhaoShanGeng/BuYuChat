# 步语 BuYu — API 参考手册

**版本：** 0.3
**阶段：** 当前实现
**最后更新：** 2026-04-08

本文解释当前代码已经落地的 Tauri IPC 接口。

## 1. 总体规则

### 1.1 调用方式

BuYu 实际使用的是 Tauri IPC，而不是 HTTP。

语义映射示例：

```ts
invoke("list_channels")
invoke("create_channel", { input })
invoke("update_conversation", { id, input })
invoke("send_message", { id, input, eventChannel })
```

OpenAPI 文档中的 REST 路径只用于表达“这个命令在概念上像哪个接口”。

### 1.2 命名与转换

当前工程里有三层数据形态：

| 层 | 例子 |
|------|------|
| Rust / Tauri 原始字段 | `channel_model_id`, `enabled_tools`, `error_details` |
| 前端 transport 转换后 | `channelModelId`, `enabledTools`, `errorDetails` |
| 前端页面最终使用 | 更适合 UI 的对象和联合类型 |

当前 transport 做了这些关键转换：

- `snake_case -> camelCase`
- `enabled_tools: string | null -> string[]`
- `send_message` 返回值补充 `kind`
- `GenerationEvent` 转换为前端联合类型

### 1.3 当前几个容易踩坑的真实约定

#### `update_conversation` 需要 `*_set`

因为 Rust 侧要区分：

- 字段没传
- 显式清空为 `null`
- 更新为某个具体值

所以当前请求体必须显式传：

- `agent_id_set`
- `channel_id_set`
- `channel_model_id_set`
- `enabled_tools_set`

示例：

```json
{
  "channel_id_set": true,
  "channel_id": null
}
```

这表示“把当前会话的渠道解绑”。

#### 某些字段在原始返回里是 JSON 字符串

当前后端真实返回中，以下字段仍是字符串：

- `api_keys`
- `thinking_tags`
- `enabled_tools`

其中前端当前的真实处理是：

- `enabled_tools` 会解析成 `string[]`
- `error_details` 在消息与错误返回中已经是结构化对象，不是 IPC 原始字符串
- `api_keys`、`thinking_tags` 仍按字符串透传

### 1.4 错误格式

统一错误结构：

```json
{
  "error_code": "NO_CHANNEL",
  "message": "conversation has no channel configured",
  "details": null
}
```

`details` 只有在部分失败场景下才有值，字段可能包括：

- `request_url`
- `request_method`
- `request_body`
- `response_status`
- `response_body`
- `raw_message`

## 2. 资源与命令对照

| 语义资源 | Tauri 命令 |
|------|------|
| 渠道 | `list_channels`, `get_channel`, `create_channel`, `update_channel`, `delete_channel`, `test_channel` |
| 模型 | `list_models`, `create_model`, `update_model`, `delete_model`, `fetch_remote_models` |
| Agent | `list_agents`, `get_agent`, `create_agent`, `update_agent`, `delete_agent` |
| 会话 | `list_conversations`, `get_conversation`, `create_conversation`, `update_conversation`, `delete_conversation` |
| 消息 | `list_messages`, `get_version_content`, `set_active_version`, `delete_version` |
| 生成 | `send_message`, `reroll`, `edit_message`, `cancel_generation` |
| 工具 | `list_builtin_tools` |

## 3. 渠道与模型

### 3.1 渠道字段

当前渠道对象包含这些重要字段：

```json
{
  "id": "0195...",
  "name": "OpenAI",
  "channel_type": "openai_compatible",
  "base_url": "https://api.openai.com",
  "api_key": "sk-...",
  "api_keys": "[\"sk-a\",\"sk-b\"]",
  "auth_type": "bearer",
  "models_endpoint": "/v1/models",
  "chat_endpoint": "/v1/chat/completions",
  "stream_endpoint": "/v1/chat/completions",
  "thinking_tags": "[\"think\",\"reasoning\"]",
  "enabled": true,
  "created_at": 1735000000000,
  "updated_at": 1735000000000
}
```

当前真实代码已经包含：

- `api_keys`
- `thinking_tags`
- `enabled` 在创建时也可传

### 3.2 模型字段

当前模型对象除了基础字段外，还包括：

- `temperature`
- `top_p`

示例：

```json
{
  "id": "0195...",
  "channel_id": "0195...",
  "model_id": "gpt-4o",
  "display_name": "GPT-4o",
  "context_window": 128000,
  "max_output_tokens": 16384,
  "temperature": "0.7",
  "top_p": "0.95"
}
```

## 4. 会话接口

### 4.1 会话详情真实字段

当前会话对象已经包含 `enabled_tools`：

```json
{
  "id": "0195...",
  "title": "新会话",
  "agent_id": null,
  "channel_id": null,
  "channel_model_id": null,
  "archived": false,
  "pinned": false,
  "enabled_tools": "[\"fetch\"]",
  "created_at": 1735000000000,
  "updated_at": 1735000000000
}
```

前端 transport 会把它转成：

```ts
enabledTools: ["fetch"]
```

### 4.2 更新会话示例

绑定渠道和模型：

```json
{
  "channel_id_set": true,
  "channel_id": "channel-1",
  "channel_model_id_set": true,
  "channel_model_id": "model-1"
}
```

清空 Agent：

```json
{
  "agent_id_set": true,
  "agent_id": null
}
```

启用工具：

```json
{
  "enabled_tools_set": true,
  "enabled_tools": ["fetch"]
}
```

## 5. 消息查询

### 5.1 `list_messages`

当前接口返回：

- 全部 `message node`
- 每个 node 下全部 `version`
- 只有 active version 带完整内容

active version 现在不只包含 `content`，还可能带：

- `thinking_content`
- `images`
- `files`
- `tool_calls`
- `tool_results`
- `error_code`
- `error_message`
- `error_details`
- `received_at`
- `completed_at`

示例：

```json
{
  "id": "node-1",
  "conversation_id": "conv-1",
  "author_agent_id": null,
  "role": "assistant",
  "order_key": "0000001735000000000-1-a3f9",
  "active_version_id": "ver-2",
  "versions": [
    {
      "id": "ver-2",
      "node_id": "node-1",
      "content": "这是正文",
      "thinking_content": "这是思考内容",
      "images": [],
      "files": [],
      "tool_calls": [],
      "tool_results": [],
      "status": "committed",
      "error_code": null,
      "error_message": null,
      "error_details": null,
      "model_name": "gpt-4o",
      "prompt_tokens": 123,
      "completion_tokens": 456,
      "finish_reason": "stop",
      "received_at": 1735000001111,
      "completed_at": 1735000002222,
      "created_at": 1735000001000
    }
  ],
  "created_at": 1735000001000
}
```

### 5.2 `get_version_content`

这个接口当前只返回纯正文：

```json
{
  "version_id": "ver-2",
  "content": "这是正文",
  "content_type": "text/plain"
}
```

它**不会**返回：

- `thinking_content`
- 图片
- 文件
- 工具调用
- 工具结果

## 6. 发送、重试、编辑

### 6.1 `send_message`

当前请求体支持：

- `content`
- `images`
- `files`
- `tool_results`
- `stream`
- `dry_run`

也就是说，现在已经不是“只支持纯文本发送”。

最小请求：

```json
{
  "content": "你好"
}
```

带图片和文件：

```json
{
  "content": "",
  "images": [
    { "base64": "...", "mime_type": "image/png" }
  ],
  "files": [
    { "name": "notes.txt", "base64": "...", "mime_type": "text/plain" }
  ]
}
```

dry run 返回：

```json
{
  "messages": [
    {
      "role": "system",
      "content": "你是一个有帮助的助手",
      "images": [],
      "files": [],
      "tool_calls": [],
      "tool_results": []
    }
  ],
  "total_tokens_estimate": 156,
  "model": "gpt-4o"
}
```

### 6.2 `reroll`

当前行为：

| 楼层角色 | 实际行为 |
|------|------|
| assistant | 在原 node 下创建一个新的 generating version |
| user | 优先复用紧邻的 assistant node；如果没有，则插入新的 assistant node |

### 6.3 `edit_message`

这是当前代码里已经存在的能力。

请求体：

```json
{
  "content": "修改后的内容",
  "resend": true,
  "stream": true
}
```

语义：

1. 先在当前 node 下创建一个新的 committed version
2. 把它切为 active version
3. 如果 `resend=true`
   - 编辑 assistant 楼层时，相当于基于新内容重新 reroll
   - 编辑 user 楼层时，会重新触发后续 assistant 回复

响应：

```json
{
  "edited_version_id": "ver-edited",
  "assistant_node_id": "node-assistant",
  "assistant_version_id": "ver-assistant"
}
```

如果 `resend=false`，后两个字段会是 `null`。

## 7. 生成事件

当前事件种类如下：

| 事件 | 含义 |
|------|------|
| `chunk` | 流式正文 / thinking / tool call delta |
| `completed` | 正常结束 |
| `failed` | 失败结束 |
| `cancelled` | 被取消 |
| `empty_rollback` | 空响应自动回滚 |
| `tool_call_start` | 工具调用开始 |
| `tool_result` | 工具执行结果回填 |

### 7.1 `chunk`

现在的 `chunk` 事件不仅有 `delta`，还有：

- `reasoning_delta`
- `tool_call_deltas`

示例：

```json
{
  "type": "chunk",
  "conversation_id": "conv-1",
  "node_id": "node-1",
  "version_id": "ver-1",
  "delta": "",
  "reasoning_delta": "我先分析一下",
  "tool_call_deltas": []
}
```

### 7.2 `failed`

当前失败事件字段为：

```json
{
  "type": "failed",
  "conversation_id": "conv-1",
  "node_id": "node-1",
  "version_id": "ver-1",
  "error_code": "AI_REQUEST_FAILED",
  "error_message": "upstream returned 503",
  "error_details": {
    "response_status": 503,
    "raw_message": "service unavailable"
  }
}
```

### 7.3 工具事件

工具调用开始：

```json
{
  "type": "tool_call_start",
  "conversation_id": "conv-1",
  "node_id": "node-1",
  "version_id": "ver-1",
  "tool_calls": [
    {
      "id": "call-1",
      "name": "fetch",
      "arguments_json": "{\"url\":\"https://example.com\"}"
    }
  ]
}
```

工具结果回填：

```json
{
  "type": "tool_result",
  "conversation_id": "conv-1",
  "node_id": "node-1",
  "version_id": "ver-1",
  "results": [
    {
      "tool_call_id": "call-1",
      "name": "fetch",
      "content": "网页纯文本内容",
      "is_error": false
    }
  ]
}
```

## 8. 内置工具接口

当前已存在：

- `list_builtin_tools`

返回值：

```json
[
  {
    "name": "fetch",
    "description": "获取网页内容（HTML 转纯文本，最大 32KB）"
  }
]
```

这是当前接口的一部分。

## 9. 当前常见错误码

| 错误码 | 场景 |
|------|------|
| `VALIDATION_ERROR` | 输入校验失败 |
| `INVALID_URL` | 渠道地址不合法 |
| `MODEL_ID_CONFLICT` | 同渠道下模型 ID 冲突 |
| `NOT_FOUND` | 资源不存在 |
| `NO_AGENT` | 会话未绑定 Agent |
| `AGENT_DISABLED` | 会话绑定的 Agent 已禁用 |
| `NO_CHANNEL` | 会话未绑定渠道 |
| `CHANNEL_DISABLED` | 渠道已禁用 |
| `NO_MODEL` | 会话未绑定模型 |
| `NOT_LAST_USER_NODE` | user reroll 不满足业务规则 |
| `VERSION_NOT_IN_NODE` | 版本不属于指定楼层 |
| `MESSAGE_STILL_GENERATING` | 不能编辑仍在生成中的版本 |
| `CHANNEL_UNREACHABLE` | 渠道不可达 |
| `AI_REQUEST_FAILED` | 上游模型请求失败 |
| `INTERNAL_ERROR` | 内部错误 |

## 10. 结论

当前接口层最重要的变化有 5 个：

1. 消息系统已经支持附件、thinking、工具调用和工具结果
2. 会话更新接口必须使用 `*_set` 标记
3. 生成事件已经扩展到工具相关事件
4. 新增了 `edit_message`
5. 新增了 `list_builtin_tools`

如果后续代码继续演进，应优先同步：

- `docs/03_database.md`
- `docs/04_api_openapi.yaml`
- `docs/05_api_reference.md`
