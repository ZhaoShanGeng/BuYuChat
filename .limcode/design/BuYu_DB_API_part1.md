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
-- 字段说明:
--   id           UUID v7，时间有序，全局唯一
--   name         用户自定义显示名，不允许为空
--   channel_type 服务商类型，当前仅 'openai_compatible'
--   base_url     域名，不含路径，如 https://api.openai.com
--   api_key      API 密钥，允许为空（本地部署无需认证）
--                已知风险：MVP 阶段明文存储，P1 迁移至系统 Keychain
--   auth_type    NULL=使用 channel_type 默认值; 'bearer'/'x_api_key'/'none'
--   *_endpoint   NULL=使用 channel_type 对应默认值
--   enabled      1=启用, 0=禁用
--   created_at   Unix 毫秒时间戳
--   updated_at   Unix 毫秒时间戳
-- ============================================================
CREATE TABLE IF NOT EXISTS api_channels (
    id              TEXT    NOT NULL,
    name            TEXT    NOT NULL,
    channel_type    TEXT    NOT NULL DEFAULT 'openai_compatible',
    base_url        TEXT    NOT NULL,
    api_key         TEXT,
    auth_type       TEXT,
    models_endpoint TEXT,
    chat_endpoint   TEXT,
    stream_endpoint TEXT,
    enabled         INTEGER NOT NULL DEFAULT 1,
    created_at      INTEGER NOT NULL,
    updated_at      INTEGER NOT NULL,

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
-- 字段说明:
--   model_id      实际调用标识，如 'gpt-4o'，同渠道内唯一
--   display_name  用户友好名称，NULL 时前端展示 model_id
--   context_window / max_output_tokens  可选元数据
-- ============================================================
CREATE TABLE IF NOT EXISTS api_channel_models (
    id                TEXT    NOT NULL,
    channel_id        TEXT    NOT NULL,
    model_id          TEXT    NOT NULL,
    display_name      TEXT,
    context_window    INTEGER,
    max_output_tokens INTEGER,

    PRIMARY KEY (id),
    FOREIGN KEY (channel_id)
        REFERENCES api_channels (id)
        ON DELETE CASCADE
        DEFERRABLE INITIALLY DEFERRED,
    UNIQUE (channel_id, model_id),
    CHECK (length(model_id) > 0)
);

-- 按渠道查询模型列表（最常用路径）
CREATE INDEX IF NOT EXISTS idx_api_channel_models_channel_id
    ON api_channel_models (channel_id);


-- ============================================================
-- 表3: agents
-- 用途: Agent 定义，包含系统提示词
-- 字段说明:
--   system_prompt  NULL 或空字符串均表示无提示词，无内置默认值
--   avatar_uri     MVP 阶段可不实现
-- ============================================================
CREATE TABLE IF NOT EXISTS agents (
    id            TEXT    NOT NULL,
    name          TEXT    NOT NULL,
    system_prompt TEXT,
    avatar_uri    TEXT,
    enabled       INTEGER NOT NULL DEFAULT 1,
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
--       P1 引入多 Agent 协作时拆出 conversation_agents 中间表并做数据迁移
-- 字段说明:
--   agent_id / channel_id / channel_model_id  均允许 NULL（未配置状态）
--   archived      1=已归档，不出现在活跃列表
--   pinned        1=已置顶，始终排在活跃列表最前
--   updated_at    有新消息时更新（仅在生成终态时写，不在 chunk 时写）
-- ============================================================
CREATE TABLE IF NOT EXISTS conversations (
    id               TEXT    NOT NULL,
    title            TEXT    NOT NULL DEFAULT '新会话',
    agent_id         TEXT,
    channel_id       TEXT,
    channel_model_id TEXT,
    archived         INTEGER NOT NULL DEFAULT 0,
    pinned           INTEGER NOT NULL DEFAULT 0,
    created_at       INTEGER NOT NULL,
    updated_at       INTEGER NOT NULL,

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

-- 会话列表主查询：置顶优先，再按 updated_at 降序（覆盖索引，避免回表）
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
--       node 本身无状态字段，状态全部在 message_versions 中。
-- 字段说明:
--   author_agent_id  发出该楼层的 Agent，user 消息为 NULL
--   role             'user' 或 'assistant'
--   order_key        字典序即消息顺序，格式: {timestamp_ms:016}-{pos}-{random}
--                    user 用 -0-，assistant 用 -1-，保证同批 user 在 assistant 前
--   active_version_id 当前展示的版本，初始为 NULL 直到第一个版本创建
-- ============================================================
CREATE TABLE IF NOT EXISTS message_nodes (
    id                TEXT    NOT NULL,
    conversation_id   TEXT    NOT NULL,
    author_agent_id   TEXT,
    role              TEXT    NOT NULL,
    order_key         TEXT    NOT NULL,
    active_version_id TEXT,
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
    UNIQUE (conversation_id, order_key),
    CHECK (role IN ('user', 'assistant'))
);

-- 拉取会话消息列表（核心高频查询）
CREATE INDEX IF NOT EXISTS idx_message_nodes_conversation_order
    ON message_nodes (conversation_id, order_key ASC);


-- ============================================================
-- 表6: message_versions
-- 用途: 消息版本（内容实体）。状态机的载体。
--       一个 node 可有多个 version（reroll、编辑产生）
-- 字段说明:
--   content          generating 期间逐步追加，初始为空字符串
--   status           状态机：generating→committed/failed/cancelled
--   model_name       存 model_id 字符串，不存外键，历史记录不受删除影响
--   prompt_tokens    生成完成后填充
--   completion_tokens 生成完成后填充
--   finish_reason    'stop'|'length'|'content_filter' 等
-- ============================================================
CREATE TABLE IF NOT EXISTS message_versions (
    id                TEXT    NOT NULL,
    node_id           TEXT    NOT NULL,
    content           TEXT    NOT NULL DEFAULT '',
    status            TEXT    NOT NULL DEFAULT 'generating',
    model_name        TEXT,
    prompt_tokens     INTEGER,
    completion_tokens INTEGER,
    finish_reason     TEXT,
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
|------|------|----------|
| `idx_conversations_list` | 覆盖索引，避免回表 | `WHERE archived=0 ORDER BY pinned DESC, updated_at DESC` |
| `idx_conversations_agent_id` | 部分索引（非 NULL） | Agent 禁用/删除时批量查找受影响会话 |
| `idx_conversations_channel_id` | 部分索引（非 NULL） | 渠道禁用/删除时批量查找 |
| `idx_message_nodes_conversation_order` | 复合索引 | `WHERE conversation_id=? ORDER BY order_key ASC` |
| `idx_message_versions_node_id` | 复合索引含时间 | 版本切换器展示所有版本；reroll 后更新 active |
| `idx_message_versions_status` | 部分索引仅 generating | 启动清理，通常命中行数为 0，极快 |

---

## 1.4 外键约束与数据一致性

| 外键 | ON DELETE 策略 | 理由 |
|------|---------------|------|
| `api_channel_models → api_channels` | CASCADE | 渠道删除时模型失去意义，一并清除 |
| `conversations → agents` | SET NULL | 删 Agent 不应影响会话历史 |
| `conversations → api_channels` | SET NULL | 历史消息仍可查看 |
| `conversations → api_channel_models` | SET NULL | 同上 |
| `message_nodes → conversations` | CASCADE | 会话删除时所有楼层一并删除 |
| `message_nodes → agents (author)` | SET NULL | Agent 删除后历史消息作者置空，内容保留 |
| `message_nodes → message_versions (active)` | SET NULL | 版本删除时 active 指针置空，应用层重新指向 |
| `message_versions → message_nodes` | CASCADE | 楼层删除时版本内容一并删除 |

**DEFERRABLE INITIALLY DEFERRED：**
`message_nodes.active_version_id` 与 `message_versions.node_id` 存在循环引用。DEFERRED 约束允许同一事务内先插入两表，提交时再检查外键，避免插入顺序问题。

---

## 1.5 启动清理 SQL

```sql
-- 应用每次启动时执行，清理崩溃残留
UPDATE message_versions
SET status = 'failed'
WHERE status = 'generating';
```

---

## 1.6 核心查询 SQL

### 消息列表（单 SQL，避免 N+1）

```sql
SELECT
    n.id                AS node_id,
    n.role,
    n.order_key,
    n.active_version_id,
    n.author_agent_id,
    n.created_at        AS node_created_at,
    v.id                AS version_id,
    v.content,
    v.status,
    v.model_name,
    v.prompt_tokens,
    v.completion_tokens,
    v.finish_reason,
    v.created_at        AS version_created_at
FROM message_nodes n
LEFT JOIN message_versions v ON v.node_id = n.id
WHERE n.conversation_id = ?
ORDER BY n.order_key ASC, v.created_at ASC;
```

Rust 应用层按 `node_id`