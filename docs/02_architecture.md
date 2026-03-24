# 步语 BuYu — 架构设计

**版本：** 0.1
**阶段：** MVP（P0）

---

## 1. 系统分层

```
┌──────────────────────────────────────────────────┐
│  Frontend (Svelte 5 + TypeScript)                │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐ │
│  │   Views     │  │   Stores   │  │ Transport  │ │
│  │  (Svelte)   │  │  ($state)  │  │   Layer    │ │
│  └────────────┘  └────────────┘  └─────┬──────┘ │
│                                        │         │
├────────────────────────────────────────┼─────────┤
│  Tauri IPC Bridge                      │         │
│  invoke() + Channel                    │         │
├────────────────────────────────────────┼─────────┤
│  Backend (Rust)                        ▼         │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐ │
│  │  Commands   │  │  Services  │  │ Generation │ │
│  │  (handlers) │  │  (业务层)   │  │  Engine    │ │
│  └────────────┘  └────────────┘  └────────────┘ │
│  ┌────────────┐  ┌────────────┐                  │
│  │    Repo     │  │  AI Client │                  │
│  │  (SQLite)   │  │  (HTTP)    │                  │
│  └────────────┘  └────────────┘                  │
└──────────────────────────────────────────────────┘
```

---

## 2. 层级职责

### 2.1 Frontend

| 模块 | 职责 |
|------|------|
| **Views** | Svelte 5 组件，纯 UI 渲染，不含业务逻辑 |
| **Stores** | Svelte 5 runes (`$state`, `$derived`)，管理前端状态 |
| **Transport** | 封装 `invoke()` 调用，将 REST 风格路径映射为 Tauri command 名；管理 `Channel` 事件监听 |

**Transport 层映射示例：**
```typescript
// transport.ts
export async function createChannel(input: CreateChannelInput): Promise<Channel> {
  return invoke("create_channel", { input });
}

export async function sendMessage(
  conversationId: string,
  input: SendMessageInput,
  onEvent: (event: GenerationEvent) => void
): Promise<SendMessageResult> {
  const channel = new Channel<GenerationEvent>();
  channel.onmessage = onEvent;
  return invoke("send_message", { conversationId, input, channel });
}
```

### 2.2 Tauri IPC Bridge

- 前端通过 `invoke(commandName, args)` 调用 Rust 命令
- AI 生成的流式事件通过 `Channel<T>` 从 Rust 推送到前端
- Channel 是有序、有 back-pressure 的，不使用 Tauri 全局事件广播
- 每个 `send_message` / `reroll` 调用独立创建一个 Channel 实例

### 2.3 Backend

| 模块 | 职责 | 依赖 |
|------|------|------|
| **Commands** | Tauri `#[tauri::command]` 入口，参数校验、错误转换 | Services |
| **Services** | 业务逻辑编排（事务管理、状态机流转、上下文构建） | Repo, AI Client |
| **Repo** | 数据访问层，SQL 查询（sqlx），返回领域模型 | SQLite |
| **AI Client** | aisdk + aisdk-macros 封装，OpenAI-compatible API 调用；渠道探测仍复用 `AppState.http_client` | aisdk |
| **Generation Engine** | 后台异步任务，管理 CancellationToken，通过 Channel 推送事件 | AI Client, Repo |

---

## 3. AI 生成流水线

```
用户点击发送
    │
    ▼
[send_message command]
    │
    ├─ 1. 校验配置（agent/channel/model 是否存在且启用）
    ├─ 2. 创建 user node + version (status=committed)
    ├─ 3. 创建 assistant node + version (status=generating)
    ├─ 4. 更新 conversations.updated_at
    ├─ 5. 立即返回 SendMessageResult（4个 ID）
    │
    └─ 6. spawn 异步任务 ──►  [Generation Engine]
                                    │
                                    ├─ 构建上下文（按 order_key 取各 node 的 active version）
                                    ├─ 拼接 system_prompt（实时读取，不快照）
                                    │
                                    ├─ stream=true:  逐 chunk 推送 Channel 事件
                                    │   └─ 每 N 个 chunk 批量 UPDATE content（不逐字写库）
                                    ├─ stream=false: 等待完整响应后一次推送
                                    │
                                    ├─ dry_run=true: 只返回组装的 prompt 元信息，不调用 AI
                                    │
                                    ├─ 正常完成 → status=committed, 推 generation:completed
                                    ├─ 空内容   → 执行空内容回滚逻辑
                                    ├─ 错误     → status=failed, 推 generation:failed
                                    └─ 取消     → status=cancelled, 推 generation:cancelled
```

