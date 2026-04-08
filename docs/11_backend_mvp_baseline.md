# 步语 BuYu — 后端基线

**版本：** 0.2
**阶段：** 当前工作树基线
**最后更新：** 2026-04-08

本文描述的是仓库当前后端代码已经具备的结构与能力。

## 1. 基线范围

当前后端已经覆盖：

- 渠道管理与连通性测试
- 模型管理与远程模型拉取
- Agent 管理
- 会话管理
- 消息楼层与版本系统
- 流式 / 非流式生成
- 取消生成
- 版本切换、版本删除、正文按需加载
- 内置工具调用与 MCP 模块接入

当前不在稳定基线内：

- OS Keychain 凭证存储
- 桌面端窗口级 E2E
- 完整安全加固策略

## 2. 当前代码结构

### 2.1 模块

`src-tauri/src/` 当前目录：

```text
ai/
bin/
commands/
mcp/
models/
repo/
services/
utils/
channel_types.rs
error.rs
lib.rs
main.rs
state.rs
```

### 2.2 命令层

`src-tauri/src/lib.rs` 当前注册了：

- `agents::*`
- `channels::*`
- `conversations::*`
- `messages::*`
- `models::*`
- `tools::list_builtin_tools`

### 2.3 共享状态

`AppState` 当前维护：

- `SqlitePool`
- `reqwest::Client`
- `DashMap<String, CancellationToken>`
- `Semaphore`
- `ToolRegistry`

## 3. 数据与持久化基线

### 3.1 数据库

当前持久化使用 `SQLite + SQLx`，启动时自动运行迁移。

核心表仍然是：

- `api_channels`
- `api_channel_models`
- `agents`
- `conversations`
- `message_nodes`
- `message_versions`
- `message_contents`

### 3.2 内容存储策略

当前实现中：

- 元数据写在 `message_versions`
- 正文、thinking、工具调用、工具结果等内容写在 `message_contents`
- `list_messages` 会把 active version 的正文、thinking、附件和工具相关内容聚合返回
- 非 active version 仍只返回版本元数据；切换后通过 `get_version_content` 按需补正文

## 4. AI 与生成基线

### 4.1 AI 接入方式

当前 AI 接入使用自建 OpenAI-compatible 适配层：

- 模型探测和远程模型拉取走 `GET /models`
- 聊天生成走 `POST /chat/completions`
- 流式响应由后端自行解析 SSE
- 支持文本、thinking、图片、文件与 tool call delta

关键文件：

- `src-tauri/src/ai/adapter.rs`
- `src-tauri/src/services/generation_engine.rs`

### 4.2 生成任务

生成链路当前特性：

- `send_message` 与 `reroll` 会创建 `generating` 版本
- 后台任务通过 `tauri::async_runtime::spawn` 执行
- 并发上限由 `Semaphore(5)` 控制
- 取消通过 `CancellationToken` 实现
- 启动时会把遗留 `generating` 修成 `failed`

### 4.3 流式刷盘

当前规则：

- 每 `2048 bytes` 或每 `2 秒` flush 一次
- `text/plain` 和 `text/thinking` 分开写入
- 生成完成后再统一写入 `prompt_tokens`、`completion_tokens`、`finish_reason`、`model`

### 4.4 工具调用

当前后端已支持工具调用回路：

1. 模型返回 `tool_calls`
2. 后端持久化 tool call chunk
3. 通过 `ToolRegistry` 执行工具
4. 持久化 tool result
5. 把结果作为新的 prompt 消息继续补全

当前默认内置工具：

- `fetch`

## 5. 错误与状态机基线

`message_versions.status` 当前使用：

- `generating`
- `committed`
- `failed`
- `cancelled`

当前还包括以下行为：

- 空内容自动回滚
- 重复取消保持幂等
- 对外统一返回结构化业务错误

## 6. 测试基线

仓库当前已经有：

- Repo 测试
- Command 测试
- 前端 transport / 状态相关测试
- `cargo clippy` 静态检查

本地完整门禁入口：

```bash
pnpm verify
```

其中包括：

- 版本一致性检查
- 前端类型检查、测试、构建
- Rust 测试与 `clippy`

## 7. 当前结论

后端当前已经不是“只有 CRUD 的骨架”，而是已经具备：

- 可运行的本地数据库与迁移
- 可流式生成的消息系统
- 可取消、可回滚、可切版本的生成链路
- 内置工具执行与 MCP 扩展入口

后续文档如果再描述后端能力，应以本文和实际代码为准。
