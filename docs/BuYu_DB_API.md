# 步语 BuYu — 数据库设计 & API 接口设计

**版本：** 0.1  
**阶段：** MVP（P0）  
**数据库：** SQLite（通过 sqlx 运行时查询）  
**接口层：** Tauri IPC 命令，前端以 REST 风格路由表封装调用

---

# 一、数据库设计

## 1.1 总览

| 表名 | 用途 |
|------|------|
| `api_channels` | AI 服务渠道（服务商接入配置） |
| `api_channel_models` | 渠道下的模型列表 |
| `agents` | Agent 定义（含系统提示词） |
| `conversations` | 会话，MVP 直接内嵌 Agent/渠道/模型绑定 |
| `message_nodes` | 消息楼层（位置实体，无状态） |
| `message_versions` | 消息版本（内容实体，有状态） |

---

## 1.2 完整 DDL

```sql
PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;
PRAGMA synchronous = NORMAL;

-- ============================================================
-- 表1: api_channels
-- 用途: 存储 AI 服务渠道配置，每条记录对应一个服务商接入点
-- ============================================================
CREATE TABLE IF NOT EXISTS api_channels (
    id           TEXT    NOT NULL,                -- UUID v7，时间有序
    name         TEXT    NOT NULL,                -- 用户自定义显示名，不允许为空
    channel_type TEXT    NOT NULL                 -- 服务商类型，当前仅 'openai_compatible'
                         DEFAULT 'openai_compatible',
    base_url     TEXT    NOT NULL,                -- 域名，不含路径，如 https://api.openai.com
    api_key      TEXT,                            -- API 密钥，允许为空（某些本地部署无需认证）
    auth_type    TEXT,                            -- NULL=使用 channel_type 默认值; 'bearer'/'x_api_key'/'none'
    models_endpoint  TEXT,                        -- NULL=使用默认值 /v1/models
    chat_endpoint    TEXT,                        -- NULL=使用默认值 /v1/chat/completions
    stream_endpoint  TEXT,                        -- NULL=使用默认值 /v1/chat/completions
    enabled      INTEGER NOT NULL DEFAULT 1,      -- 1=启用, 0=禁用
    created_at   INTEGER NOT NULL,                -- Unix 毫秒时间戳
    updated_at   INTEGER NOT NULL,                -- Unix 毫秒时间戳

    PRIMARY KEY (id),
    CHECK (enabled IN (0, 1)),
    CHECK (length(name) > 0),
    CHECK (base_url LIKE 'http://%' OR base_url LIKE 'https://%')
);

-- 渠道列表页按 created_at 降序展示
CREATE INDEX IF NOT EXISTS idx_api_channels_created_at
    ON api_channels (created_at DESC);


-- ============================================================
-- 表2: api_channel_models
-- 用途: 渠道下的可用模型列表，model_id 是实际调用标识符
-- ============================================================
CREATE TABLE IF NOT EXISTS api_channel_models (
    id              TEXT    NOT NULL,             -- UUID v7
    channel_id      TEXT    NOT NULL,             -- 所属渠道
    model_id        TEXT    NOT NULL,             -- 实际调用标识，如 'gpt-4o'
    display_name    TEXT,                         -- 用户友好名称，NULL 时前端展示 model_id
    context_window  INTEGER,                      -- 上下文窗口大小（tokens），可选
    max_output_tokens INTEGER,                    -- 最大输出 tokens，可选

    PRIMARY KEY (id),
    FOREIGN KEY (channel_id)
        REFERENCES api_channels (id)
        ON DELETE CASCADE                         -- 删渠道时级联删模型
        DEFERRABLE INITIALLY DEFERRED,
    UNIQUE (channel_id, model_id),               -- 同一渠道下 model_id 不重复
    CHECK (length(model_id) > 0)
);

-- 按渠道查询模型列表（最常用路径）
CREATE INDEX IF NOT EXISTS idx_api_channel_models_channel_id
    ON api_channel_models (channel_id);


-- ============================================================
-- 表3: agents
-- 用途: Agent 定义，包含系统提示词，system_prompt 无默认值
-- ============================================================
CREATE TABLE IF NOT EXISTS agents (
    id            TEXT    NOT NULL,               -- UUID v7
    name          TEXT    NOT NULL,               -- 显示名，不允许为空
    system_prompt TEXT,                           -- 系统提示词，NULL 或空字符串均表示无提示词
    avatar_uri    TEXT,                           -- 头像 URI，MVP 阶段可不实现
    enabled       INTEGER NOT NULL DEFAULT 1,     -- 1=启用, 0=禁用
    created_at    INTEGER NOT NULL,
    updated_at    INTEGER NOT NULL,

    PRIMARY KEY (id),
    CHECK (enabled IN (0, 1)),
    CHECK (length(name) > 0)
);

CREATE INDEX IF NOT EXISTS idx_agents_created_at
    ON agents (created_at DESC);


-- ============================================================
-- 表4: conversations
-- 用途: 会话，MVP 直接内嵌 Agent/渠道/模型绑定
--       P1 引入多 Agent 时拆出 conversation_agents 中间表
-- ============================================================
CREATE TABLE IF NOT EXISTS conversations (
    id               TEXT    NOT NULL,            -- UUID v7
    title            TEXT    NOT NULL DEFAULT '新会话',
    agent_id         TEXT,                        -- 绑定的 Agent，NULL=未配置
    channel_id       TEXT,                        -- 绑定的渠道，NULL=未配置
    channel_model_id TEXT,                        -- 绑定的模型，NULL=未配置
    archived         INTEGER NOT NULL DEFAULT 0,  -- 1=已归档
    pinned           INTEGER NOT NULL DEFAULT 0,  -- 1=已置顶
    created_at       INTEGER NOT NULL,
    updated_at       INTEGER NOT NULL,            -- 有新消息时更新，用于列表排序

    PRIMARY KEY (id),
    FOREIGN KEY (agent_id)
        REFERENCES agents (id)
        ON DELETE SET NULL
        DEFERRABLE INITIALLY DEFERRED,
    FOREIGN KEY (channel_id)
        REFERENCES api_channels (id)
        ON DELETE SET NULL
        DEFERRABLE INITIALLY DEFERRED,
    FOREIGN KEY (channel_model_id)
        REFERENCES api_channel_models (id)
        ON DELETE SET NULL
        DEFERRABLE INITIALLY DEFERRED,
    CHECK (archived IN (0, 1)),
    CHECK (pinned IN (0, 1)),
    CHECK (length(title) > 0)
);

-- 会话列表主查询：置顶优先，再按 updated_at 降序
-- 覆盖索引，避免回表
CREATE INDEX IF NOT EXISTS idx_conversations_list
    ON conversations (archived, pinned DESC, updated_at DESC);

-- 按 agent_id 查找受影响会话（Agent 禁用/删除时使用）
CREATE INDEX IF NOT EXISTS idx_conversations_agent_id
    ON conversations (agent_id)
    WHERE agent_id IS NOT NULL;

-- 按 channel_id 查找受影响会话
CREATE INDEX IF NOT EXISTS idx_conversations_channel_id
    ON conversations (channel_id)
    WHERE channel_id IS NOT NULL;


-- ============================================================
-- 表5: message_nodes
-- 用途: 消息楼层（位置实体）。只管"在哪里"，不管"内容是什么"。
--       状态已移至 message_versions，node 本身无状态字段。
-- ============================================================
CREATE TABLE IF NOT EXISTS message_nodes (
    id                TEXT    NOT NULL,           -- UUID v7
    conversation_id   TEXT    NOT NULL,           -- 所属会话
    author_agent_id   TEXT,                       -- 发出该楼层的 Agent，user 消息为 NULL
    role              TEXT    NOT NULL,           -- 'user' 或 'assistant'
    order_key         TEXT    NOT NULL,           -- 字典序排列，格式见业务规则
    active_version_id TEXT,                       -- 当前展示的版本，初始为 NULL 直到第一个版本创建
    created_at        INTEGER NOT NULL,

    PRIMARY KEY (id),
    FOREIGN KEY (conversation_id)
        REFERENCES conversations (id)
        ON DELETE CASCADE
        DEFERRABLE INITIALLY DEFERRED,
    FOREIGN KEY (author_agent_id)
        REFERENCES agents (id)
        ON DELETE SET NULL
        DEFERRABLE INITIALLY DEFERRED,
    FOREIGN KEY (active_version_id)
        REFERENCES message_versions (id)
        ON DELETE SET NULL
        DEFERRABLE INITIALLY DEFERRED,
    UNIQUE (conversation_id, order_key),         -- order_key 在同一会话内唯一
    CHECK (role IN ('user', 'assistant'))
);

-- 拉取会话消息列表（核心高频查询）
CREATE INDEX IF NOT EXISTS idx_message_nodes_conversation_order
    ON message_nodes (conversation_id, order_key ASC);


-- ============================================================
-- 表6: message_versions
-- 用途: 消息版本（内容实体）。有生命周期状态，是状态机的载体。
--       一个 node 可以有多个 version（reroll、编辑产生）
-- ============================================================
CREATE TABLE IF NOT EXISTS message_versions (
    id                TEXT    NOT NULL,           -- UUID v7
    node_id           TEXT    NOT NULL,           -- 所属楼层
    content           TEXT    NOT NULL DEFAULT '', -- 消息正文，generating 期间可为空字符串
    status            TEXT    NOT NULL DEFAULT 'generating',
                                                  -- 'generating'|'committed'|'failed'|'cancelled'
    model_name        TEXT,                       -- 实际调用的 model_id 字符串，不存外键
                                                  -- 历史记录不受模型/渠道删除影响
    prompt_tokens     INTEGER,                    -- 输入 token 数，完成后填充
    completion_tokens INTEGER,                    -- 输出 token 数，完成后填充
    finish_reason     TEXT,                       -- 'stop'|'length'|'content_filter' 等
    created_at        INTEGER NOT NULL,

    PRIMARY KEY (id),
    FOREIGN KEY (node_id)
        REFERENCES message_nodes (id)
        ON DELETE CASCADE
        DEFERRABLE INITIALLY DEFERRED,
    CHECK (status IN ('generating', 'committed', 'failed', 'cancelled'))
);

-- 按 node_id 查版本列表（版本切换器、reroll 时常用）
CREATE INDEX IF NOT EXISTS idx_message_versions_node_id
    ON message_versions (node_id, created_at ASC);

-- 启动清理查询：找所有 generating 状态的版本
CREATE INDEX IF NOT EXISTS idx_message_versions_status
    ON message_versions (status)
    WHERE status = 'generating';
```

