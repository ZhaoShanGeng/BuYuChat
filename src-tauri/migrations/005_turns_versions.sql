CREATE TABLE IF NOT EXISTS conversation_turns (
    id                TEXT PRIMARY KEY,
    conversation_id   TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    parent_turn_id    TEXT REFERENCES conversation_turns(id) ON DELETE SET NULL,
    role              TEXT NOT NULL,
    active_version_id TEXT,
    deleted_at        INTEGER,
    created_at        INTEGER NOT NULL,
    updated_at        INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_turns_conversation_parent_created
    ON conversation_turns(conversation_id, parent_turn_id, created_at ASC);

CREATE INDEX IF NOT EXISTS idx_turns_conversation_created
    ON conversation_turns(conversation_id, created_at ASC);

CREATE TABLE IF NOT EXISTS turn_versions (
    id             TEXT PRIMARY KEY,
    turn_id        TEXT NOT NULL REFERENCES conversation_turns(id) ON DELETE CASCADE,
    version_index  INTEGER NOT NULL,
    content        TEXT,
    content_parts  TEXT,
    tool_calls     TEXT,
    tool_call_id   TEXT,
    citations_json TEXT,
    tokens_used    INTEGER,
    provider       TEXT,
    model_id       TEXT,
    created_at     INTEGER NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_turn_versions_turn_version
    ON turn_versions(turn_id, version_index ASC);

CREATE INDEX IF NOT EXISTS idx_turn_versions_turn_created
    ON turn_versions(turn_id, created_at ASC);