### 3.1 Chunk 写库策略

- **不逐字写库**：流式 chunk 先在内存 buffer 中累积
- **定期刷盘**：每积累 ~500 字符或每 2 秒（以先到为准），`UPDATE message_versions SET content = ? WHERE id = ?`
- **终态写库**：生成结束时最终 flush，同时写 status/tokens/finish_reason
- **conversations.updated_at**：仅在终态（completed/failed/cancelled）时更新一次，不在 chunk 时更新

### 3.2 取消机制

```
DashMap<VersionId, CancellationToken>
    │
    ├─ send_message / reroll 时插入
    ├─ cancel_generation 时触发 token.cancel()
    ├─ 生成任务在每个 chunk 前检查 token.is_cancelled()
    └─ 生成结束后从 DashMap 移除
```

- 取消不存在的 version_id → 忽略，返回 OK（幂等）
- 应用重启后 DashMap 为空 → 启动清理已将 generating 改为 failed

### 3.3 多会话并发

- 每个生成任务是独立的 `tokio::spawn` 异步任务
- Channel 事件携带 `conversation_id` + `node_id` + `version_id`
- 前端按 `conversation_id` 路由到对应会话视图
- SQLite WAL 模式支持并发读，写操作串行但锁粒度小

---

## 4. 状态管理

### 4.1 前端状态

```
conversations: Map<string, Conversation>     // 会话列表
activeConversationId: string | null           // 当前活跃会话
messages: Map<string, MessageNode[]>          // conversation_id → nodes
generatingVersions: Set<string>              // 正在生成的 version_id 集合
pendingInput: Map<string, string>            // conversation_id → 待恢复的输入内容
```

### 4.2 后端状态（内存）

```
AppState {
    db: SqlitePool,                                    // 连接池
    cancellation_tokens: DashMap<String, CancellationToken>, // 生成任务取消令牌
    http_client: reqwest::Client,                       // 复用的 HTTP 客户端
}
```

---

## 5. 错误处理架构

### 5.1 错误分类

```rust
enum AppError {
    // 400 — 入参校验
    Validation { field: String, message: String },

    // 404 — 资源不存在
    NotFound { resource: String, id: String },

    // 409 — 唯一约束冲突
    Conflict { message: String },

    // 422 — 业务规则不满足
    NoAgent,
    AgentDisabled,
    NoChannel,
    ChannelDisabled,
    NoModel,
    NotLastUserNode,

    // 500 — 内部错误
    Database(sqlx::Error),
    AiClient { status: u16, body: String },
    Internal(String),
}
```

### 5.2 错误响应格式

所有错误统一返回：
```json
{
  "error_code": "NO_CHANNEL",
  "message": "conversation has no channel configured"
}
```

- `error_code`：机器可读，前端用于 i18n 翻译
- `message`：英文调试信息，不直接展示给用户

---

## 6. 目录结构规划