---

## 1.3 索引设计说明

| 索引 | 目的 | 查询场景 |
|------|------|---------|
| `idx_conversations_list` | 覆盖索引，避免回表 | 会话列表页：`WHERE archived=0 ORDER BY pinned DESC, updated_at DESC` |
| `idx_conversations_agent_id` | 部分索引（非 NULL） | Agent 禁用/删除时批量更新受影响会话 |
| `idx_conversations_channel_id` | 部分索引（非 NULL） | 渠道禁用/删除时批量查找 |
| `idx_message_nodes_conversation_order` | 复合索引 | 拉取会话全部消息：`WHERE conversation_id=? ORDER BY order_key` |
| `idx_message_versions_node_id` | 复合索引含时间 | 版本切换器展示所有版本；reroll 后更新 active |
| `idx_message_versions_status` | 部分索引仅 generating | 启动清理：`UPDATE ... WHERE status='generating'`，通常命中行数为 0 |

---

## 1.4 外键约束与数据一致性说明

| 外键 | ON DELETE 策略 | 理由 |
|------|---------------|------|
| `api_channel_models → api_channels` | CASCADE | 渠道删除时，其下模型失去意义，应一并清除 |
| `conversations → agents` | SET NULL | 删 Agent 不应影响会话历史，保留会话但解绑 |
| `conversations → api_channels` | SET NULL | 同上，历史消息仍可查看 |
| `conversations → api_channel_models` | SET NULL | 同上 |
| `message_nodes → conversations` | CASCADE | 会话删除时，其下所有楼层内容一并删除 |
| `message_nodes → agents (author)` | SET NULL | Agent 删除后历史消息作者字段置空，内容保留 |
| `message_nodes → message_versions (active)` | SET NULL | 版本被删除时 active 指针置空，由应用层重新指向 |
| `message_versions → message_nodes` | CASCADE | 楼层删除时版本内容一并删除 |

