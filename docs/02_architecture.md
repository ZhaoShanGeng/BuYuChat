# 步语 BuYu — 架构设计

**版本：** 0.3
**阶段：** MVP 迭代中
**最后更新：** 2026-04-04

本文只描述仓库当前代码已经存在的结构，不再保留已废弃的 `aisdk` 方案假设。

## 1. 系统分层

```text
┌──────────────────────────────────────────────────────┐
│ Frontend (Svelte 5 + TypeScript)                    │
│ 组件 / 工作台状态 / invoke transport / 富文本渲染       │
├──────────────────────────────────────────────────────┤
│ Tauri IPC                                           │
│ invoke() 命令调用 + Channel<GenerationEvent> 流式事件 │
├──────────────────────────────────────────────────────┤
│ Backend (Rust)                                      │
│ commands -> services -> repo -> SQLite              │
│              └-> ai::adapter -> OpenAI-compatible   │
│              └-> mcp / ToolRegistry                 │
└──────────────────────────────────────────────────────┘
```

## 2. 当前代码中的核心模块

### 2.1 前端

| 模块 | 位置 | 职责 |
|------|------|------|
| 页面与工作台 | `src/components/` | 聊天区、侧边栏、设置页、窗口控件 |
| 状态层 | `src/components/*.svelte.ts` | 使用 Svelte 5 runes 管理工作台与设置状态 |
| 传输层 | `src/lib/transport/` | 封装 Tauri `invoke()`，承接 conversations / messages / models / channels |
| 富文本渲染 | `src/lib/rich-text.ts` 等 | Markdown、代码高亮、KaTeX 渲染 |

### 2.2 Tauri 后端

| 模块 | 位置 | 职责 |
|------|------|------|
| 命令入口 | `src-tauri/src/commands/` | `#[tauri::command]` 暴露资源 CRUD、消息生成、工具查询 |
| 业务层 | `src-tauri/src/services/` | 渠道、模型、会话、消息、生成调度 |
| 数据访问 | `src-tauri/src/repo/` | SQLx 查询、事务、chunk 持久化 |
| AI 适配层 | `src-tauri/src/ai/adapter.rs` | 自建 OpenAI-compatible 请求/响应与 SSE 解析 |
| MCP / 工具 | `src-tauri/src/mcp/` | 内置工具注册、MCP stdio/http 传输 |
| 应用状态 | `src-tauri/src/state.rs` | `SqlitePool`、`reqwest::Client`、取消令牌、并发信号量、工具注册表 |

## 3. 运行时事实

### 3.1 命令面

`src-tauri/src/lib.rs` 当前注册的命令覆盖：

- `agents`
- `channels`
- `conversations`
- `messages`
- `models`
- `tools`

其中消息链路包含：

- `send_message`
- `reroll`
- `edit_message`
- `cancel_generation`
- `get_version_content`
- `set_active_version`
- `delete_version`

### 3.2 共享状态

`AppState` 当前字段如下：

```rust
AppState {
    db: SqlitePool,
    http_client: reqwest::Client,
    cancellation_tokens: Arc<DashMap<String, CancellationToken>>,
    generation_semaphore: Arc<Semaphore>,
    tool_registry: Arc<ToolRegistry>,
}
```

初始化阶段会：

- 建立 SQLite 连接池
- 执行迁移
- 把遗留 `generating` 版本修正为 `failed`
- 初始化内置工具注册表

### 3.3 AI 接入

当前实现不是 `aisdk`，而是仓库内自建适配层：

- 使用 `reqwest` 访问 OpenAI-compatible `models` / `chat/completions`
- 自行构建兼容的请求体与响应体
- 自行解析 SSE 流
- 支持 `text`、`reasoning`、图片、文件和 tool call delta
- 支持轮询式 API Key 选择
- 支持温度、`top_p`、`reasoning_effort`

相关代码入口：

- `src-tauri/src/ai/adapter.rs`
- `src-tauri/src/channel_types.rs`
- `src-tauri/src/services/generation_engine.rs`

### 3.4 工具与 MCP

当前后端已经有两层能力：

- 内置工具注册表 `ToolRegistry`
- MCP 传输模块 `src-tauri/src/mcp/`

仓库当前默认注册了一个内置工具：

- `fetch`：抓取网页内容并返回纯文本结果

生成引擎在模型返回 `tool_calls` 时会：

1. 持久化 tool call chunk
2. 发送 `ToolCallStart` 事件
3. 执行工具
4. 把 tool result 回填到 prompt
5. 最多进行 10 轮工具调用循环

## 4. 消息生成流水线

```text
前端 send_message / reroll
    -> message_service 创建 user / assistant 版本
    -> generation_engine::spawn_generation
    -> Semaphore 控制并发
    -> AiAdapter 发起普通或流式请求
    -> Channel<GenerationEvent> 推送前端
    -> message_contents 按 chunk 落库
    -> committed / failed / cancelled / empty_rollback
```

当前实现要点：

- 流式正文与 thinking 分开持久化
- flush 条件为 `2048 bytes` 或 `2 秒`
- 取消基于 `CancellationToken`
- 空内容会触发自动回滚
- 启动时会清理上次异常退出留下的 `generating`

## 5. 目录结构

```text
src-tauri/src/
├── ai/
├── bin/
├── commands/
├── mcp/
├── models/
├── repo/
├── services/
├── utils/
├── channel_types.rs
├── error.rs
├── lib.rs
├── main.rs
└── state.rs

src/
├── components/
├── lib/
│   ├── components/
│   ├── transport/
│   └── ...
├── styles/
├── app.css
└── main.ts

.github/
└── workflows/
    ├── ci.yml
    └── release.yml
```

## 6. 技术栈

| 层级 | 技术 | 当前用途 |
|------|------|----------|
| 前端框架 | Svelte 5 | 工作台、设置页、状态编排 |
| 样式 | Tailwind CSS 4 | 原子样式与排版 |
| 组件基础 | bits-ui | Headless UI 基础能力 |
| 桌面壳 | Tauri v2 | IPC、窗口管理、打包 |
| 窗口状态 | `tauri-plugin-window-state` | 记忆窗口尺寸与位置 |
| 数据库 | SQLite + SQLx | 本地持久化、迁移、查询 |
| HTTP | reqwest | 模型拉取、聊天补全、工具请求 |
| 异步 | tokio + tokio-util | 并发任务、取消控制、定时 flush |
| 并发映射 | DashMap | 生成任务取消令牌 |
| 工具协议 | MCP + 自定义 ToolRegistry | 内置工具和外部工具接入 |

## 7. 工程治理

- 版本事实来源：`package.json`、`src-tauri/Cargo.toml`、`src-tauri/tauri.conf.json`
- 统一脚本：`scripts/version.mjs`
- CI：版本一致性、前端检查/测试/构建、Rust 测试与 `clippy`
- Release：手动触发构建，或在 `v*` tag 上发布安装包

## 8. 当前边界

以下内容在代码里还没有形成稳定公共契约，不应在文档里写成既成事实：

- 统一日志框架与文件轮转
- OS Keychain 存储 API Key
- 云端服务端部署
- 多平台安装包矩阵

这些能力如果后续落地，再增量更新本文。