```
src-tauri/src/
├── main.rs                    // 入口
├── lib.rs                     // Tauri Builder 配置 + command 注册
├── commands/                  // Tauri command handlers
│   ├── mod.rs
│   ├── channels.rs
│   ├── models.rs
│   ├── agents.rs
│   ├── conversations.rs
│   └── messages.rs
├── services/                  // 业务逻辑层
│   ├── mod.rs
│   ├── channel_service/
│   │   ├── mod.rs
│   │   ├── crud.rs
│   │   ├── validation.rs
│   │   └── connectivity.rs
│   ├── model_service.rs
│   ├── agent_service.rs
│   ├── conversation_service.rs
│   ├── message_service.rs
│   └── generation_engine.rs   // AI 生成流水线
├── repo/                      // 数据访问层
│   ├── mod.rs
│   ├── channel_repo.rs
│   ├── model_repo.rs
│   ├── agent_repo.rs
│   ├── conversation_repo.rs
│   ├── message_repo.rs
│   └── migrations.rs          // DDL 执行
├── ai/                        // AI 客户端（基于 aisdk）
│   ├── mod.rs
│   ├── adapter.rs             // aisdk 适配层，封装请求构建与响应解析
│   └── types.rs               // 补充类型定义（如 GenerationEvent）
├── models/                    // 领域模型 (structs)
│   ├── mod.rs
│   ├── channel.rs
│   ├── agent.rs
│   ├── conversation.rs
│   └── message.rs
├── error.rs                   // AppError 定义
└── state.rs                   // AppState 定义

src/
├── main.ts
├── App.svelte
├── app.css
├── lib/
│   ├── transport/             // Tauri invoke 封装
│   │   ├── index.ts
│   │   ├── channels.ts
│   │   ├── agents.ts
│   │   ├── conversations.ts
│   │   └── messages.ts
│   ├── stores/                // Svelte 5 runes 状态
│   │   ├── conversations.svelte.ts
│   │   ├── messages.svelte.ts
│   │   └── settings.svelte.ts
│   ├── types/                 // TypeScript 类型定义
│   │   └── index.ts
│   └── utils/
│       └── order-key.ts
├── components/                // 可复用 UI 组件
│   ├── Sidebar.svelte
│   ├── ChatView.svelte
│   ├── MessageBubble.svelte
│   ├── VersionSwitcher.svelte
│   └── ...
└── views/                     // 页面级组件
    ├── ChatPage.svelte
    └── SettingsPage.svelte
```

---

## 7. 技术选型

| 层级 | 技术 | 理由 |
|------|------|------|
| 前端框架 | Svelte 5 | 编译时优化，runes 语法简洁 |
| 样式 | Tailwind CSS 4 | CSS-first 配置，无 JS 运行时 |
| 组件库 | bits-ui | headless，不限制样式 |
| 图标 | lucide-svelte | 轻量 tree-shakeable |
| 桌面壳 | Tauri v2 | Rust 后端，内存占用小 |
| 数据库 | SQLite (sqlx) | 单文件嵌入式，WAL 并发 |
| HTTP | reqwest（aisdk 内部依赖） | aisdk 封装了 HTTP 细节 |
| AI SDK | aisdk + aisdk-macros | OpenAI-compatible 请求/响应/SSE 解析 |
| UUID | uuid v7 | 时间有序，可排序 |
| 并发 | tokio + DashMap | 异步运行时 + 并发安全 HashMap |
| 日志 | tracing + tracing-subscriber | 结构化日志，异步友好 |

---

## 8. 日志策略

### 8.1 日志框架

使用 `tracing` + `tracing-subscriber`（Rust 生态标准，tokio 原生集成）。

### 8.2 日志级别

| 级别 | 用途 | 示例 |
|------|------|------|
| `ERROR` | 不可恢复错误，需要用户关注 | 数据库迁移失败、AI 请求 5xx |
| `WARN` | 可恢复的异常情况 | order_key 冲突重试、chunk 写库失败 |
| `INFO` | 关键业务操作 | 会话创建、消息发送、生成完成 |
| `DEBUG` | 调试信息 | SQL 执行、HTTP 请求/响应、Channel 事件 |
| `TRACE` | 极细粒度 | 流式 chunk 内容、buffer 状态 |

### 8.3 生产 vs 开发

| 环境 | 默认级别 | 输出目标 |
|------|---------|---------|
| `pnpm tauri dev` | DEBUG | 终端 stdout（彩色格式） |
| 生产 build | INFO | 文件（轮转） + 终端 |

### 8.4 日志文件

```
{app_data_dir}/logs/
├── buyu.log          # 当前日志
├── buyu.log.1        # 上一个轮转
└── buyu.log.2        # ...
```

- 路径：`tauri::api::path::app_log_dir()`
- 单文件上限：10MB
- 保留：最多 5 个轮转文件（总计 ≤ 50MB）
- 格式：`2026-03-24T20:30:00.123Z INFO buyu::services::message_service: message sent conversation_id="..." version_id="..."`

### 8.5 结构化字段

```rust
use tracing::{info, warn, instrument};

#[instrument(skip(pool), fields(conversation_id = %id))]
async fn send_message(pool: &SqlitePool, id: &str, input: SendMessageInput) -> Result<...> {
    info!(content_len = input.content.len(), "sending message");
    // ...
    warn!(retry = attempt, "order_key collision, retrying");
}
```

### 8.6 前端日志

MVP 使用 `console.log` / `console.error`，P1 考虑 `tauri-plugin-log` 将前端日志写入同一日志文件。
