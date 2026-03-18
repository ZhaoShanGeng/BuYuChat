# OmniChat 文档索引

OmniChat 是一个基于 **Rust + Tauri v2** 的桌面端 AI 聊天应用。

## 文档结构

| 文档 | 说明 |
|------|------|
| [03-directory-structure.md](./03-directory-structure.md) | 后端/前端目录约束，避免 AI 随意发散目录 |
| [02-architecture.md](./02-architecture.md) | 核心架构、主链路数据流 |
| [04-database-schema.md](./04-database-schema.md) | SQLite 建表 SQL、初始化代码、关键查询模式 |
| [05-modules/](./05-modules/) | 各模块接口规格（实现级别） |
| [06-tech-stack.md](./06-tech-stack.md) | 技术选型与依赖说明 |
| [07-development-plan.md](./07-development-plan.md) | 功能分层与开发阶段计划 |
| [08-design-decisions.md](./08-design-decisions.md) | 关键设计决策与非功能需求 |
| [09-ai-implementation-guide.md](./09-ai-implementation-guide.md) | 交给 AI 逐步生成代码时的执行顺序与约束 |

## 模块规格文档

**实现前必读**：先读 `types.md` 和 `errors.md`，再按功能领域读对应文档。

| 模块 | 文件 | 关键内容 |
|------|------|---------|
| 共用类型 | [types.md](./05-modules/types.md) | `ChatRequest` / `Message` / `StreamEvent` / `ModelParams` 等所有跨模块类型 |
| 错误体系 | [errors.md](./05-modules/errors.md) | `AppError` 枚举，统一跨模块错误处理 |
| Provider 抽象层 | [provider-abstraction.md](./05-modules/provider-abstraction.md) | `LlmProvider` trait、各 Adapter 实现要点（OpenAI/Claude/Gemini/Ollama）、启动初始化 |
| 自定义渠道 | [custom-channel.md](./05-modules/custom-channel.md) | `CustomChannelAdapter` 实现、Handlebars 模板渲染、SSE/NDJSON 流解析 |
| 对话编排 | [chat-service.md](./05-modules/chat-service.md) | `ChatService` 13 步编排流程、上下文裁剪算法、工具调用循环、取消机制 |
| 对话/消息存储 | [conversation-store.md](./05-modules/conversation-store.md) | `db::conversation` / `db::message` 完整 CRUD 接口、事务规范 |
| 消息版本系统 | [message-versioning.md](./05-modules/message-versioning.md) | 重新生成/版本切换/编辑截断的精确 SQL 步骤 |
| 助手与提示词 | [assistant-prompt.md](./05-modules/assistant-prompt.md) | `AssistantService` / `PromptService` / `ParamService` 完整方法签名与约束 |
| RAG 引擎 | [rag-engine.md](./05-modules/rag-engine.md) | 文档解析/分块/向量化/检索/引用标注各组件接口 |
| 工具调用 / MCP | [tool-mcp.md](./05-modules/tool-mcp.md) | `ToolRegistry` / `ToolService` / `McpClient` / `McpTransport` 完整接口 |
| IPC 命令参考 | [ipc-commands.md](./05-modules/ipc-commands.md) | 全部 Tauri 命令（参数/返回值）、前端事件列表、`invoke_handler` 注册代码 |
| 增强模块 | [enhancements.md](./05-modules/enhancements.md) | `MultiChatOrchestrator` / `VoiceEngine` / `TranslateService` / `NamingService` 完整接口 |
