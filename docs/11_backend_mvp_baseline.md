# 步语 BuYu — 后端 MVP 基线

**版本：** 0.1  
**阶段：** MVP（P0）  
**最后更新：** 2026-03-25

---

## 1. 文档用途

本文档用于描述“当前已经落到仓库里的后端 MVP 基线”，重点回答三个问题：

1. 现在后端到底已经做到了什么程度。
2. 基线数据库、命令接口、生成链路是什么形状。
3. 下一位开发者接手时，应该以什么为准继续做前端或继续扩展后端。

它不是需求文档，也不是设计推演文档，而是对当前已实现状态的基线快照。

---

## 2. 基线定位

| 项目 | 值 |
|------|------|
| 本地分支 | `buyu-backend-mvp-baseline` |
| 基线提交 | `ef4610ec53bc9dae441f8f0b7595c1dc1cebdd79` |
| 提交标题 | `feat: 建立 BuYu 后端 MVP 基线` |
| 代码范围 | 仅 `src-tauri` 后端基线代码，不含前端页面和未整理文档 |

补充说明：上表描述的是“后端 MVP 基线提交”本身。当前仓库 HEAD 已在这份基线之上继续补了前端 transport、工作台页面、流式渲染和相关测试；本文其余章节仍以后端事实为主，必要时会注明“仓库当前 HEAD”的补充状态。

---

## 3. 基线范围

### 3.1 已纳入的前序能力

以下能力不是本轮才出现，但已经被视为当前后端基线的一部分：

- 渠道管理：渠道 CRUD、连通性测试。
- 模型管理：模型 CRUD、远程模型拉取。
- AI 适配：OpenAI-compatible provider 的统一接入。

### 3.2 本轮补齐的后端能力

- Agent 管理。
- 会话管理。
- 消息楼层与消息版本系统。
- `send_message` 发送消息。
- `reroll` 重生成。
- `cancel_generation` 取消生成。
- `set_active_version` 版本切换。
- `delete_version` 删除版本。
- `get_version_content` 按需加载非 active 版本正文。
- 启动清理、空内容回滚、并发生成限制。

### 3.3 当前不在基线内的内容

- 前端 transport、页面与组件不属于“后端基线提交”本身；但仓库当前 HEAD 已补齐对应接入原型。
- Tauri 窗口级 E2E 测试。
- API Key 上 OS Keychain。
- SSRF 风险治理与安全加固。

---

## 4. 数据库基线

### 4.1 迁移规范

迁移命名已统一改为四位序号风格：

```text
src-tauri/migrations/
├── 0000_initial_schema.sql
├── 0001_xxx.sql
└── ...
```

当前基线采用“重写并重建”策略，不再沿用时间戳迁移名。MVP 当前完整 schema 直接收敛在 `0000_initial_schema.sql`。

### 4.2 核心表

| 表名 | 职责 |
|------|------|
| `api_channels` | 渠道配置 |
| `api_channel_models` | 渠道下模型 |
| `agents` | Agent 定义与 system_prompt |
| `conversations` | 会话与绑定关系 |
| `message_nodes` | 消息楼层 |
| `message_versions` | 版本元数据与状态机 |
| `message_contents` | 正文 chunk 分块存储 |

### 4.3 关键设计

1. 会话在 MVP 阶段直接内嵌 `agent_id`、`channel_id`、`channel_model_id`，不引入中间表。
2. `message_nodes` 负责“楼层位置”，`message_versions` 负责“版本状态”，`message_contents` 负责“正文内容”。
3. `list_messages` 只加载 active version 的正文，非 active version 只返回元数据。
4. `message_contents` 支持流式追加写入，避免反复 UPDATE 大文本。
5. 删除渠道、模型、Agent 时保留历史会话，统一用 `SET NULL`；删除会话时级联删除消息数据。

---

## 5. 后端模块基线

### 5.1 模型层

- `src/models/channel.rs`
- `src/models/model.rs`
- `src/models/agent.rs`
- `src/models/conversation.rs`
- `src/models/message.rs`

这些文件定义了资源结构、创建/更新入参、返回对象和生成事件结构。

### 5.2 Repo 层

- `src/repo/channel_repo.rs`
- `src/repo/model_repo.rs`
- `src/repo/agent_repo.rs`
- `src/repo/conversation_repo.rs`
- `src/repo/message_repo.rs`

Repo 层负责数据库读写、事务内聚合写入、chunk 拼接和上下文查询。

### 5.3 Service 层

- `src/services/channel_service.rs`
- `src/services/model_service.rs`
- `src/services/agent_service.rs`
- `src/services/conversation_service.rs`
- `src/services/message_service.rs`
- `src/services/generation_engine.rs`

Service 层负责业务校验、错误码转换、状态流转和 AI 生成调度。

### 5.4 Command 层

- `src/commands/channels.rs`
- `src/commands/models.rs`
- `src/commands/agents.rs`
- `src/commands/conversations.rs`
- `src/commands/messages.rs`

