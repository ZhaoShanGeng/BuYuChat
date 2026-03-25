# API 总览

**版本：** 0.2 | **传输层：** Tauri IPC | **鉴权：** 无（本地单用户桌面应用）

---

## 1. IPC 机制

BuYu 使用两种 Tauri IPC 机制，按场景选择：

| 机制 | 调用方式 | 适用场景 | 特性 |
|------|----------|----------|------|
| `invoke` | `await invoke(cmd, args)` | 所有 CRUD、查询、控制类操作 | 一次请求，一次响应，同步等待 |
| `Channel` | `invoke(cmd, { ...args, channel })` | AI 流式生成 chunk 推送 | 一次请求，多次推送；有序、有背压；IPC 不经网络栈 |

### invoke 调用示例

```typescript
// 前端
const channel = await invoke<Channel>('create_channel', {
  input: { name: 'My OpenAI', baseUrl: 'https://api.openai.com', apiKey: 'sk-xxx' }
});
```

### Channel 流式调用示例

```typescript
// 前端
import { Channel } from '@tauri-apps/api/core';

const onEvent = new Channel<GenerationEvent>();
onEvent.onmessage = (event) => {
  if (event.event === 'generation:chunk') {
    appendChunk(event.data.versionId, event.data.delta);
  } else if (event.event === 'generation:completed') {
    markCompleted(event.data.versionId, event.data);
  }
};

const result = await invoke<SendMessageResult>('send_message', {
  conversationId: 'xxx',
  input: { content: '你好' },
  channel: onEvent,  // 传入 Channel 对象
});
// result 立即返回 { userNodeId, userVersionId, assistantNodeId, assistantVersionId }
// chunk 通过 onEvent.onmessage 异步回调
```

---

## 2. REST 风格路径与 Tauri 命令映射

前端 Transport 层将 REST 路径翻译为 `invoke()` 命令名，规则：

```
GET    /channels              → invoke('list_channels')
POST   /channels              → invoke('create_channel')
GET    /channels/:id          → invoke('get_channel')
PUT    /channels/:id          → invoke('update_channel')
DELETE /channels/:id          → invoke('delete_channel')
```

路径参数拍平为参数字段（如 `id`、`channelId`、`nodeId`、`versionId`）。

---

## 3. 通用约定

### 3.1 ID 格式

所有资源 ID 使用 **UUID v7**（时间有序），TEXT 类型存储，格式：`019587ab-xxxx-7xxx-xxxx-xxxxxxxxxxxx`。

### 3.2 时间戳

所有时间字段（`created_at`、`updated_at`）均为 **Unix 毫秒时间戳**（INTEGER）。

### 3.3 空值语义

| 字段 | NULL 含义 |
|------|-----------|
| `conversations.agent_id` | 未配置 Agent |
| `conversations.channel_id` | 未配置渠道 |
| `conversations.channel_model_id` | 未配置模型 |
| `message_nodes.active_version_id` | 尚无版本（node 刚创建）|
| `message_versions.model_name` | user 消息，无模型调用 |
| `agents.system_prompt` | 无系统提示词，等价于空字符串 |

### 3.4 布尔值

JSON 响应中布尔字段使用 `true/false`；数据库中 INTEGER `1/0`，应用层转换。

### 3.5 端点拼接规则

```
final_url = base_url.trim_end_matches('/') + endpoint
```

| channel_type | auth_type | models_endpoint | chat/stream_endpoint |
|---|---|---|---|
| `openai_compatible` | `bearer` | `/v1/models` | `/v1/chat/completions` |

字段为 NULL 时使用上表默认值。

---

## 4. 状态码映射

| 语义 | 状态码 | 说明 |
|------|--------|------|
| 查询/操作成功 | `200 OK` | 通用成功，含更新、取消等 |
| 创建成功 | `201 Created` | POST 创建资源成功 |
| 删除成功 | `204 No Content` | DELETE 成功，无响应体 |
| 参数校验失败 | `400 Bad Request` | 字段格式错误、必填项缺失 |
| 资源不存在 | `404 Not Found` | ID 对应记录不存在 |
| 业务规则不满足 | `422 Unprocessable Entity` | 配置缺失、状态不允许等 |
| 唯一约束冲突 | `409 Conflict` | 如 model_id 重复 |
| 服务内部错误 | `500 Internal Server Error` | 数据库异常、order_key 重试超限等 |

---

## 5. 统一错误响应格式

所有错误均返回统一结构：

```typescript
interface ApiError {
  error_code: string;  // 机器可读，前端用于 i18n
  message: string;     // 英文调试信息，不直接展示给用户
}
```

