# 步语 BuYu — 数据库设计

**版本：** 0.5
**阶段：** 当前实现
**数据库：** SQLite（WAL 模式，通过 `sqlx` 访问）
**最后更新：** 2026-04-08

本文描述当前代码已经生效的数据库结构。真正的 schema 事实来源是：

- `src-tauri/migrations/0000_initial_schema.sql`
- `src-tauri/migrations/0001_add_channel_thinking_tags.sql`
- `src-tauri/migrations/0002_add_message_version_timing.sql`
- `src-tauri/migrations/0003_add_provider_keys_and_model_sampling.sql`
- `src-tauri/migrations/0004_add_conversation_enabled_tools.sql`
- `src-tauri/migrations/0005_add_message_version_error_details.sql`

## 1. 当前表总览

| 表名 | 用途 |
|------|------|
| `api_channels` | AI 渠道配置 |
| `api_channel_models` | 某个渠道下可选模型 |
| `agents` | Agent 定义 |
| `conversations` | 会话与会话级配置 |
| `message_nodes` | 会话中的消息楼层 |
| `message_versions` | 同一楼层的多个版本元数据 |
| `message_contents` | 版本正文、thinking、附件、工具调用等内容块 |

## 2. 当前 schema 关键字段

### 2.1 `api_channels`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `TEXT` | 主键 |
| `name` | `TEXT` | 渠道名称 |
| `channel_type` | `TEXT` | 当前默认 `openai_compatible` |
| `base_url` | `TEXT` | 根地址 |
| `api_key` | `TEXT NULL` | 单个 API Key |
| `api_keys` | `TEXT NULL` | 多个 API Key 的 JSON 字符串 |
| `auth_type` | `TEXT NULL` | 鉴权方式 |
| `models_endpoint` | `TEXT NULL` | 模型列表接口 |
| `chat_endpoint` | `TEXT NULL` | 非流式聊天接口 |
| `stream_endpoint` | `TEXT NULL` | 流式聊天接口 |
| `thinking_tags` | `TEXT NULL` | thinking 标签 JSON 字符串，例如 `["think","reasoning"]` |
| `enabled` | `INTEGER` | 0 / 1 |
| `created_at` | `INTEGER` | Unix 毫秒 |
| `updated_at` | `INTEGER` | Unix 毫秒 |

约束：

- `base_url` 必须以 `http://` 或 `https://` 开头
- `name` 不能为空

### 2.2 `api_channel_models`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `TEXT` | 主键 |
| `channel_id` | `TEXT` | 外键，指向 `api_channels.id` |
| `model_id` | `TEXT` | 实际调用的模型标识 |
| `display_name` | `TEXT NULL` | 展示名称 |
| `context_window` | `INTEGER NULL` | 上下文窗口 |
| `max_output_tokens` | `INTEGER NULL` | 最大输出 token |
| `temperature` | `TEXT NULL` | 默认 temperature，按字符串存储 |
| `top_p` | `TEXT NULL` | 默认 top_p，按字符串存储 |

约束：

- `(channel_id, model_id)` 唯一

### 2.3 `agents`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `TEXT` | 主键 |
| `name` | `TEXT` | 名称 |
| `system_prompt` | `TEXT NULL` | 系统提示词 |
| `avatar_uri` | `TEXT NULL` | 头像 |
| `enabled` | `INTEGER` | 0 / 1 |
| `created_at` | `INTEGER` | Unix 毫秒 |
| `updated_at` | `INTEGER` | Unix 毫秒 |

### 2.4 `conversations`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `TEXT` | 主键 |
| `title` | `TEXT` | 会话标题 |
| `agent_id` | `TEXT NULL` | 绑定 Agent |
| `channel_id` | `TEXT NULL` | 绑定渠道 |
| `channel_model_id` | `TEXT NULL` | 绑定模型 |
| `archived` | `INTEGER` | 是否归档 |
| `pinned` | `INTEGER` | 是否置顶 |
| `enabled_tools` | `TEXT NULL` | 启用工具列表 JSON 字符串，例如 `["fetch"]` |
| `created_at` | `INTEGER` | Unix 毫秒 |
| `updated_at` | `INTEGER` | Unix 毫秒 |

