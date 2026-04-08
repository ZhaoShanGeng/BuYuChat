# 步语 BuYu — API 设计

**版本：** 0.1
**阶段：** 当前实现
**最后更新：** 2026-04-08

本文描述当前接口层的设计方式、资源划分和关键约束。

## 1. 设计目标

当前 API 设计目标：

1. 让前端可以按资源访问后端能力。
2. 让命令层和业务层保持清晰分层。
3. 让流式生成和普通 CRUD 使用统一的前端调用入口。
4. 让错误、事件、消息版本等复杂行为有稳定数据结构。

## 2. 当前接口形态

BuYu 当前不是 HTTP 服务架构，而是：

- 同进程 Tauri IPC `invoke()`
- 配合 `Channel<GenerationEvent>` 做流式事件推送

文档里继续使用 REST 风格路径，只是为了表达资源语义，不代表当前运行时真的暴露 HTTP 端口。

## 3. 资源划分

当前接口按下面几组资源划分：

- `channels`
- `models`
- `agents`
- `conversations`
- `messages`
- `generation`
- `tools`

对应后端命令入口定义在：

- `src-tauri/src/commands/`
- `src-tauri/src/lib.rs`

## 4. 资源设计原则

### 4.1 渠道、模型、Agent、会话

这几类资源采用标准 CRUD 思路：

- list
- get
- create
- update
- delete

其中：

- 模型资源依附于渠道
- 会话资源绑定 Agent、渠道和模型
- 工具启用状态挂在会话上，而不是单独做全局工具配置表

### 4.2 消息资源

消息不是“单表单记录”模型，而是三层结构：

1. `message_nodes`
2. `message_versions`
3. `message_contents`

因此接口设计也按这个结构展开：

- `list_messages` 返回 node 列表
- 每个 node 带全部版本元数据
- active version 直接附带聚合后的内容
- 非 active version 按需通过 `get_version_content` 取正文

### 4.3 生成控制

生成相关接口单独区分出来，因为它们不是普通 CRUD：

- `send_message`
- `reroll`
- `edit_message`
- `cancel_generation`

这些接口共同特征：

- 会创建或修改消息版本状态
- 可能触发后台异步生成任务
- 可能伴随流式事件

## 5. 命令命名规则

当前命令命名规则：

- 统一使用 `snake_case`
- 命令名和 Rust handler 函数名保持一致
- 资源操作按“动词 + 资源”命名

例如：

- `list_channels`
- `create_conversation`
- `set_active_version`
- `list_builtin_tools`

## 6. 数据形态设计

当前接口层存在三层数据形态：

1. Rust / IPC 原始字段：`snake_case`
2. 前端 transport 转换后字段：`camelCase`
3. 页面层使用的更高阶状态对象

设计原因：

- 后端模型与数据库字段保持一致更直接
- 前端页面使用 `camelCase` 更自然
- transport 层统一承担命名转换和轻量解码

## 7. 更新语义设计

### 7.1 普通 create / update

普通资源更新使用结构化输入对象，例如：

- `CreateChannelInput`
- `UpdateChannelInput`
- `CreateAgentInput`
- `UpdateConversationInput`

### 7.2 会话 patch 的 `*_set`

`update_conversation` 当前采用显式 `*_set` 语义。

原因：

- 需要区分“字段没传”
- 需要区分“显式清空为 null”
- 需要区分“更新为具体值”

所以会话更新中存在：

- `agent_id_set`
- `channel_id_set`
- `channel_model_id_set`
- `enabled_tools_set`

这是当前接口里最重要的 patch 约束之一。

## 8. 流式事件设计

当前流式事件类型：

- `chunk`
- `completed`
- `failed`
- `cancelled`
- `empty_rollback`
- `tool_call_start`
- `tool_result`

设计原则：

- 所有事件都必须带定位字段
- 至少包含 `conversation_id`、`node_id`、`version_id`
- 前端按这些 ID 把事件路由到正确的消息版本

其中：

- `chunk` 承载正文增量、thinking 增量和工具调用增量
- `completed` 承载 token 统计和结束原因
- `failed` 承载错误码、错误消息和结构化错误详情
- `empty_rollback` 承载空响应回滚结果

## 9. 错误模型设计

当前错误模型统一为 `AppError`。

接口层返回给前端的错误结构包括：

- `error_code`
- `message`
- `details`

设计目标：

- 前端可以按 `error_code` 做本地化提示
- 排障时仍然能看到 `details`
- 渠道调用失败时可保留请求 / 响应上下文

## 10. 工具接口设计

当前工具能力分两层：

1. 工具元信息查询
2. 生成链路中的工具调用结果

当前显式开放给前端的工具接口只有：

- `list_builtin_tools`

设计原因：

- 前端只需要知道有哪些内置工具可选
- 具体工具执行发生在生成引擎内，不通过单独的通用执行接口暴露给前端

## 11. 文档关系

当前接口相关文档分工如下：

- `04_api_openapi.yaml`
  - 放完整契约和 schema
- `05_api_reference.md`
  - 放调用说明、字段解释、错误码和示例
- `13_api_design.md`
  - 放接口层为什么这样设计、资源如何划分、有哪些关键约束

这三份文档一起构成当前 API 文档集。