**DEFERRABLE INITIALLY DEFERRED 的作用：**  
`message_nodes.active_version_id` 和 `message_versions.node_id` 存在循环引用（node 指向 version，version 指向 node）。使用 DEFERRED 约束，允许在同一事务内先插入两张表，事务提交时再检查外键，避免插入顺序问题。

---

## 1.5 启动清理 SQL

```sql
-- 应用每次启动时执行，清理崩溃残留
-- 将所有 generating 状态改为 failed，已收内容保留
UPDATE message_versions
SET status = 'failed'
WHERE status = 'generating';
```

---

# 二、API 接口设计

## 2.1 说明

BuYu 当前使用 Tauri IPC 而非 HTTP。前端通过统一的 Transport 层，将 REST 风格路径翻译为 Tauri `invoke()` 命令。  
本文档按 REST 风格描述语义，并标注实际对应的 Tauri 命令名。  
**无鉴权**（本地单用户桌面应用）。

---

## 2.2 OpenAPI 3.0 规范

```yaml
openapi: "3.0.3"
info:
  title: BuYu Internal API
  version: "0.1.0"
  description: |
    BuYu 桌面客户端内部接口。底层为 Tauri IPC 命令。
    前端 Transport 层将 REST 路径映射为 invoke() 调用。

tags:
  - name: channels
    description: 渠道管理
  - name: models
    description: 模型管理
  - name: agents
    description: Agent 管理
  - name: conversations
    description: 会话管理
  - name: messages
    description: 消息与生成控制

paths:

  # ─────────────────────────────────────────────
  # 渠道管理
  # ─────────────────────────────────────────────

  /channels:
    get:
      operationId: list_channels
      tags: [channels]
      summary: 获取渠道列表
      responses:
        "200":
          description: 渠道列表
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: "#/components/schemas/Channel"

    post:
      operationId: create_channel
      tags: [channels]
      summary: 创建渠道
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: "#/components/schemas/CreateChannelInput"
      responses:
        "201":
          description: 创建成功
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/Channel"
        "400":
          $ref: "#/components/responses/BadRequest"

  /channels/{id}:
    get:
      operationId: get_channel
      tags: [channels]
      summary: 获取单个渠道
      parameters:
        - $ref: "#/components/parameters/Id"
      responses:
        "200":
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/Channel"
          description: 渠道详情
        "404":
          $ref: "#/components/responses/NotFound"

    put:
      operationId: update_channel
      tags: [channels]
      summary: 更新渠道
      parameters:
        - $ref: "#/components/parameters/Id"
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: "#/components/schemas/UpdateChannelInput"
      responses:
        "200":
          description: 更新后的渠道
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/Channel"
        "400":
          $ref: "#/components/responses/BadRequest"
        "404":
          $ref: "#/components/responses/NotFound"

    delete:
      operationId: delete_channel
      tags: [channels]
      summary: 删除渠道（级联删除模型，SET NULL 会话引用）
      parameters:
        - $ref: "#/components/parameters/Id"
      responses:
        "204":
          description: 删除成功
        "404":
          $ref: "#/components/responses/NotFound"

  # ─────────────────────────────────────────────
  # 模型管理
  # ─────────────────────────────────────────────

  /channels/{channelId}/models:
    get:
      operationId: list_models
      tags: [models]
      summary: 获取渠道下的模型列表
      parameters:
        - name: channelId
          in: path
          required: true
          schema:
            type: string
      responses:
        "200":
          description: 模型列表
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: "#/components/schemas/ChannelModel"
        "404":
          $ref: "#/components/responses/NotFound"

    post:
      operationId: create_model
      tags: [models]
      summary: 在渠道下添加模型
      parameters:
        - name: channelId
          in: path
          required: true
          schema:
            type: string
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: "#/components/schemas/CreateModelInput"
      responses:
        "201":
          description: 创建成功
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/ChannelModel"
        "400":
          $ref: "#/components/responses/BadRequest"
        "409":
          $ref: "#/components/responses/Conflict"

  /channels/{channelId}/models/{id}:
    put:
      operationId: update_model
      tags: [models]
      summary: 更新模型信息
      parameters:
        - name: channelId
          in: path
          required: true
          schema:
            type: string
        - $ref: "#/components/parameters/Id"
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: "#/components/schemas/UpdateModelInput"
      responses:
        "200":
          description: 更新后的模型
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/ChannelModel"
        "404":
          $ref: "#/components/responses/NotFound"

    delete:
      operationId: delete_model
      tags: [models]
      summary: 删除模型（SET NULL 会话引用）
      parameters:
        - name: channelId
          in: path
          required: true
          schema:
            type: string
        - $ref: "#/components/parameters/Id"
      responses:
        "204":
          description: 删除成功
        "404":
          $ref: "#/components/responses/NotFound"

  /channels/{channelId}/models/fetch:
    post:
      operationId: fetch_models_from_channel
      tags: [models]
      summary: 从渠道 API 拉取可用模型列表（不写库，仅返回供用户选择）
      parameters:
        - name: channelId
          in: path
          required: true
          schema:
            type: string
      responses:
        "200":
          description: 拉取到的模型 ID 列表
          content:
            application/json:
              schema:
                type: array
                items:
                  type: string
                example: ["gpt-4o", "gpt-4o-mini", "gpt-3.5-turbo"]
        "400":
          $ref: "#/components/responses/BadRequest"
        "404":
          $ref: "#/components/responses/NotFound"

  # ─────────────────────────────────────────────
  # Agent 管理
  # ─────────────────────────────────────────────

  /agents:
    get:
      operationId: list_agents
      tags: [agents]
      summary: 获取 Agent 列表
      parameters:
        - name: includeDisabled
          in: query
          schema:
            type: boolean
            default: false
          description: 是否包含禁用的 Agent
      responses:
        "200":
          description: Agent 列表
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: "#/components/schemas/Agent"

    post:
      operationId: create_agent
      tags: [agents]
      summary: 创建 Agent
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: "#/components/schemas/CreateAgentInput"
      responses:
        "201":
          description: 创建成功
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/Agent"
        "400":
          $ref: "#/components/responses/BadRequest"

  /agents/{id}:
    get:
      operationId: get_agent
      tags: [agents]
      summary: 获取单个 Agent
      parameters:
        - $ref: "#/components/parameters/Id"
      responses:
        "200":
          description: Agent 详情
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/Agent"
        "404":
          $ref: "#/components/responses/NotFound"

    put:
      operationId: update_agent
      tags: [agents]
      summary: 更新 Agent
      parameters:
        - $ref: "#/components/parameters/Id"
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: "#/components/schemas/UpdateAgentInput"
      responses:
        "200":
          description: 更新后的 Agent
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/Agent"
        "400":
          $ref: "#/components/responses/BadRequest"
        "404":
          $ref: "#/components/responses/NotFound"

    delete:
      operationId: delete_agent
      tags: [agents]
      summary: 删除 Agent（SET NULL 会话引用）
      parameters:
        - $ref: "#/components/parameters/Id"
      responses:
        "204":
          description: 删除成功
        "404":
          $ref: "#/components/responses/NotFound"

  # ─────────────────────────────────────────────
  # 会话管理
  # ─────────────────────────────────────────────

  /conversations:
    get:
      operationId: list_conversations
      tags: [conversations]
      summary: 获取会话列表
      parameters:
        - name: archived
          in: query
          schema:
            type: boolean
            default: false
          description: true=归档列表，false=活跃列表
      responses:
        "200":
          description: 会话列表（置顶优先，再按 updated_at 降序）
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: "#/components/schemas/Conversation"

    post:
      operationId: create_conversation
      tags: [conversations]
      summary: 创建会话
      requestBody:
        required: false
        content:
          application/json:
            schema:
              $ref: "#/components/schemas/CreateConversationInput"
      responses:
        "201":
          description: 创建成功（标题默认"新会话"，绑定关系均为 null）
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/Conversation"

  /conversations/{id}:
    get:
      operationId: get_conversation
      tags: [conversations]
      summary: 获取会话详情（含绑定配置）
      parameters:
        - $ref: "#/components/parameters/Id"
      responses:
        "200":
          description: 会话详情
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/Conversation"
        "404":
          $ref: "#/components/responses/NotFound"

    put:
      operationId: update_conversation
      tags: [conversations]
      summary: 更新会话（标题/绑定/归档/置顶）
      parameters:
        - $ref: "#/components/parameters/Id"
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: "#/components/schemas/UpdateConversationInput"
      responses:
        "200":
          description: 更新后的会话
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/Conversation"
        "400":
          $ref: "#/components/responses/BadRequest"
        "404":
          $ref: "#/components/responses/NotFound"

    delete:
      operationId: delete_conversation
      tags: [conversations]
      summary: 删除会话（级联删除所有消息）
      parameters:
        - $ref: "#/components/parameters/Id"
      responses:
        "204":
          description: 删除成功
        "404":
          $ref: "#/components/responses/NotFound"

  # ─────────────────────────────────────────────
  # 消息查询
  # ─────────────────────────────────────────────

  /conversations/{id}/messages:
    get:
      operationId: list_messages
      tags: [messages]
      summary: 获取会话的完整消息列表（含各 node 的所有 versions）
      parameters:
        - $ref: "#/components/parameters/Id"
      responses:
        "200":
          description: |
            消息节点列表，按 order_key 升序。
            每个 node 包含其所有 versions，active_version_id 标记当前展示版本。
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: "#/components/schemas/MessageNode"
        "404":
          $ref: "#/components/responses/NotFound"

  # ─────────────────────────────────────────────
  # AI 生成：发送消息
  # ─────────────────────────────────────────────

  /conversations/{id}/send:
    post:
      operationId: send_message
      tags: [messages]
      summary: |
        发送用户消息并触发 AI 流式生成。
        立即返回创建好的 node/version ID，生成过程通过 Tauri Channel 事件推送。
      parameters:
        - $ref: "#/components/parameters/Id"
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: "#/components/schemas/SendMessageInput"
      responses:
        "200":
          description: 消息节点已创建，生成已在后台启动
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/SendMessageResult"
        "400":
          $ref: "#/components/responses/BadRequest"
        "404":
          $ref: "#/components/responses/NotFound"
        "422":
          description: |
            业务规则不满足，error_code 取值：
            - NO_AGENT: 会话未配置 Agent
            - AGENT_DISABLED: Agent 已禁用
            - NO_CHANNEL: 未配置渠道
            - NO_MODEL: 未配置模型
            - CHANNEL_DISABLED: 渠道已禁用
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/ApiError"

  # ─────────────────────────────────────────────
  # AI 生成：Reroll
  # ─────────────────────────────────────────────

  /conversations/{id}/nodes/{nodeId}/reroll:
    post:
      operationId: reroll
      tags: [messages]
      summary: |
        对指定楼层执行 Reroll。
        assistant node: 新建版本重新生成。
        user node（仅末尾楼层）: 新建 user version（复制当前内容）+ 新建 assistant node 生成。
      parameters:
        - $ref: "#/components/parameters/Id"
        - name: nodeId
          in: path
          required: true
          schema:
            type: string
      responses:
        "200":
          description: Reroll 已触发
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/RerollResult"
        "400":
          $ref: "#/components/responses/BadRequest"
        "404":
          $ref: "#/components/responses/NotFound"
        "422":
          description: |
            - NOT_LAST_USER_NODE: user node 不是最后一个楼层，不允许 reroll
            - NO_AGENT / NO_CHANNEL / NO_MODEL / AGENT_DISABLED / CHANNEL_DISABLED
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/ApiError"

  # ─────────────────────────────────────────────
  # AI 生成：取消
  # ─────────────────────────────────────────────

  /generations/{versionId}/cancel:
    post:
      operationId: cancel_generation
      tags: [messages]
      summary: 取消指定 version 的生成（幂等，version 不存在或已完成时返回 200）
      parameters:
        - name: versionId
          in: path
          required: true
          schema:
            type: string
      responses:
        "200":
          description: 取消成功或已完成（幂等）

  # ─────────────────────────────────────────────
  # 版本切换
  # ─────────────────────────────────────────────

  /conversations/{id}/nodes/{nodeId}/active-version:
    put:
      operationId: set_active_version
      tags: [messages]
      summary: 切换楼层的当前展示版本（立即写库）
      parameters:
        - $ref: "#/components/parameters/Id"
        - name: nodeId
          in: path
          required: true
          schema:
            type: string
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              required: [versionId]
              properties:
                versionId:
                  type: string
      responses:
        "200":
          description: 切换成功
        "404":
          $ref: "#/components/responses/NotFound"
        "400":
          description: versionId 不属于该 node
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/ApiError"

  # ─────────────────────────────────────────────
  # 版本删除
  # ─────────────────────────────────────────────

  /conversations/{id}/nodes/{nodeId}/versions/{versionId}:
    delete:
      operationId: delete_version
      tags: [messages]
      summary: |
        删除指定版本。
        若版本处于 generating 状态，先自动取消生成再删除。
        若为该 node 的最后一个版本，连同 node 一起删除。
        若为 active version，自动切换到同 node 最新的其他版本。
      parameters:
        - $ref: "#/components/parameters/Id"
        - name: nodeId
          in: path
          required: true
          schema:
            type: string
        - name: versionId
          in: path
          required: true
          schema:
            type: string
      responses:
        "200":
          description: 删除结果
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/DeleteVersionResult"
        "404":
          $ref: "#/components/responses/NotFound"

components:

  parameters:
    Id:
      name: id
      in: path
      required: true
      schema:
        type: string
      description: UUID v7

  responses:
    BadRequest:
      description: 入参校验失败
      content:
        application/json:
          schema:
            $ref: "#/components/schemas/ApiError"
    NotFound:
      description: 资源不存在
      content:
        application/json:
          schema:
            $ref: "#/components/schemas/ApiError"
    Conflict:
      description: 唯一约束冲突
      content:
        application/json:
          schema:
            $ref: "#/components/schemas/ApiError"

  schemas:

    ApiError:
      type: object
      required: [error_code, message]
      properties:
        error_code:
          type: string
          description: 机器可读错误码，前端用于 i18n
          example: NO_CHANNEL
        message:
          type: string
          description: 英文调试信息，不直接展示给用户

    Channel:
      type: object
      required: [id, name, channelType, baseUrl, enabled, createdAt, updatedAt]
      properties:
        id:           { type: string }
        name:         { type: string }
        channelType:  { type: string, example: openai_compatible }
        baseUrl:      { type: string, example: "https://api.openai.com" }
        apiKey:       { type: string, nullable: true }
        authType:     { type: string, nullable: true }
        modelsEndpoint:  { type: string, nullable: true }
        chatEndpoint:    { type: string, nullable: true }
        streamEndpoint:  { type: string, nullable: true }
        enabled:      { type: boolean }
        createdAt:    { type: integer, description: "Unix ms" }
        updatedAt:    { type: integer }

    CreateChannelInput:
      type: object
      required: [name, baseUrl]
      properties:
        name:         { type: string, minLength: 1 }
        channelType:  { type: string, default: openai_compatible }
        baseUrl:      { type: string, pattern: "^https?://" }
        apiKey:       { type: string, nullable: true }
        authType:     { type: string, nullable: true }
        modelsEndpoint:  { type: string, nullable: true }
        chatEndpoint:    { type: string, nullable: true }
        streamEndpoint:  { type: string, nullable: true }

    UpdateChannelInput:
      type: object
      properties:
        name:         { type: string, minLength: 1 }
        baseUrl:      { type: string, pattern: "^https?://" }
        apiKey:       { type: string, nullable: true }
        authType:     { type: string, nullable: true }
        modelsEndpoint:  { type: string, nullable: true }
        chatEndpoint:    { type: string, nullable: true }
        streamEndpoint:  { type: string, nullable: true }
        enabled:      { type: boolean }

    ChannelModel:
      type: object
      required: [id, channelId, modelId, createdAt]
      properties:
        id:               { type: string }
        channelId:        { type: string }
        modelId:          { type: string }
        displayName:      { type: string, nullable: true }
        contextWindow:    { type: integer, nullable: true }
        maxOutputTokens:  { type: integer, nullable: true }
        createdAt:        { type: integer }

    CreateModelInput:
      type: object
      required: [modelId]
      properties:
        modelId:          { type: string, minLength: 1 }
        displayName:      { type: string, nullable: true }
        contextWindow:    { type: integer, nullable: true }
        maxOutputTokens:  { type: integer, nullable: true }

    UpdateModelInput:
      type: object
      properties:
        displayName:      { type: string, nullable: true }
        contextWindow:    { type: integer, nullable: true }
        maxOutputTokens:  { type: integer, nullable: true }

    Agent:
      type: object
      required: [id, name, enabled, createdAt, updatedAt]
      properties:
        id:           { type: string }
        name:         { type: string }
        systemPrompt: { type: string, nullable: true }
        avatarUri:    { type: string, nullable: true }
        enabled:      { type: boolean }
        createdAt:    { type: integer }
        updatedAt:    { type: integer }

    CreateAgentInput:
      type: object
      required: [name]
      properties:
        name:         { type: string, minLength: 1 }
        systemPrompt: { type: string, nullable: true }

    UpdateAgentInput:
      type: object
      properties:
        name:         { type: string, minLength: 1 }
        systemPrompt: { type: string, nullable: true }
        enabled:      { type: boolean }

    Conversation:
      type: object
      required: [id, title, archived, pinned, createdAt, updatedAt]
      properties:
        id:             { type: string }
        title:          { type: string }
        agentId:        { type: string, nullable: true }
        channelId:      { type: string, nullable: true }
        channelModelId: { type: string, nullable: true }
        archived:       { type: boolean }
        pinned:         { type: boolean }
        createdAt:      { type: integer }
        updatedAt:      { type: integer }

    CreateConversationInput:
      type: object
      properties:
        title:          { type: string, default: "新会话" }

    UpdateConversationInput:
      type: object
      properties:
        title:          { type: string, minLength: 1 }
        agentId:        { type: string, nullable: true }
        channelId:      { type: string, nullable: true }
        channelModelId: { type: string, nullable: true }
        archived:       { type: boolean }
        pinned:         { type: boolean }

    MessageVersion:
      type: object
      required: [id, nodeId, content, status, createdAt]
      properties:
        id:               { type: string }
        nodeId:           { type: string }
        content:          { type: string }
        status:
          type: string
          enum: [generating, committed, failed, cancelled]
        modelName:        { type: string, nullable: true }
        promptTokens:     { type: integer, nullable: true }
        completionTokens: { type: integer, nullable: true }
        finishReason:     { type: string, nullable: true }
        createdAt:        { type: integer }

    MessageNode:
      type: object
      required: [id, conversationId, role, orderKey, versions, createdAt]
      properties:
        id:              { type: string }
        conversationId:  { type: string }
        authorAgentId:   { type: string, nullable: true }
        role:
          type: string
          enum: [user, assistant]
        orderKey:        { type: string }
        activeVersionId: { type: string, nullable: true }
        versions:
          type: array
          items:
            $ref: "#/components/schemas/MessageVersion"
        createdAt:       { type: integer }

    SendMessageInput:
      type: object
      required: [content]
      properties:
        content:
          type: string
          minLength: 1
          description: 用户消息正文

    SendMessageResult:
      type: object
      required: [userNodeId, userVersionId, assistantNodeId, assistantVersionId]
      properties:
        userNodeId:        { type: string }
        userVersionId:     { type: string }
        assistantNodeId:   { type: string }
        assistantVersionId: { type: string }
        description: |
          前端凭 assistantVersionId 订阅 generation:* 事件，
          凭 conversationId 路由到正确的会话视图。

    RerollResult:
      type: object
      required: [assistantNodeId, assistantVersionId]
      properties:
        newUserVersionId:   { type: string, nullable: true, description: "user node reroll 时创建的新 user version" }
        assistantNodeId:    { type: string }
        assistantVersionId: { type: string }

    DeleteVersionResult:
      type: object
      required: [nodeDeleted]
      properties:
        nodeDeleted:       { type: boolean, description: "true 表示版本是最后一个，node 也被删除" }
        newActiveVersionId: { type: string, nullable: true, description: "切换后的新 active version，nodeDeleted=true 时为 null" }
```

