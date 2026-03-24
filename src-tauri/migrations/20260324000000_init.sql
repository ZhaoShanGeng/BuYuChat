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
    FOREIGN KEY (channel_id) REFERENCES api_channels (id)
        ON DELETE SET NULL DEFERRABLE INITIALLY DEFERRED,
    FOREIGN KEY (channel_model_id) REFERENCES api_channel_models (id)
        ON DELETE SET NULL DEFERRABLE INITIALLY DEFERRED,
    CHECK (archived IN (0, 1)),
    CHECK (pinned IN (0, 1)),
    CHECK (length(title) > 0)
);

CREATE INDEX IF NOT EXISTS idx_conversations_channel_id
    ON conversations (channel_id) WHERE channel_id IS NOT NULL;
