# 步语 BuYu — 数据库设计

**版本：** 0.4
**阶段：** MVP（P0）
**数据库：** SQLite（WAL 模式，通过 sqlx 异步查询）

---

## 1. 表总览

| 表名 | 用途 | 行数量级（MVP） |
|------|------|----------------|
| `api_channels` | AI 服务渠道配置 | ~10 |
| `api_channel_models` | 渠道下的模型列表 | ~50 |
| `agents` | Agent 定义（含系统提示词） | ~10 |
| `conversations` | 会话，MVP 内嵌 Agent/渠道/模型绑定 | ~100 |
| `message_nodes` | 消息楼层（位置实体） | ~10,000 |
| `message_versions` | 消息版本（元数据 + 状态机） | ~15,000 |
| `message_contents` | 消息内容（分块存储，与版本分离） | ~20,000 |

### 设计理念：内容与版本分离

`message_versions` 只存元数据（status、tokens、model_name 等），实际文本内容存在 `message_contents` 表中，按 chunk 分块。

**好处：**
- 查询版本列表时不传输大文本，`list_messages` 只为 active version 拼接 content
- 流式生成时 INSERT chunk 行（追加），不反复 UPDATE 同一行
- 为未来多模态内容（图片/文件）预留 `content_type` 字段
- 大文本分块存储，避免单行膨胀

---

## 2. 完整 DDL

```sql
-- ============================================================
-- PRAGMA 设置（应用启动时执行，非建表语句）
-- ============================================================
PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;
PRAGMA synchronous = NORMAL;
PRAGMA busy_timeout = 5000;

-- ============================================================
-- 表1: api_channels
-- 用途: AI 服务渠道配置
-- ============================================================
CREATE TABLE IF NOT EXISTS api_channels (
    id              TEXT    NOT NULL,
    name            TEXT    NOT NULL,
    channel_type    TEXT    NOT NULL DEFAULT 'openai_compatible',
    base_url        TEXT    NOT NULL,
    api_key         TEXT,                            -- MVP 明文，P1 迁移 Keychain
    auth_type       TEXT,                            -- NULL = channel_type 默认值
    models_endpoint TEXT,                            -- NULL → /v1/models
    chat_endpoint   TEXT,                            -- NULL → /v1/chat/completions
    stream_endpoint TEXT,                            -- NULL → /v1/chat/completions
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
-- 表2: api_channel_models
-- 用途: 渠道下的可用模型
-- ============================================================
CREATE TABLE IF NOT EXISTS api_channel_models (
    id                TEXT    NOT NULL,
    channel_id        TEXT    NOT NULL,
    model_id          TEXT    NOT NULL,
    display_name      TEXT,
    context_window    INTEGER,
    max_output_tokens INTEGER,

    PRIMARY KEY (id),
    FOREIGN KEY (channel_id) REFERENCES api_channels (id)
        ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED,
    UNIQUE (channel_id, model_id),
    CHECK (length(model_id) > 0)
);

CREATE INDEX IF NOT EXISTS idx_api_channel_models_channel_id
    ON api_channel_models (channel_id);


-- ============================================================
-- 表3: agents
-- 用途: Agent 定义
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
-- 用途: 会话，MVP 内嵌 Agent/渠道/模型绑定
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
    updated_at       INTEGER NOT NULL,               -- 仅在生成终态时更新

    PRIMARY KEY (id),
    FOREIGN KEY (agent_id) REFERENCES agents (id)
        ON DELETE SET NULL DEFERRABLE INITIALLY DEFERRED,
    FOREIGN KEY (channel_id) REFERENCES api_channels (id)
        ON DELETE SET NULL DEFERRABLE INITIALLY DEFERRED,
    FOREIGN KEY (channel_model_id) REFERENCES api_channel_models (id)
        ON DELETE SET NULL DEFERRABLE INITIALLY DEFERRED,
    CHECK (archived IN (0, 1)),
    CHECK (pinned IN (0, 1)),
    CHECK (length(title) > 0)
);

CREATE INDEX IF NOT EXISTS idx_conversations_list
    ON conversations (archived, pinned DESC, updated_at DESC);

CREATE INDEX IF NOT EXISTS idx_conversations_agent_id
    ON conversations (agent_id) WHERE agent_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_conversations_channel_id
    ON conversations (channel_id) WHERE channel_id IS NOT NULL;


-- ============================================================
-- 表5: message_nodes
-- 用途: 消息楼层（位置实体）
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
    FOREIGN KEY (conversation_id) REFERENCES conversations (id)
        ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED,
    FOREIGN KEY (author_agent_id) REFERENCES agents (id)
        ON DELETE SET NULL DEFERRABLE INITIALLY DEFERRED,
    FOREIGN KEY (active_version_id) REFERENCES message_versions (id)
        ON DELETE SET NULL DEFERRABLE INITIALLY DEFERRED,
    UNIQUE (conversation_id, order_key),
    CHECK (role IN ('user', 'assistant'))
);

CREATE INDEX IF NOT EXISTS idx_message_nodes_conversation_order
    ON message_nodes (conversation_id, order_key ASC);


-- ============================================================
-- 表6: message_versions
-- 用途: 消息版本元数据（不含内容文本）
--       内容存在 message_contents 中
-- ============================================================
CREATE TABLE IF NOT EXISTS message_versions (
    id                TEXT    NOT NULL,
    node_id           TEXT    NOT NULL,
    status            TEXT    NOT NULL DEFAULT 'generating',
    model_name        TEXT,                          -- 存 model_id 字符串，不存外键
    prompt_tokens     INTEGER,
    completion_tokens INTEGER,
    finish_reason     TEXT,
    created_at        INTEGER NOT NULL,

    PRIMARY KEY (id),
    FOREIGN KEY (node_id) REFERENCES message_nodes (id)
        ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED,
    CHECK (status IN ('generating', 'committed', 'failed', 'cancelled'))
);

CREATE INDEX IF NOT EXISTS idx_message_versions_node_id
    ON message_versions (node_id, created_at ASC);

CREATE INDEX IF NOT EXISTS idx_message_versions_status
    ON message_versions (status) WHERE status = 'generating';


-- ============================================================
-- 表7: message_contents
-- 用途: 消息内容分块存储
--       与 message_versions 分离，支持：
--       1. 流式追加（INSERT chunk，不 UPDATE）
--       2. 按需加载（非 active version 不读 content）
--       3. 未来多模态扩展（content_type 字段）
-- ============================================================
CREATE TABLE IF NOT EXISTS message_contents (
    id              TEXT    NOT NULL,                -- UUID v7
    version_id      TEXT    NOT NULL,                -- 所属版本
    chunk_index     INTEGER NOT NULL,                -- 块序号，从 0 开始
    content_type    TEXT    NOT NULL DEFAULT 'text/plain',
                                                     -- 'text/plain' | 'text/markdown'
                                                     -- P1: 'image/ref' | 'file/ref'
    body            TEXT    NOT NULL DEFAULT '',      -- 实际文本内容，每块 ≤ 64KB
    created_at      INTEGER NOT NULL,

    PRIMARY KEY (id),
    FOREIGN KEY (version_id) REFERENCES message_versions (id)
        ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED,
    UNIQUE (version_id, chunk_index)
);

-- 拼接版本完整内容：WHERE version_id=? ORDER BY chunk_index ASC
CREATE INDEX IF NOT EXISTS idx_message_contents_version
    ON message_contents (version_id, chunk_index ASC);
```