---

## 2.3 接口说明与调用示例

### 渠道管理

#### POST /channels — 创建渠道

**入参：**
- `name`（必填）：渠道显示名
- `baseUrl`（必填）：服务商域名，必须以 `http://` 或 `https://` 开头
- `apiKey`（选填）：API 密钥
- 其余字段全部选填，NULL 时使用 channel_type 对应默认值

**出参：** 完整的 Channel 对象，含服务端生成的 `id`、`createdAt`、`updatedAt`

**错误码：**
- `400 INVALID_URL`：base_url 不符合格式
- `400 NAME_EMPTY`：name 为空字符串

**调用示例：**
```json
// 请求
POST /channels
{
  "name": "My OpenAI",
  "baseUrl": "https://api.openai.com",
  "apiKey": "sk-xxxx"
}

// 响应 201
{
  "id": "019587ab-0000-7abc-8def-000000000001",
  "name": "My OpenAI",
  "channelType": "openai_compatible",
  "baseUrl": "https://api.openai.com",
  "apiKey": "sk-xxxx",
  "authType": null,
  "modelsEndpoint": null,
  "chatEndpoint": null,
  "streamEndpoint": null,
  "enabled": true,
  "createdAt": 1735000000000,
  "updatedAt": 1735000000000
}
```

---