这些命令已经在 `lib.rs` 中注册，构成当前 Tauri 后端接口面。

---

## 6. AI 与生成链路基线

### 6.1 技术栈

当前 AI 接入统一使用：

- `aisdk`
- `aisdk-macros`

不再额外实现第二套聊天 HTTP 客户端。已有 `reqwest` 仅作为适配层内部能力使用。

### 6.2 send_message

`send_message` 支持两种模式：

| 模式 | 行为 |
|------|------|
| 正常发送 | 落库 user version + assistant generating version，并启动后台生成 |
| `dry_run=true` | 只返回 prompt 组装结果，不落库、不发请求 |

### 6.3 流式写入策略

流式生成时，正文按以下规则刷盘到 `message_contents`：

- 达到 `2048 bytes`
- 或达到 `2 秒`

满足任一条件即 flush 一次。

补充说明：

1. `GenerationEvent::Chunk` 会在后端收到 delta 后先发往前端，数据库 flush 仅用于持久化，不作为前端首字显示前置条件。
2. 仓库当前 HEAD 的前端工作台已经取消按帧缓冲，chunk 到达后优先直接更新当前版本；只有在“事件先到、本地节点后到”的竞态下，才会短暂暂存早到 chunk。

### 6.4 状态机

`message_versions.status` 当前支持：

- `generating`
- `committed`
- `failed`
- `cancelled`

启动时会自动把遗留的 `generating` 清理成 `failed`。

### 6.5 取消与并发

- 使用 `CancellationToken` 跟踪生成中的 version。
- 使用 `Semaphore(5)` 限制并发生成数。
- `cancel_generation` 为幂等操作，重复取消不会报错。

### 6.6 空内容回滚

如果 assistant 最终生成空内容：

1. 若该 node 只有当前这一个 version，则删除整个 node。
2. 若 node 下还有其它版本，则删除空 version，并把 active 指针切回有效版本。
3. 同时发送 `empty_rollback` 事件，让前端同步视图。

补充说明：对于 OpenAI-compatible 流式返回，provider 终态消息可能携带空文本，但前面的 delta 已经正常产生正文。仓库当前 HEAD 已修正这类兼容性问题，只有在 `message_contents` 中从未落下任何正文时，才会执行空内容回滚，不再因为终态空文本误删 assistant 节点。

---

## 7. 消息系统基线

### 7.1 list_messages

返回：

- 所有 node
- 每个 node 的全部 version 元数据
- 仅 active version 携带正文

非 active version 的正文必须通过 `get_version_content` 按需加载。

### 7.2 reroll

| 场景 | 行为 |
|------|------|
| assistant reroll | 在原 node 下新建 generating version |
| user reroll | 仅允许最后一个 user node，复制 user version，再新建 assistant node/version |

### 7.3 版本切换

`set_active_version` 立即写库，不做 debounce。若用户在生成过程中切换 active version，生成完成后不会强行把 active 指针切回旧版本。

### 7.4 上下文构建

上下文严格按 `order_key` 读取 active version，并实时读取 Agent 的 `system_prompt`。当前不做 prompt 快照缓存。

---

## 8. 测试基线

### 8.1 已有测试

当前已覆盖：

- 渠道 repo / service / command / adapter 测试
- 模型 repo / service / command 测试
- Agent repo / command 测试
- Conversation repo / command 测试
- Message repo / command 测试

### 8.2 已通过的验证命令

```bash
cd src-tauri
CARGO_BUILD_JOBS=1 cargo test -j 1
CARGO_BUILD_JOBS=1 cargo clippy -- -D warnings
```

说明：上面是后端基线提交阶段的通过命令。当前 Windows 开发环境仍建议串行编译，避免 pagefile / `mmap` 相关问题影响验证稳定性。

### 8.3 仓库当前 HEAD 的附加验证

在后续前端接入和流式修复基础上，当前仓库 HEAD 还额外通过了以下验证：

```bash
cd src-tauri
cargo test --test cmd_messages_test
cargo test --test repo_messages_test
cargo clippy -- -D warnings

cd ..
pnpm check
pnpm test
```

这些命令覆盖了最近的流式空消息误回滚修复、前端 transport 类型约束以及工作台状态层回归。

### 8.4 桌面壳 feature

为了让后端验证不被桌面壳编译链拖住，桌面壳二进制目标当前挂在 `desktop-shell` feature 下。

如果需要直接运行桌面壳：

```bash
cd src-tauri
cargo run --features desktop-shell
```

---

## 9. 当前开发建议

下一阶段应以这份基线为后端事实来源，优先推进以下工作：

1. 继续前后端联调，清理流式生成、取消、Reroll 与会话刷新之间的剩余竞态。
2. 补窗口级联调测试和更细粒度的前端状态层测试，把“工作台可用”收口成“回归可控”。
3. 回到后端技术债，补安全项、历史快照一致性和消息大小硬约束。
4. 再根据联调反馈，决定是否补 service 层更细粒度单元测试。