说明：

- `enabled_tools` 为 `NULL` 或空时，表示当前会话未启用任何内置工具
- 当前代码没有单独的“全局工具配置表”，工具启用状态是会话级字段

### 2.5 `message_nodes`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `TEXT` | 主键 |
| `conversation_id` | `TEXT` | 所属会话 |
| `author_agent_id` | `TEXT NULL` | assistant 消息对应的 Agent |
| `role` | `TEXT` | `user` 或 `assistant` |
| `order_key` | `TEXT` | 会话内顺序键 |
| `active_version_id` | `TEXT NULL` | 当前展示中的版本 |
| `created_at` | `INTEGER` | Unix 毫秒 |

说明：

- 一个 `message_node` 表示聊天里的“这一条消息位置”
- 它下面可以有多个 `message_versions`

### 2.6 `message_versions`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `TEXT` | 主键 |
| `node_id` | `TEXT` | 所属楼层 |
| `status` | `TEXT` | `generating / committed / failed / cancelled` |
| `error_code` | `TEXT NULL` | 失败错误码 |
| `error_message` | `TEXT NULL` | 失败错误信息 |
| `error_details` | `TEXT NULL` | 结构化错误详情 JSON 字符串 |
| `model_name` | `TEXT NULL` | 生成时使用的模型名 |
| `prompt_tokens` | `INTEGER NULL` | prompt token 数 |
| `completion_tokens` | `INTEGER NULL` | completion token 数 |
| `finish_reason` | `TEXT NULL` | 结束原因 |
| `received_at` | `INTEGER NULL` | 首字节收到时间 |
| `completed_at` | `INTEGER NULL` | 完成时间 |
| `created_at` | `INTEGER` | 版本创建时间 |

说明：

- `received_at` / `completed_at` 只在生成型版本上有意义
- 编辑消息产生的新 committed version 也会写入这张表，但通常没有 token 和 timing 信息

### 2.7 `message_contents`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `TEXT` | 主键 |
| `version_id` | `TEXT` | 所属版本 |
| `chunk_index` | `INTEGER` | 块序号 |
| `content_type` | `TEXT` | 内容类型 |
| `body` | `TEXT` | 内容体 |
| `created_at` | `INTEGER` | Unix 毫秒 |

## 3. `message_contents` 的真实用途

当前 `message_contents` 不只是纯文本正文，还承担多种内容块：

| `content_type` | 含义 |
|------|------|
| `text/plain` | 正文文本 |
| `text/thinking` | thinking 文本 |
| `image/base64` | 图片附件，`body` 为 `ImageAttachment` JSON |
| `file/base64` | 文件附件，`body` 为 `FileAttachment` JSON |
| `application/vnd.buyu.tool-call+json` | 工具调用记录 |
| `application/vnd.buyu.tool-result+json` | 工具结果记录 |

这意味着：

- 一个版本的“完整内容”实际上是多种块拼起来的
- `list_messages` 会把 active version 的文本、thinking、图片、文件、工具调用、工具结果都聚合出来
- `get_version_content` 当前只返回 `text/plain` 正文，不返回 thinking 和附件

## 4. 当前索引

| 索引 | 作用 |
|------|------|
| `idx_api_channels_created_at` | 渠道列表排序 |
| `idx_api_channel_models_channel_id` | 按渠道查模型 |
| `idx_agents_created_at` | Agent 列表排序 |
| `idx_conversations_list` | 会话列表排序 |
| `idx_conversations_agent_id` | Agent 相关查询 |
| `idx_conversations_channel_id` | 渠道相关查询 |
| `idx_message_nodes_conversation_order` | 按会话顺序读消息 |
| `idx_message_versions_node_id` | 读取楼层下版本 |
| `idx_message_versions_status` | 启动时清理 generating |
| `idx_message_contents_version` | 按版本顺序拼接内容块 |

## 5. 外键与删除策略