#### DELETE /channels/{id} — 删除渠道

**入参：** path 中的渠道 ID

**出参：** 无（204）

**副作用：**
1. 级联删除该渠道下所有 `api_channel_models`
2. 所有引用该渠道的 `conversations.channel_id` 置为 NULL
3. 所有引用该渠道模型的 `conversations.channel_model_id` 置为 NULL

**错误码：** `404 NOT_FOUND`

---

### 模型管理

#### POST /channels/{channelId}/models — 添加模型

**入参：** `modelId`（必填）、`displayName`（选填）

**错误码：**
- `409 MODEL_ID_CONFLICT`：同渠道下 model_id 已存在

**调用示例：**
```json
// 请求
POST /channels/019587ab-0000-7abc-8def-000000000001/models
{
  "modelId": "gpt-4o",
  "displayName": "GPT-4o"
}

// 响应 201
{
  "id": "019587ab-0001-7abc-8def-000000000002",
  "channelId": "019587ab-0000-7abc-8def-000000000001",
  "modelId": "gpt-4o",
  "displayName": "GPT-4o",
  "contextWindow": null,
  "maxOutputTokens": null,
  "createdAt": 1735000001000
}
```

---

### 消息与生成

#### POST /conversations/{id}/send — 发送消息

**入参：** `content`（必填，不能为空字符串）

