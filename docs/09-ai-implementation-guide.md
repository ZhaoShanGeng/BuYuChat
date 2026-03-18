# AI 实施指南

这份文档的目标不是介绍架构，而是让另一个 AI 可以按顺序、按边界、按命名准确生成代码。

---

## 先读哪些文档

必须按这个顺序读：

1. [03-directory-structure.md](./03-directory-structure.md)
2. [04-database-schema.md](./04-database-schema.md)
3. [05-modules/types.md](./05-modules/types.md)
4. [05-modules/errors.md](./05-modules/errors.md)
5. [05-modules/conversation-store.md](./05-modules/conversation-store.md)
6. [05-modules/provider-abstraction.md](./05-modules/provider-abstraction.md)
7. [05-modules/assistant-prompt.md](./05-modules/assistant-prompt.md)
8. [05-modules/chat-service.md](./05-modules/chat-service.md)
9. [05-modules/rag-engine.md](./05-modules/rag-engine.md)
10. [05-modules/tool-mcp.md](./05-modules/tool-mcp.md)
11. [05-modules/ipc-commands.md](./05-modules/ipc-commands.md)

---

## 范围锁定

生成时必须遵守：

- 只做线性消息版本，不做 Fork / branch
- 不做收藏、临时对话、Prompt 历史、全局 Prompt 库、记忆系统
- 不做 Artifacts、WebDAV、STT、统计面板
- 工具系统先做内置工具，MCP 可以后补
- `send_message` 返回前必须已经插入 assistant 占位消息
- 表字段名、命令名、事件名必须与文档一致

---

## 推荐生成顺序

### 第 1 步：底层骨架

- 建 `src-tauri` 基础工程
- 写 `error.rs`
- 写 `types.rs`
- 写 `migrations/001_init.sql`
- 写 `db/mod.rs`

### 第 2 步：数据库访问层

- `db/models.rs`
- `db/conversation.rs`
- `db/message.rs`
- 其他基础表的 CRUD

完成标准：

- 能创建对话
- 能插入和读取消息
- 能切换消息版本

### 第 3 步：Provider 层

- `providers/mod.rs`
- `providers/openai.rs`
- 再补 `claude.rs` / `gemini.rs` / `ollama.rs`

完成标准：

- 至少一个 Provider 可以正常非流式和流式返回

### 第 4 步：核心服务层

- `services/prompt.rs`
- `services/assistant.rs`
- `services/param.rs`
- `services/versioning.rs`
- `services/chat.rs`

完成标准：

- 普通对话可跑通
- 重新生成可跑通
- 编辑用户消息可跑通

### 第 5 步：RAG 与工具

- `services/rag/*`
- `services/tool/mod.rs`
- `services/tool/builtin/*`

完成标准：

- 可以导入文档并检索
- 可以调用至少一个工具

### 第 6 步：命令层

- 按 [ipc-commands.md](./05-modules/ipc-commands.md) 实现
- `lib.rs` 中统一注册

### 第 7 步：前端

- 先做 `api/` 和 `stores/`
- 再做聊天页、设置页、知识库页
- 最后补增强页面

---

## 每一步都要自检

每生成完一层，都检查：

1. 是否引入了文档里没有的字段或表
2. 是否命令名和文档不一致
3. 是否把数据库逻辑写进了 `services/`
4. 是否把业务逻辑写进了 `commands/`
5. 是否提前实现了不在当前范围里的功能

---

## 最终验收清单

- 能配置 Provider 并测试连接
- 能发起流式对话并停止生成
- 能编辑历史用户消息并重新生成
- 能切换 assistant 版本
- 能上传文档并进行带引用回答
- 能调用基础工具
- 能导出和导入本地数据

如果以上都满足，再做多模型对比、自定义渠道、TTS、翻译等增强项。
