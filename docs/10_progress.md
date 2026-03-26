# 步语 BuYu — 进度追踪

**最后更新：** 2026-03-25

---

## 总体进度

| 阶段 | 状态 | 说明 |
|------|------|------|
| 需求分析 | ✅ 完成 | SRS v0.2 |
| 数据库设计 | ✅ 完成 | DDL v0.3（含 message_contents 分块） |
| API 设计 | ✅ 完成 | OpenAPI v0.3 + API 参考 |
| 架构设计 | ✅ 完成 | 分层架构 + 生成流水线 |
| 代码规范 | ✅ 完成 | 命名、复杂度、TDD |
| 设计评审 | ✅ 完成 | v0.3，所有问题已有处理方案 |
| **实现** | 🔨 进行中 | 后端 MVP 基线已完成，前端 MVP 主路径已接通，当前进入联调与体验优化阶段 |

---

## 当前里程碑

| 里程碑 | 状态 | 说明 |
|------|------|------|
| 后端 MVP 基线 | ✅ 完成 | 已纳入渠道、模型、Agent、会话、消息、生成、Reroll 全链路 |
| 本地提交归档 | ✅ 完成 | 分支 `buyu-backend-mvp-baseline`，提交 `ef4610ec53bc9dae441f8f0b7595c1dc1cebdd79` |
| 后端验证 | ✅ 完成 | 基线提交阶段已通过 `cargo test` 与 `cargo clippy -- -D warnings` |
| 前端 MVP 接入 | 🔨 进行中 | transport、工作台、流式渲染、Reroll、取消按钮已接通，当前继续做联调与体验优化 |

---

## MVP 功能进度

### 功能1：渠道管理

| 模块 | 任务 | 状态 | 备注 |
|------|------|------|------|
| 后端 | channel_repo CRUD | ✅ | `sqlx + SqlitePool`，含级联删除/SET NULL |
| 后端 | channel_service 逻辑 | ✅ | UUID v7、默认值、校验、连通性测试 |
| 后端 | Tauri commands | ✅ | `State<AppState> + async fn`，含 `test_channel` |
| 后端 | 测试 | ✅ | repo/service/command 全覆盖；command 基于真实 `AppState` |
| 前端 | transport 层 | ✅ | `src/lib/transport/channels.ts` |
| 前端 | 渠道列表页 | ✅ | `ChannelListPanel` |
| 前端 | 渠道编辑表单 | ✅ | `ChannelFormPanel`，创建/编辑/启用开关 |
| 前端 | 连通性测试按钮 | ✅ | 调用 `test_channel` |

### 功能2：模型管理

| 模块 | 任务 | 状态 | 备注 |
|------|------|------|------|
| 后端 | model_repo CRUD | ✅ | `sqlx + SqlitePool`，含同渠道唯一约束与 `SET NULL` 副作用 |
| 后端 | model_service | ✅ | CRUD + 远程模型拉取，统一委托 AI adapter |
| 后端 | Tauri commands | ✅ | `list/create/update/delete/fetch_remote_models` |
| 后端 | 测试 | ✅ | repo/service/command/adapter 全覆盖 |
| 前端 | transport 层 | ✅ | `src/lib/transport/models.ts` |
| 前端 | 模型列表（渠道详情页内） | ✅ | `ModelSettingsPanel` 已接入会话工作台 |
| 前端 | 远程拉取模型 | ✅ | 支持前端触发远程拉取并刷新模型列表 |

### 功能3：Agent 管理

| 模块 | 任务 | 状态 | 备注 |
|------|------|------|------|
| 后端 | agent_repo CRUD | ✅ | 含启用状态、时间排序、删除后会话 `SET NULL` |
| 后端 | agent_service | ✅ | 统一校验名称、系统提示词与删除副作用 |
| 后端 | Tauri commands | ✅ | `list/get/create/update/delete_agent` |
| 后端 | 测试 | ✅ | repo + command 集成测试已补齐 |
| 前端 | transport 层 | ✅ | `src/lib/transport/agents.ts` |
| 前端 | Agent 列表页 | ✅ | `AgentSettingsPanel` |
| 前端 | Agent 编辑表单 | ✅ | 支持创建、编辑、启用状态切换 |

### 功能4：会话管理

| 模块 | 任务 | 状态 | 备注 |
|------|------|------|------|
| 后端 | conversation_repo CRUD | ✅ | 含归档、置顶、排序与绑定字段读写 |
| 后端 | conversation_service | ✅ | 显式校验 Agent / Channel / Model 引用存在且可用 |
| 后端 | Tauri commands | ✅ | `list/get/create/update/delete_conversation` |
| 后端 | 测试 | ✅ | repo + command 集成测试已补齐 |
| 前端 | transport 层 | ✅ | `src/lib/transport/conversations.ts` |
| 前端 | 会话侧边栏 | ✅ | `ConversationSidebar` 已接入主工作台 |
| 前端 | 会话设置（绑定 Agent/模型） | ✅ | `ConversationSettingsPanel` 已接通绑定保存 |
| 前端 | 归档/置顶/重命名 | ✅ | 已支持置顶、归档、标题编辑 |