**出参：** `SendMessageResult`，包含四个 ID

**后续流程：**
前端拿到 `assistantVersionId` 后，通过 Tauri Channel 监听 `generation:*` 事件，
用 `conversation_id` 路由到对应会话，用 `version_id` 匹配具体楼层。

**错误码（422）：**

| error_code | 含义 |
|---|---|
| `NO_AGENT` | 会话未绑定 Agent |
| `AGENT_DISABLED` | Agent 已禁用 |
| `NO_CHANNEL` | 会话未绑定渠道 |
| `NO_MODEL` | 会话未绑定模型 |
| `CHANNEL_DISABLED` | 渠道已禁用 |

**调用示例：**
```json
// 请求
POST /conversations/019587ab-0002-7abc-8def-000000000003/send
{
  "content": "用 Rust 写一个快速排序"
}

// 响应 200
{
  "userNodeId":        "019587ab-0010-7abc-0000-000000000010",
  "userVersionId":     "019587ab-0011-7abc-0000-000000000011",
  "assistantNodeId":   "019587ab-0012-7abc-0001-000000000012",
  "assistantVersionId":"019587ab-0013-7abc-0001-000000000013"
}

// 随后的 Tauri Channel 事件流
{ "event": "generation:chunk",     "data": { "conversationId": "...", "nodeId": "...0012", "versionId": "...0013", "delta": "当然" } }
{ "event": "generation:chunk",     "data": { "conversationId": "...", "nodeId": "...0012", "versionId": "...0013", "delta": "，这是..." } }
{ "event": "generation:completed", "data": { "conversationId": "...", "nodeId": "...0012", "versionId": "...0013", "promptTokens": 23, "completionTokens": 312, "finishReason": "stop", "model": "gpt-4o" } }
```

