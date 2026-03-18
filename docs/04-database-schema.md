# 数据库表设计

使用 **SQLite + sqlx**。本文档只保留当前实现范围需要的表，不再混入收藏、分支、临时对话、记忆等后续扩展字段，避免 AI 生成时过度实现。

---

## migrations/001_init.sql

```sql
CREATE TABLE IF NOT EXISTS conversations (
    id            TEXT PRIMARY KEY,
    title         TEXT NOT NULL DEFAULT '新对话',
    model_id      TEXT NOT NULL,
    provider      TEXT NOT NULL,
    assistant_id  TEXT,
    system_prompt TEXT,
    pinned        BOOLEAN NOT NULL DEFAULT 0,
    created_at    INTEGER NOT NULL,
    updated_at    INTEGER NOT NULL
);
CREATE INDEX idx_conversations_updated
    ON conversations(pinned DESC, updated_at DESC);

CREATE TABLE IF NOT EXISTS messages (
    id                TEXT PRIMARY KEY,
    conversation_id   TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    parent_message_id TEXT,
    version_group_id  TEXT NOT NULL,
    version_index     INTEGER NOT NULL DEFAULT 1,
    is_active         BOOLEAN NOT NULL DEFAULT 1,
    role              TEXT NOT NULL,
    content           TEXT,
    content_parts     TEXT,
    tool_calls        TEXT,
    tool_call_id      TEXT,
    citations_json    TEXT,
    tokens_used       INTEGER,
    created_at        INTEGER NOT NULL
);
CREATE INDEX idx_messages_conv_created
    ON messages(conversation_id, created_at ASC);
CREATE INDEX idx_messages_conv_active
    ON messages(conversation_id, is_active, created_at ASC);
CREATE INDEX idx_messages_vgroup
    ON messages(version_group_id, version_index ASC);

CREATE TABLE IF NOT EXISTS provider_configs (
    id          TEXT PRIMARY KEY,
    provider    TEXT NOT NULL UNIQUE,
    api_key_id  TEXT,
    base_url    TEXT,
    extra_json  TEXT,
    enabled     BOOLEAN NOT NULL DEFAULT 1,
    updated_at  INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS custom_channels (
    id                    TEXT PRIMARY KEY,
    name                  TEXT NOT NULL,
    base_url              TEXT NOT NULL,
    auth_json             TEXT NOT NULL,
    endpoints_json        TEXT NOT NULL,
    stream_protocol       TEXT NOT NULL DEFAULT 'sse',
    request_template_json TEXT NOT NULL,
    response_mapping_json TEXT NOT NULL,
    stream_mapping_json   TEXT NOT NULL,
    models_json           TEXT NOT NULL DEFAULT '[]',
    enabled               BOOLEAN NOT NULL DEFAULT 1,
    created_at            INTEGER NOT NULL,
    updated_at            INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS assistants (
    id                 TEXT PRIMARY KEY,
    name               TEXT NOT NULL,
    icon               TEXT NOT NULL DEFAULT 'assistant',
    category           TEXT NOT NULL DEFAULT '通用',
    system_prompt      TEXT NOT NULL DEFAULT '',
    model_id           TEXT,
    provider           TEXT,
    tools_json         TEXT NOT NULL DEFAULT '[]',
    knowledge_base_ids TEXT NOT NULL DEFAULT '[]',
    params_json        TEXT,
    builtin            BOOLEAN NOT NULL DEFAULT 0,
    created_at         INTEGER NOT NULL,
    updated_at         INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS param_presets (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    provider    TEXT,
    params_json TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS documents (
    id           TEXT PRIMARY KEY,
    name         TEXT NOT NULL,
    source_type  TEXT NOT NULL,
    path_or_url  TEXT NOT NULL,
    chunk_count  INTEGER NOT NULL DEFAULT 0,
    status       TEXT NOT NULL DEFAULT 'indexing',
    error_msg    TEXT,
    created_at   INTEGER NOT NULL,
    updated_at   INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS chunks (
    id          TEXT PRIMARY KEY,
    document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    content     TEXT NOT NULL,
    embedding   BLOB NOT NULL,
    chunk_index INTEGER NOT NULL,
    page_number INTEGER
);
CREATE INDEX idx_chunks_document
    ON chunks(document_id, chunk_index ASC);

CREATE TABLE IF NOT EXISTS tools (
    id            TEXT PRIMARY KEY,
    name          TEXT NOT NULL UNIQUE,
    description   TEXT NOT NULL,
    schema_json   TEXT NOT NULL,
    source        TEXT NOT NULL,
    mcp_server_id TEXT,
    enabled       BOOLEAN NOT NULL DEFAULT 1,
    created_at    INTEGER NOT NULL,
    updated_at    INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS mcp_servers (
    id         TEXT PRIMARY KEY,
    name       TEXT NOT NULL,
    transport  TEXT NOT NULL,
    command    TEXT,
    args_json  TEXT,
    env_json   TEXT,
    url        TEXT,
    enabled    BOOLEAN NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
```

---

## 不进入当前版本的数据项

以下字段和表当前不做，文档中也不再保留，避免 AI 擅自生成：

- `tags`
- `is_temporary`
- `branch_id`
- `starred`
- `prompt_versions_json`
- `global_prompts`
- `memories`

---

## 数据库初始化

```rust
// src-tauri/src/db/mod.rs
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

pub async fn init_pool(db_path: &str) -> crate::error::Result<SqlitePool> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&format!("sqlite://{}?mode=rwc", db_path))
        .await?;

    sqlx::query("PRAGMA journal_mode=WAL").execute(&pool).await?;
    sqlx::query("PRAGMA foreign_keys=ON").execute(&pool).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
```

`db_path` 使用 Tauri `app_handle.path().app_data_dir()` 下的 `omnichat.db`。

---

## 关键查询模式

### 对话列表

```sql
SELECT *
FROM conversations
ORDER BY pinned DESC, updated_at DESC
LIMIT ? OFFSET ?;
```

### 当前展示消息链

```sql
SELECT *
FROM messages
WHERE conversation_id = ?
  AND is_active = 1
ORDER BY created_at ASC;
```

### 版本切换

```sql
UPDATE messages
SET is_active = 0
WHERE version_group_id = ?;

UPDATE messages
SET is_active = 1
WHERE version_group_id = ?
  AND version_index = ?;
```

### 重新生成

```sql
SELECT MAX(version_index)
FROM messages
WHERE version_group_id = ?;
```

### 编辑用户消息

```sql
DELETE FROM messages
WHERE conversation_id = ?
  AND created_at >= ?;
```