### 功能5：基本对话

| 模块 | 任务 | 状态 | 备注 |
|------|------|------|------|
| 后端 | message_repo（含 contents） | ✅ | 支持 node / version / content chunk、上下文构建与按需加载 |
| 后端 | message_service | ✅ | `list_messages`、`get_version_content`、`set_active_version`、`delete_version` |
| 后端 | generation_engine | ✅ | `aisdk + aisdk-macros` 接入、并发限制、取消令牌、终态写库 |
| 后端 | send_message command | ✅ | 支持 `stream` 与 `dry_run` |
| 后端 | cancel_generation | ✅ | 幂等取消，生成中版本可安全终止 |
| 后端 | 启动清理 | ✅ | 启动自动执行 `generating -> failed` 清理 |
| 后端 | 测试 | ✅ | repo + command 集成测试已补齐 |
| 前端 | transport 层 + Channel 事件 | ✅ | `messages.ts + message-codecs.ts` 已接通 |
| 前端 | 聊天视图 | ✅ | `WorkspaceShell + ChatPanel + MessageTimeline` |
| 前端 | 消息气泡（流式渲染） | ✅ | 流式 chunk 直接投影到当前版本内容 |
| 前端 | 输入框（发送/恢复） | ✅ | 支持发送、dry run、错误恢复 |
| 前端 | 取消按钮 | ✅ | 已接入 `cancel_generation` |

### 功能6：Reroll

| 模块 | 任务 | 状态 | 备注 |
|------|------|------|------|
| 后端 | reroll command | ✅ | 支持 assistant reroll 与末尾 user reroll |
| 后端 | 空内容回滚逻辑 | ✅ | 空 assistant 内容自动删版本或删 node |
| 后端 | 测试 | ✅ | command + repo 场景已覆盖 |
| 前端 | 版本切换器 `< [1] 2 3 >` | ✅ | 版本切换与 active version 同步 |
| 前端 | Reroll 按钮 | ✅ | 支持 assistant reroll 与 user reroll |
| 前端 | 版本内容按需加载 | ✅ | 非 active 版本正文按需请求 |

---

## 基础设施进度

| 任务 | 状态 | 备注 |
|------|------|------|
| SQLite 初始化 + 迁移 | ✅ | 基线迁移已重写为 `0000_initial_schema.sql` |
| AppState 定义 | ✅ | `db + http_client + cancellation_tokens + generation_semaphore` |
| AppError 统一错误处理 | ✅ | `error_code + message` 契约 |
| UUID v7 生成工具 | ✅ | 渠道资源 ID 已接入 |
| AISDK adapter | ✅ | OpenAI-compatible provider 已接入 |
| order_key 生成工具 | ✅ | `timestamp_ms + position_tag + random_suffix` |
| 启动残留清理 | ✅ | 应用启动自动清理 `generating` 状态 |
| 生成并发限制 | ✅ | `Semaphore(5)` |
| CI 流水线 | 🔲 | GitHub Actions |
| 前端路由（P1） | 🔲 | MVP 单页 |

---

## 已完成验证

| 验证项 | 状态 | 备注 |
|------|------|------|
| Rust 后端测试 | ✅ | `cd src-tauri && cargo test` |
| Rust 代码规范检查 | ✅ | `cd src-tauri && cargo clippy -- -D warnings` |
| 桌面壳环境恢复 | ✅ | 已移除二进制 feature 门槛，直接使用常规 `cargo run` / `pnpm tauri dev` |
| 流式回归验证 | ✅ | `cargo test --test cmd_messages_test` 与 `cargo test --test repo_messages_test` 已覆盖空消息误回滚场景 |
| 前端类型与组件测试 | ✅ | `pnpm check` 与 `pnpm test` 已通过 |
| 流式前端直达更新 | ✅ | 已移除前端按帧缓冲，chunk 到达后优先直接渲染到当前版本 |

---

## 下一阶段

1. 继续前后端联调，清理流式生成、Reroll、取消与会话刷新间的剩余竞态。
2. 补更细粒度的前端状态层测试与窗口级联调测试，减少只靠人工回归。
3. 回到后端技术债，补安全项、历史快照一致性与消息大小硬约束。

---

## 状态图例

| 符号 | 含义 |
|------|------|
| 🔲 | 未开始 |
| 🔨 | 进行中 |
| ✅ | 完成 |
| ⏸️ | 暂停/阻塞 |