| 外键 | 删除策略 |
|------|------|
| `api_channel_models.channel_id -> api_channels.id` | `ON DELETE CASCADE` |
| `conversations.agent_id -> agents.id` | `ON DELETE SET NULL` |
| `conversations.channel_id -> api_channels.id` | `ON DELETE SET NULL` |
| `conversations.channel_model_id -> api_channel_models.id` | `ON DELETE SET NULL` |
| `message_nodes.conversation_id -> conversations.id` | `ON DELETE CASCADE` |
| `message_nodes.author_agent_id -> agents.id` | `ON DELETE SET NULL` |
| `message_nodes.active_version_id -> message_versions.id` | `ON DELETE SET NULL` |
| `message_versions.node_id -> message_nodes.id` | `ON DELETE CASCADE` |
| `message_contents.version_id -> message_versions.id` | `ON DELETE CASCADE` |

## 6. 当前运行时数据库行为

### 6.1 启动初始化

`AppState::initialize_*()` 会：

1. 建立 SQLite 连接池
2. 执行全部迁移
3. 执行下面这条修复语句

```sql
UPDATE message_versions
SET status = 'failed'
WHERE status = 'generating';
```

含义：

- 如果上次程序异常退出，残留的 `generating` 版本会在本次启动时改为 `failed`

### 6.2 消息写入规则

当前代码里的真实写入方式：

| 场景 | 写入方式 |
|------|------|
| 用户发送文本 | `message_contents` 写 `text/plain` |
| 用户发送图片 | 追加 `image/base64` |
| 用户发送文件 | 追加 `file/base64` |
| 用户回填工具结果 | 追加 `application/vnd.buyu.tool-result+json` |
| AI 流式生成正文 | 按 buffer flush 追加 `text/plain` |
| AI 流式生成 thinking | 按 buffer flush 追加 `text/thinking` |
| AI 触发工具调用 | 追加 `application/vnd.buyu.tool-call+json` |
| 工具执行结果回填 | 追加 `application/vnd.buyu.tool-result+json` |

### 6.3 flush 策略

流式生成时：

- 文本与 thinking 分开缓存
- 满足 `2048 bytes` 或 `2 秒` 就刷盘

## 7. 当前查询事实

### 7.1 `list_messages`

当前返回策略：

- 返回某个会话下的全部 node
- 每个 node 带上全部 version 元数据
- 只有 active version 会内嵌：
  - `content`
  - `thinking_content`
  - `images`
  - `files`
  - `tool_calls`
  - `tool_results`
- 非 active version 这些内容字段为空或空数组

### 7.2 `get_version_content`

当前实现只做：

```sql
SELECT body
FROM message_contents
WHERE version_id = ?
  AND content_type = 'text/plain'
ORDER BY chunk_index ASC;
```

因此这个接口：

- 只能取完整正文
- 不包含 thinking
- 不包含图片 / 文件 / 工具调用 / 工具结果

### 7.3 prompt 构建

构建 AI prompt 时，后端会读取 active version 的内容块，并按类型组装：

- 正文 -> `content`
- 图片 -> `images`
- 文件 -> `files`
- 工具调用 -> `tool_calls`
- 工具结果 -> `tool_results`
- `text/thinking` 不进入最终 prompt 正文

## 8. 当前迁移历史

```text
0000_initial_schema.sql
  建立 7 张基础表

0001_add_channel_thinking_tags.sql
  api_channels.thinking_tags

0002_add_message_version_timing.sql
  message_versions.received_at
  message_versions.completed_at

0003_add_provider_keys_and_model_sampling.sql
  api_channels.api_keys
  api_channel_models.temperature
  api_channel_models.top_p

0004_add_conversation_enabled_tools.sql
  conversations.enabled_tools

0005_add_message_version_error_details.sql
  message_versions.error_code
  message_versions.error_message
  message_versions.error_details
```

## 9. 文档边界

以下内容不要再写成数据库既成事实：

- Keychain / 安全凭据表
- 独立日志表
- 云端同步表
- MCP 外部服务器注册表

这些目前都不在 schema 中。