### 错误码速查表

| error_code | 状态码 | 含义 |
|-----------|--------|------|
| `NAME_EMPTY` | 400 | name 字段为空 |
| `INVALID_URL` | 400 | base_url 格式不合法 |
| `MODEL_ID_EMPTY` | 400 | model_id 为空 |
| `CONTENT_EMPTY` | 400 | 消息内容为空 |
| `VERSION_NOT_IN_NODE` | 400 | versionId 不属于该 node |
| `NOT_FOUND` | 404 | 资源不存在 |
| `MODEL_ID_CONFLICT` | 409 | 同渠道下 model_id 重复 |
| `NO_AGENT` | 422 | 会话未配置 Agent |
| `AGENT_DISABLED` | 422 | Agent 已禁用 |
| `NO_CHANNEL` | 422 | 会话未配置渠道 |
| `NO_MODEL` | 422 | 会话未配置模型 |
| `CHANNEL_DISABLED` | 422 | 渠道已禁用 |
| `NOT_LAST_USER_NODE` | 422 | user node 不是末尾楼层，不可 reroll |
| `ORDER_KEY_CONFLICT` | 500 | order_key 重试 3 次仍冲突 |
| `INTERNAL_ERROR` | 500 | 其他内部错误 |

---

## 6. 生成事件规范（Channel 推送）

所有 AI 生成事件通过 Tauri `Channel` 推送（有序、有背压），不使用全局广播。

前端通过 `conversationId` 路由到对应会话视图，通过 `versionId` 匹配具体楼层版本。

### 事件类型定义

```typescript
type GenerationEvent =
  | { event: 'generation:chunk';          data: ChunkData }
  | { event: 'generation:completed';      data: CompletedData }
  | { event: 'generation:failed';         data: FailedData }
  | { event: 'generation:cancelled';      data: CancelledData }
  | { event: 'generation:empty_rollback'; data: EmptyRollbackData };

interface ChunkData {
  conversationId: string;
  nodeId:         string;
  versionId:     string;
  delta:          string;   // 本次新增内容片段
}

interface CompletedData {
  conversationId:   string;
  nodeId:           string;
  versionId:        string;
  promptTokens:     number;
  completionTokens: number;
  finishReason:     string; // 'stop' | 'length' | 'content_filter'
  model:            string; // 实际调用的 model_id
}

interface FailedData {
  conversationId: string;
  nodeId:         string;
  versionId:      string;
  errorCode:      string;
  message:        string;
}

interface CancelledData {
  conversationId: string;
  nodeId:         string;
  versionId:      string;
}

interface EmptyRollbackData {
  conversationId:    string;
  nodeId:            string;
  nodeDeleted:       boolean; // true=node 已被删除（唯一版本为空）
  fallbackVersionId: string | null; // nodeDeleted=true 时为 null
}
```

### 事件流转时序

```
invoke('send_message', { channel }) ──→ 立即返回 SendMessageResult
        │
        ├─ generation:chunk        × N   （流式内容片段）
        ├─ generation:chunk        × N
        │
        └─ generation:completed         （正常结束）
           OR generation:failed         （网络/API 错误）
           OR generation:cancelled      （用户取消）
           OR generation:empty_rollback （AI 返回空内容）
```

### 重启后注意事项

应用重启后，后端启动清理将所有 `generating` 状态改为 `failed`，内存中的 Channel 和 CancellationToken 全部失效。前端**不应**在重启后尝试重新订阅旧 `versionId` 的事件，直接刷新消息列表读取终态即可。

---

## 7. 业务状态机

### message_versions.status

```
                    ┌─────────────────────────────────────┐
                    │         generating                  │
                    └──────┬──────────┬──────────┬────────┘
                           │          │          │
                    content≠""   网络/API错误  用户取消
                           │          │          │
                    ┌──────▼──┐  ┌────▼───┐  ┌──▼───────┐
                    │committed│  │ failed │  │cancelled │
                    └─────────┘  └────────┘  └──────────┘
                    （终态）      （终态）      （终态）

content="" 且 finish_reason 到达 → 空内容回滚（删 version 或 node）
status=generating 且应用重启    → 批量改为 failed
```

| 转换 | 触发条件 |
|------|---------|
| `generating → committed` | 生成正常完成且 content 不为空 |
| `generating → failed` | 网络/API 错误，或应用启动清理 |
| `generating → cancelled` | 用户调用 cancel_generation |
| `generating → (deleted)` | AI 返回空内容，执行空内容回滚 |