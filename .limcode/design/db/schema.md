# 数据库 Schema 设计

**版本：** 0.2 | **数据库：** SQLite (WAL 模式) | **ORM：** sqlx 运行时查询

---

## 表总览

| 表名 | 职责 | 关键特性 |
|------|------|----------|
| `api_channels` | AI 服务商接入配置 | base_url CHECK、enabled 开关 |
| `api_channel_models` | 渠道下的模型列表 | UNIQUE(channel_id, model_id) |
| `agents` | Agent 定义与系统提示词 | system_prompt 无默认值 |
| `conversations` | 会话，MVP 内嵌绑定关系 | 三个 SET NULL 外键；P1 拆表 |
| `message_nodes` | 消息楼层（位置实体，无状态） | UNIQUE(conversation_id, order_key)；循环外键 DEFERRED |
| `message_versions` | 消息版本（内容实体，状态机载体） | status CHECK；content 逐步追加 |

---

## 完整 DDL

```sql
PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;
PRAGMA synchronous = NORMAL;

-- ============================================================
-- 表 1: api_channels
-- 职责: 存储 AI 服务渠道配置，每条记录对应一个服务商接入点
--
-- 字段说明:
--   id              UUID v7，时间有序，全局唯一
--   name            用户自定义显示名，不允许为空
--   channel_type    服务商类型，当前仅 'openai_compatible'
--   base_url        域名，不含路径，如 https://api.openai.com
--   api_key         API 密钥，允许 NULL（本地部署无需认证）
--                   [已知风险] MVP 明文存储，P1 迁移至系统 Keychain
--   auth_type       NULL=使用 channel_type 默认值
--                   可选值: 'bearer' | 'x_api_key' | 'none'
--   *_endpoint      NULL=使用 channel_type 对应默认值
--   enabled         1=启用, 0=禁用
--   created_at      Unix 毫秒时间戳
--   updated_at      Unix 毫秒时间戳
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

CREATE INDEX IF NOT EXISTS idx_api_channels_created_at
    ON api_channels (created_at DESC);


-- ============================================================
-- 表 2: api_channel_models
-- 职责: 渠道下的可用模型列表，model_id 是实际 API 调用标识符
--
-- 字段说明:
--   model_id          实际调用标识，如 'gpt-4o'，同渠道内唯一
--   display_name      用户友好名称，NULL 时前端展示 model_id
--   context_window    上下文窗口大小（tokens），可选元数据
--   max_output_tokens 最大输出 tokens，可选元数据
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

CREATE INDEX IF NOT EXISTS idx_api_channel_models_channel_id
    ON api_channel_models (channel_id);


-- ============================================================
-- 表 3: agents
-- 职责: Agent 定义，包含系统提示词
--
-- 字段说明:
--   system_prompt   NULL 或空字符串均表示无提示词，无内置默认值
--                   每次发送消息时实时读取，不做快照
--   avatar_uri      头像 URI，MVP 阶段可不实现
--   enabled         1=启用, 0=禁用；禁用时绑定会话无法发新消息
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
-- 表 4: conversations
-- 职责: 会话主体，MVP 直接内嵌 Agent/渠道/模型绑定
--       P1 引入多 Agent 协作时拆出 conversation_agents 中间表
--
-- 字段说明:
--   agent_id / channel_id / channel_model_id  均允许 NULL（未配置）
--   archived     1=已归档，不出现在活跃列表
--   pinned       1=已置顶，始终排在活跃列表最前
--   updated_at   仅在生成终态（completed/failed/cancelled）时更新
--                chunk 写库期间不触碰，避免 WAL 写竞争
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

-- 会话列表主查询覆盖索引：archived 过滤 → 置顶优先 → updated_at 降序
CREATE INDEX IF NOT EXISTS idx_conversations_list
    ON conversations (archived, pinned DESC, updated_at DESC);

-- 部分索引：Agent 禁用/删除时批量查找受影响会话
CREATE INDEX IF NOT EXISTS idx_conversations_agent_id
    ON conversations (agent_id)
    WHERE agent_id IS NOT NULL;

-- 部分索引：渠道禁用/删除时批量查找受影响会话
CREATE INDEX IF NOT EXISTS idx_conversations_channel_id
    ON conversations (channel_id)
    WHERE channel_id IS NOT NULL;


-- ============================================================
-- 表 5: message_nodes
-- 职责: 消息楼层（位置实体）。只管"在哪里"，不管"内容是什么"。
--       node 本身无状态字段，状态全部在 message_versions 中。
--
-- 字段说明:
--   author_agent_id   发出该楼层的 Agent，user 消息为 NULL
--   role              'user' | 'assistant'
--   order_key         字典序即消息顺序
--                     格式: {timestamp_ms:016}-{pos_tag}-{random_4}
--                     同批创建: user 用 -0-，assistant 用 -1-
--                     冲突时应用层最多重试 3 次，超出返回 ORDER_KEY_CONFLICT
--   active_version_id 当前展示的版本 ID
--                     初始为 NULL，第一个 version 创建后立即更新
--
-- 循环外键:
--   message_nodes.active_version_id → message_versions.id
--   message_versions.node_id        → message_nodes.id
--   两者均设 DEFERRABLE INITIALLY DEFERRED，允许同事务内先
--   插入双方，提交时再校验，避免插入顺序死锁。
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

-- 核心高频查询：拉取会话全部楼层，按 order_key 升序
CREATE INDEX IF NOT EXISTS idx_message_nodes_conversation_order
    ON message_nodes (conversation_id, order_key ASC);


-- ============================================================
-- 表 6: message_versions
-- 职责: 消息版本（内容实体）。状态机的载体。
--       一个 node 可有多个 version（reroll、P1 编辑功能产生）
--
-- 字段说明:
--   content           generating 期间逐步追加，初始为空字符串
--                     chunk 写库只写此字段，不触碰其他表
--   status            状态机: generating | committed | failed | cancelled
--   model_name        存 model_id 字符串，不存外键
--                     渠道/模型删除后历史记录仍可读（有意设计）
--                     user 消息此字段为 NULL
--   prompt_tokens     生成完成后填充，generating 期间为 NULL
--