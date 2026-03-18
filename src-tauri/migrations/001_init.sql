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