---

#### POST /conversations/{id}/nodes/{nodeId}/reroll — Reroll

**入参：** 无 body（目标已由 path 确定）

**出参：** `RerollResult`

**行为差异：**
- `role=assistant`：在同 node 新建 version，开始生成
- `role=user`（仅末尾楼层）：新建 user version（复制内容）+ 新建 assistant node，开始生成

**错误码（422）：**
- `NOT_LAST_USER_NODE`：user node 不是最后一个楼层

**调用示例（assistant reroll）：**
```json
// 请求
POST /conversations/.../nodes/019587ab-0012-7abc.../reroll

// 响应 200
{
  "newUserVersionId": null,
  "assistantNodeId":    "019587ab-0012-7abc-0001-000000000012",
  "assistantVersionId": "019587ab-0020-7abc-0002-000000000020"
}
```

---

#### POST /generations/{versionId}/cancel — 取消生成

**入参：** path 中的 versionId

**出参：** 200（幂等，version 不存在或已完成时同样返回 200）

**副作用：** 触发后台 CancellationToken，后台任务检测到后写库 status=cancelled，发送 `generation:cancelled` 事件

---

#### GET /conversations/{id}/messages — 获取消息列表

**出参：** `MessageNode[]`，每个 node 内嵌其所有 versions

**注意：** 返回全部 versions，前端用 `activeVersionId` 决定展示哪一个，其余版本用于版本切换器

