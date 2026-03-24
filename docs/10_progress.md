# 步语 BuYu — 进度追踪

**最后更新：** 2026-03-24

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
| **实现** | 🔨 进行中 | 功能1：渠道管理返工完成，后续进入模型管理 |

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
| 后端 | model_repo CRUD | 🔲 | |
| 后端 | model_service | 🔲 | |
| 后端 | Tauri commands | 🔲 | |
| 后端 | 测试 | 🔲 | |
| 前端 | transport 层 | 🔲 | |
| 前端 | 模型列表（渠道详情页内） | 🔲 | |
| 前端 | 远程拉取模型 | 🔲 | |

### 功能3：Agent 管理

| 模块 | 任务 | 状态 | 备注 |
|------|------|------|------|
| 后端 | agent_repo CRUD | 🔲 | |
| 后端 | agent_service | 🔲 | |
| 后端 | Tauri commands | 🔲 | |
| 后端 | 测试 | 🔲 | |
| 前端 | transport 层 | 🔲 | |
| 前端 | Agent 列表页 | 🔲 | |
| 前端 | Agent 编辑表单 | 🔲 | |

### 功能4：会话管理

| 模块 | 任务 | 状态 | 备注 |
|------|------|------|------|
| 后端 | conversation_repo CRUD | 🔲 | |
| 后端 | conversation_service | 🔲 | |
| 后端 | Tauri commands | 🔲 | |
| 后端 | 测试 | 🔲 | |
| 前端 | transport 层 | 🔲 | |
| 前端 | 会话侧边栏 | 🔲 | |
| 前端 | 会话设置（绑定 Agent/模型） | 🔲 | |
| 前端 | 归档/置顶/重命名 | 🔲 | |

### 功能5：基本对话

| 模块 | 任务 | 状态 | 备注 |
|------|------|------|------|
| 后端 | message_repo（含 contents） | 🔲 | |
| 后端 | message_service | 🔲 | |
| 后端 | generation_engine | 🔲 | aisdk 集成 |
| 后端 | send_message command | 🔲 | stream + dry_run |
| 后端 | cancel_generation | 🔲 | |
| 后端 | 启动清理 | 🔲 | |
| 后端 | 测试 | 🔲 | |
| 前端 | transport 层 + Channel 事件 | 🔲 | |
| 前端 | 聊天视图 | 🔲 | |
| 前端 | 消息气泡（流式渲染） | 🔲 | |
| 前端 | 输入框（发送/恢复） | 🔲 | |
| 前端 | 取消按钮 | 🔲 | |

### 功能6：Reroll

| 模块 | 任务 | 状态 | 备注 |
|------|------|------|------|
| 后端 | reroll command | 🔲 | |
| 后端 | 空内容回滚逻辑 | 🔲 | |
| 后端 | 测试 | 🔲 | |
| 前端 | 版本切换器 `< [1] 2 3 >` | 🔲 | |
| 前端 | Reroll 按钮 | 🔲 | |
| 前端 | 版本内容按需加载 | 🔲 | |

---

## 基础设施进度

| 任务 | 状态 | 备注 |
|------|------|------|
| SQLite 初始化 + 迁移 | ✅ | 含初始 migration SQL |
| AppState 定义 | ✅ | `db + http_client + cancellation_tokens` |
| AppError 统一错误处理 | ✅ | `error_code + message` 契约 |
| UUID v7 生成工具 | ✅ | 渠道资源 ID 已接入 |
| AISDK adapter | ✅ | OpenAI-compatible provider 已接入 |
| order_key 生成工具 | 🔲 | |
| CI 流水线 | 🔲 | GitHub Actions |
| 前端路由（P1） | 🔲 | MVP 单页 |

---

## 状态图例

| 符号 | 含义 |
|------|------|
| 🔲 | 未开始 |
| 🔨 | 进行中 |
| ✅ | 完成 |
| ⏸️ | 暂停/阻塞 |