---

## 3. 索引设计说明

| 索引 | 类型 | 查询场景 |
|------|------|---------|
| `idx_api_channels_created_at` | 单列降序 | 渠道列表页排序 |
| `idx_api_channel_models_channel_id` | 单列 | 按渠道查模型列表 |
| `idx_agents_created_at` | 单列降序 | Agent 列表排序 |
| `idx_conversations_list` | 复合覆盖索引 | 会话列表排序 |
| `idx_conversations_agent_id` | 部分索引 (NOT NULL) | Agent 删除时批量更新 |
| `idx_conversations_channel_id` | 部分索引 (NOT NULL) | 渠道删除时批量查找 |
| `idx_message_nodes_conversation_order` | 复合索引 | 拉取会话消息（高频） |
| `idx_message_versions_node_id` | 复合索引 + 时间 | 版本切换器、reroll |
| `idx_message_versions_status` | 部分索引 (generating) | 启动清理 |
| `idx_message_contents_version` | 复合索引 | 拼接版本完整内容 |

---

## 4. 外键约束与级联策略

| 外键 | ON DELETE | 理由 |
|------|-----------|------|
| `api_channel_models → api_channels` | CASCADE | 渠道删除时模型无意义 |
| `conversations → agents` | SET NULL | 保留会话历史 |
| `conversations → api_channels` | SET NULL | 保留会话历史 |
| `conversations → api_channel_models` | SET NULL | 保留会话历史 |
| `message_nodes → conversations` | CASCADE | 会话删除时清除所有楼层 |
| `message_nodes → agents (author)` | SET NULL | 保留消息内容 |
| `message_nodes → message_versions (active)` | SET NULL | 应用层重新指向 |
| `message_versions → message_nodes` | CASCADE | 楼层删除时清除版本 |
| `message_contents → message_versions` | CASCADE | 版本删除时清除内容 |

**循环引用处理：** `message_nodes.active_version_id` ↔ `message_versions.node_id` 使用 `DEFERRABLE INITIALLY DEFERRED`。

---

## 5. 启动清理

```sql
UPDATE message_versions SET status = 'failed' WHERE status = 'generating';
```

---

## 6. 核心查询

### 6.1 消息列表（只加载 active version 的 content）

```sql
-- 第一步：获取所有 node + 全部 version 元数据（不含 content）
SELECT
    n.id AS node_id, n.role, n.order_key, n.active_version_id,
    n.author_agent_id, n.created_at AS node_created_at,
    v.id AS version_id, v.status, v.model_name,
    v.prompt_tokens, v.completion_tokens, v.finish_reason,
    v.created_at AS version_created_at
FROM message_nodes n
LEFT JOIN message_versions v ON v.node_id = n.id
WHERE n.conversation_id = ?
ORDER BY n.order_key ASC, v.created_at ASC;

-- 第二步：批量加载 active version 的 content
-- 应用层收集所有 active_version_id，IN 查询拼接内容
SELECT version_id, body
FROM message_contents
WHERE version_id IN (?, ?, ?, ...)
ORDER BY version_id, chunk_index ASC;
```