**调用示例：**
```json
// 响应 200
[
  {
    "id": "019587ab-0010-...",
    "conversationId": "...",
    "role": "user",
    "orderKey": "0000000001735000000000-0-a3f9",
    "activeVersionId": "019587ab-0011-...",
    "versions": [
      {
        "id": "019587ab-0011-...",
        "nodeId": "019587ab-0010-...",
        "content": "用 Rust 写一个快速排序",
        "status": "committed",
        "modelName": null,
        "promptTokens": null,
        "completionTokens": null,
        "finishReason": null,
        "createdAt": 1735000001000
      }
    ],
    "createdAt": 1735000001000
  },
  {
    "id": "019587ab-0012-...",
    "role": "assistant",
    "orderKey": "0000000001735000000000-1-a3f9",
    "activeVersionId": "019587ab-0013-...",
    "versions": [
      {
        "id": "019587ab-0013-...",
        "content": "当然，这是 Rust 快速排序的实现...",
        "status": "committed",
        "modelName": "gpt-4o",
        "promptTokens": 23,
        "completionTokens": 312,
        "finishReason": "stop",
        "createdAt": 1735000002000
      }
    ],
    "createdAt": 1735000001000
  }
]
```

---

# 三、设计评审

## 3.1 潜在性能问题

### P1：消息列表 N+1 查询风险

**问题：** `GET /conversations/{id}/messages` 若先查所有 node，再逐个查 versions，会产生 N+1 查询。

**建议：** 使用单条 SQL 通过 JOIN 或子查询一次性拉取，在应用层组装 `node.versions[]`：

```sql
SELECT
    n.id              AS node_id,
    n.role,
    n.order_key,
    n.active_version_id,
    n.author_agent_id,
    n.created_at      AS node_created_at,
    v.id              AS version_id,
    v.content,
    v.status,
    v.model_name,
    v.prompt_tokens,
    v.completion_tokens,
    v.finish_reason,
    v.created_at      AS version_created_at
FROM message_nodes n
LEFT JOIN message_versions v ON v.node_id = n.id
WHERE n.conversation_id = ?
ORDER BY n.order_key ASC, v.created_at ASC;
```

结果在 Rust 应用层按 `node_id` 分组，无多次 IO。

---

### P2：大会话分页缺失

**问题：** 消息列表无分页，若一个会话有数千条消息，首次加载会全量传输。

**建议（MVP 后）：** 增加 `cursor` 或 `before_order_key` 参数，支持向上翻页加载历史。MVP 阶段可接受，但接口签名应预留扩展空间。

---

### P3：`updated_at` 写入竞争

**问题：** 并发生成时，多个后台任务都会 `UPDATE conversations SET updated_at = ?`，在 SQLite WAL 模式下写操作是串行的，高频 chunk 写库时可能形成队列。

**建议：** 后台任务的 chunk 写库（内容 buffer）只写 `message_versions.content`，不触碰 `conversations.updated_at`。仅在 `generation:completed / failed / cancelled` 时更新 `conversations.updated_at` 一次。

---

## 3.2 潜在安全/可靠性问题

### S1：API Key 明文存 SQLite

**问题：** `api_channels.api_key` 当前明文存储，若用户设备被物理访问或 DB 文件被拷走，密钥泄露。

**建议（P1）：** 使用操作系统 Keychain（Tauri 的 `keyring` 插件）存储 API Key，DB 中只存引用标识。MVP 阶段记录为已知风险，文档注明。

---

### S2：`base_url` 拼接 SSRF 风险

**问题：** `base_url` 由用户输入，`final_url = base_url + endpoint`，若用户填入内网地址（如 `http://192.168.1.1`），应用会向内网发起请求。

**当前定位：** 桌面本地应用，用户即管理员，风险可接受。但如果后续上云（P2），此处必须加白名单或沙箱。

**建议：** 在 Rust 侧校验 `base_url` 不是 loopback/私有地址（生产版本），或至少在文档中标注为已知限制。

---

### S3：`cancel_generation` 幂等性依赖内存 DashMap

**问题：** `DashMap<version_id, CancellationToken>` 存在内存中，应用重启后消失。若前端持有旧的 `versionId` 尝试取消，后端找不到 token，但版本已经是 failed 状态——此时幂等性靠"找不到就忽略"实现，没有问题。

**需确认：** 前端在应用重启后不应重新订阅旧的 versionId 的事件。启动清理已将 generating 改为 failed，前端刷新消息列表即可看到终态。行为正确，无需改动，但需要在前端文档中说明"重启后无需重新订阅事件"。

---

### S4：`order_key` 冲突重试无上限

**问题：** `UNIQUE (conversation_id, order_key)` 冲突时应用层重试，但当前文档未定义重试次数上限。

**建议：** 最多重试 3 次，超出返回内部错误。实践中同毫秒内 UUID 随机后缀冲突概率极低，3 次足够。

---

## 3.3 接口设计改进建议

| 编号 | 问题 | 建议 |
|------|------|------|
| A1 | `list_messages` 在大会话下无分页 | 预留 `before_order_key` 参数，MVP 不实现但不要破坏签名 |
| A2 | `reroll` 对 user node 的限制（仅末尾楼层）完全由后端判断，前端也需要知道 | `MessageNode` 返回中可加 `canReroll: boolean` 字段，由后端计算，前端直接用 |
| A3 | `generation:empty_rollback` 事件的 `fallback_version_id` 为 NULL 时，前端需要判断是否删 node | 可拆成 `nodeDeleted: boolean` 字段，语义更清晰（与 `DeleteVersionResult` 保持一致） |
| A4 | `fetch_models_from_channel` 返回纯字符串数组，用户无法知道模型的 context_window | 若服务商返回了元数据，可返回 `{ modelId, displayName }[]`，让用户选择时信息更完整 |
