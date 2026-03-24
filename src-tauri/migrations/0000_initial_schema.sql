PRAGMA foreign_keys = ON;

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

CREATE TABLE IF NOT EXISTS message_versions (
    id                TEXT    NOT NULL,
    node_id           TEXT    NOT NULL,
    status            TEXT    NOT NULL DEFAULT 'generating',
    model_name        TEXT,
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

CREATE TABLE IF NOT EXISTS message_contents (
    id           TEXT    NOT NULL,
    version_id   TEXT    NOT NULL,
    chunk_index  INTEGER NOT NULL,
    content_type TEXT    NOT NULL DEFAULT 'text/plain',
    body         TEXT    NOT NULL DEFAULT '',
    created_at   INTEGER NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (version_id) REFERENCES message_versions (id)
        ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED,
    UNIQUE (version_id, chunk_index)
);

CREATE INDEX IF NOT EXISTS idx_message_contents_version
    ON message_contents (version_id, chunk_index ASC);