应用层按 `version_id` 分组，将 chunks 拼接为完整 content。两次查询，零 N+1。

### 6.2 按需加载单个版本的 content

```sql
-- 用户切换版本时，前端按需请求
SELECT body
FROM message_contents
WHERE version_id = ?
ORDER BY chunk_index ASC;
```

### 6.3 上下文构建

```sql
-- 取所有 active version 的完整文本，用于构建 AI 请求
SELECT n.role, c.body, c.chunk_index
FROM message_nodes n
JOIN message_contents c ON c.version_id = n.active_version_id
WHERE n.conversation_id = ?
  AND n.id != ?              -- 排除当前正在生成的 assistant node
ORDER BY n.order_key ASC, c.chunk_index ASC;
```

### 6.4 流式写入 content

```sql
-- 每次 flush buffer 时 INSERT 新 chunk（不 UPDATE）
INSERT INTO message_contents (id, version_id, chunk_index, content_type, body, created_at)
VALUES (?, ?, ?, 'text/plain', ?, ?);
```

### 6.5 会话列表

```sql
SELECT * FROM conversations
WHERE archived = ?
ORDER BY pinned DESC, updated_at DESC;
```

---

## 7. 内容存储规则

### 7.1 写入

| 场景 | 行为 |
|------|------|
| 用户发送消息 | 单个 chunk（chunk_index=0），一次 INSERT |
| AI 流式生成 | 每次 flush buffer 追加一个 chunk |
| AI 非流式生成 | 完成后单个 chunk INSERT |

### 7.2 读取

| 场景 | 行为 |
|------|------|
| `list_messages` | 只加载 active version 的 chunks |
| 版本切换 | 按需加载目标 version 的 chunks |
| 上下文构建 | 加载所有 active version 的 chunks |

### 7.3 大小限制

- 单个 chunk body ≤ 64KB
- 单个 version 总 content ≤ 512KB（应用层校验）
- AI 生成时 buffer 超限自动截断，`finish_reason = "length"`

---

## 8. 数据迁移策略

### 8.1 工具选型

使用 `sqlx::migrate!("./migrations")` 在应用启动时自动执行迁移，不手写自定义迁移框架。

```
src-tauri/
└── migrations/
    ├── 0000_initial_schema.sql          # 当前 MVP 基线 schema
    ├── 0001_add_xxx.sql                 # 后续增量迁移
    └── ...
```

### 8.2 迁移文件规范

| 规则 | 说明 |
|------|------|
| 文件名 | `NNNN_description.sql`，四位递增序号，从 `0000` 开始 |
| 内容 | 纯 SQL，每个文件一个原子变更 |
| 方向 | 仅前进（不写 down 回滚脚本） |
| 幂等 | 使用 `IF NOT EXISTS` / `IF EXISTS` |
| 事务 | sqlx 自动在事务中执行每个迁移文件 |

### 8.3 当前基线说明

当前仓库已经采用“重写并重建”的基线策略：

1. 删除旧的时间戳迁移文件。
2. 将 MVP 所需的完整 schema 折叠进 `0000_initial_schema.sql`。
3. 本地开发数据库视为可重建资产，schema 发生大范围调整时可直接删库重建。

当前基线迁移文件为：

```text
src-tauri/migrations/0000_initial_schema.sql
```

### 8.4 运行时机

```rust
// lib.rs — Tauri Builder 初始化时
sqlx::migrate!("./migrations")
    .run(&pool)
    .await
    .expect("database migration failed");
```

应用每次启动自动执行未跑过的迁移，`_sqlx_migrations` 表记录已执行的版本。

### 8.5 常见迁移场景

```sql
-- 新增列
ALTER TABLE agents ADD COLUMN description TEXT;

-- 新增表
CREATE TABLE IF NOT EXISTS conversation_agents ( ... );

-- 数据迁移（P1 拆中间表时）
INSERT INTO conversation_agents (conversation_id, agent_id)
SELECT id, agent_id FROM conversations WHERE agent_id IS NOT NULL;

ALTER TABLE conversations DROP COLUMN agent_id;  -- SQLite 3.35.0+
```

### 8.6 开发环境重建策略

在当前 MVP 阶段，如果迁移历史被重写，例如：

- 基线文件重命名
- 多张核心表一起调整
- 外键结构整体变化

允许直接删除本地开发数据库和 `_sqlx_migrations` 记录后重新启动应用，让 `0000_initial_schema.sql` 重新建库。

### 8.7 回滚策略

SQLite 不支持 `ALTER TABLE DROP COLUMN`（3.35.0 之前）。回滚方案：

1. **优先**：发布新的前进迁移修复问题（不回退）
2. **紧急**：从备份恢复 DB 文件（应用启动前自动备份 P1）
3. **开发环境**：删除 DB 文件重新建表
