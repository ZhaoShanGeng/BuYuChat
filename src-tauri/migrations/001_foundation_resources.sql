CREATE TABLE IF NOT EXISTS api_channels (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    channel_type TEXT NOT NULL,
    base_url TEXT NOT NULL,
    auth_type TEXT NOT NULL DEFAULT 'bearer',
    api_key TEXT,
    models_endpoint TEXT,
    chat_endpoint TEXT,
    stream_endpoint TEXT,
    models_mode TEXT NOT NULL DEFAULT 'hybrid',
    enabled INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    config_json TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_api_channels_enabled_sort
    ON api_channels(enabled, sort_order);
CREATE INDEX IF NOT EXISTS idx_api_channels_updated
    ON api_channels(updated_at DESC);

CREATE TABLE IF NOT EXISTS api_channel_models (
    id TEXT PRIMARY KEY,
    channel_id TEXT NOT NULL,
    model_id TEXT NOT NULL,
    display_name TEXT,
    model_type TEXT,
    context_window INTEGER,
    max_output_tokens INTEGER,
    capabilities_json TEXT NOT NULL DEFAULT '{}',
    pricing_json TEXT NOT NULL DEFAULT '{}',
    default_parameters_json TEXT NOT NULL DEFAULT '{}',
    sort_order INTEGER NOT NULL DEFAULT 0,
    config_json TEXT NOT NULL DEFAULT '{}',
    FOREIGN KEY (channel_id) REFERENCES api_channels(id)
);

CREATE INDEX IF NOT EXISTS idx_api_channel_models_channel
    ON api_channel_models(channel_id, sort_order);
CREATE UNIQUE INDEX IF NOT EXISTS ux_api_channel_models_channel_model
    ON api_channel_models(channel_id, model_id);

CREATE TABLE IF NOT EXISTS content_objects (
    id TEXT PRIMARY KEY,
    content_type TEXT NOT NULL,
    storage_kind TEXT NOT NULL DEFAULT 'inline',
    text_content TEXT,
    primary_storage_uri TEXT,
    mime_type TEXT,
    size_bytes INTEGER,
    preview_text TEXT,
    sha256 TEXT,
    config_json TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_content_objects_type
    ON content_objects(content_type);
CREATE INDEX IF NOT EXISTS idx_content_objects_sha256
    ON content_objects(sha256);

CREATE TABLE IF NOT EXISTS content_chunks (
    id TEXT PRIMARY KEY,
    content_id TEXT NOT NULL,
    chunk_index INTEGER NOT NULL,
    storage_uri TEXT NOT NULL,
    byte_offset INTEGER NOT NULL,
    byte_length INTEGER NOT NULL,
    compression TEXT,
    checksum TEXT,
    FOREIGN KEY (content_id) REFERENCES content_objects(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_content_chunks_content_chunk
    ON content_chunks(content_id, chunk_index);

CREATE TABLE IF NOT EXISTS presets (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    enabled INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    config_json TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_presets_enabled_sort
    ON presets(enabled, sort_order);
CREATE INDEX IF NOT EXISTS idx_presets_updated
    ON presets(updated_at DESC);

CREATE TABLE IF NOT EXISTS preset_entries (
    id TEXT PRIMARY KEY,
    preset_id TEXT NOT NULL,
    name TEXT NOT NULL,
    role TEXT NOT NULL,
    primary_content_id TEXT NOT NULL,
    position_type TEXT NOT NULL DEFAULT 'relative',
    list_order INTEGER NOT NULL DEFAULT 0,
    depth INTEGER,
    depth_order INTEGER NOT NULL DEFAULT 0,
    triggers_json TEXT NOT NULL DEFAULT '[]',
    enabled INTEGER NOT NULL DEFAULT 1,
    is_pinned INTEGER NOT NULL DEFAULT 0,
    config_json TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (preset_id) REFERENCES presets(id),
    FOREIGN KEY (primary_content_id) REFERENCES content_objects(id)
);

CREATE INDEX IF NOT EXISTS idx_preset_entries_preset_list
    ON preset_entries(preset_id, list_order);
CREATE INDEX IF NOT EXISTS idx_preset_entries_preset_depth
    ON preset_entries(preset_id, role, depth, depth_order);
CREATE INDEX IF NOT EXISTS idx_preset_entries_role
    ON preset_entries(role);

CREATE TABLE IF NOT EXISTS preset_channel_bindings (
    id TEXT PRIMARY KEY,
    preset_id TEXT NOT NULL,
    channel_id TEXT NOT NULL,
    channel_model_id TEXT,
    binding_type TEXT NOT NULL DEFAULT 'available',
    enabled INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    config_json TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (preset_id) REFERENCES presets(id),
    FOREIGN KEY (channel_id) REFERENCES api_channels(id),
    FOREIGN KEY (channel_model_id) REFERENCES api_channel_models(id)
);

CREATE INDEX IF NOT EXISTS idx_preset_channel_bindings_preset
    ON preset_channel_bindings(preset_id, enabled, sort_order);
CREATE INDEX IF NOT EXISTS idx_preset_channel_bindings_channel
    ON preset_channel_bindings(channel_id);
CREATE INDEX IF NOT EXISTS idx_preset_channel_bindings_model
    ON preset_channel_bindings(channel_model_id);
CREATE INDEX IF NOT EXISTS idx_preset_channel_bindings_default
    ON preset_channel_bindings(preset_id, binding_type, enabled, sort_order);
CREATE UNIQUE INDEX IF NOT EXISTS ux_preset_channel_bindings_preset_channel
    ON preset_channel_bindings(preset_id, channel_id)
    WHERE channel_model_id IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS ux_preset_channel_bindings_preset_channel_model
    ON preset_channel_bindings(preset_id, channel_id, channel_model_id)
    WHERE channel_model_id IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS ux_preset_channel_bindings_default
    ON preset_channel_bindings(preset_id)
    WHERE binding_type = 'default' AND enabled = 1;

CREATE TABLE IF NOT EXISTS lorebooks (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    scan_depth INTEGER NOT NULL DEFAULT 0,
    token_budget INTEGER,
    insertion_strategy TEXT NOT NULL DEFAULT 'sorted_evenly',
    enabled INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    config_json TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_lorebooks_enabled_sort
    ON lorebooks(enabled, sort_order);
CREATE INDEX IF NOT EXISTS idx_lorebooks_updated
    ON lorebooks(updated_at DESC);

CREATE TABLE IF NOT EXISTS lorebook_entries (
    id TEXT PRIMARY KEY,
    lorebook_id TEXT NOT NULL,
    title TEXT,
    primary_content_id TEXT NOT NULL,
    activation_strategy TEXT NOT NULL DEFAULT 'keyword',
    keyword_logic TEXT NOT NULL DEFAULT 'and_any',
    insertion_position TEXT NOT NULL DEFAULT 'after_char_defs',
    insertion_order INTEGER NOT NULL DEFAULT 100,
    insertion_depth INTEGER,
    insertion_role TEXT,
    outlet_name TEXT,
    entry_scope TEXT NOT NULL DEFAULT 'shared',
    enabled INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    config_json TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (lorebook_id) REFERENCES lorebooks(id),
    FOREIGN KEY (primary_content_id) REFERENCES content_objects(id)
);

CREATE INDEX IF NOT EXISTS idx_lorebook_entries_book_order
    ON lorebook_entries(lorebook_id, enabled, insertion_order, sort_order);
CREATE INDEX IF NOT EXISTS idx_lorebook_entries_position
    ON lorebook_entries(insertion_position);

CREATE TABLE IF NOT EXISTS lorebook_entry_keys (
    id TEXT PRIMARY KEY,
    entry_id TEXT NOT NULL,
    key_type TEXT NOT NULL DEFAULT 'primary',
    match_type TEXT NOT NULL DEFAULT 'plain',
    pattern_text TEXT NOT NULL,
    case_sensitive INTEGER NOT NULL DEFAULT 0,
    enabled INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    config_json TEXT NOT NULL DEFAULT '{}',
    FOREIGN KEY (entry_id) REFERENCES lorebook_entries(id)
);

CREATE INDEX IF NOT EXISTS idx_lorebook_entry_keys_entry
    ON lorebook_entry_keys(entry_id, enabled, sort_order);
CREATE INDEX IF NOT EXISTS idx_lorebook_entry_keys_pattern
    ON lorebook_entry_keys(pattern_text);

CREATE TABLE IF NOT EXISTS user_profiles (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    title TEXT,
    description_content_id TEXT,
    avatar_uri TEXT,
    injection_position TEXT NOT NULL DEFAULT 'prompt_manager',
    injection_depth INTEGER,
    injection_role TEXT,
    enabled INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    config_json TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (description_content_id) REFERENCES content_objects(id)
);

CREATE INDEX IF NOT EXISTS idx_user_profiles_enabled_sort
    ON user_profiles(enabled, sort_order);
CREATE INDEX IF NOT EXISTS idx_user_profiles_updated
    ON user_profiles(updated_at DESC);

CREATE TABLE IF NOT EXISTS agents (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    title TEXT,
    description_content_id TEXT,
    personality_content_id TEXT,
    scenario_content_id TEXT,
    example_messages_content_id TEXT,
    main_prompt_override_content_id TEXT,
    post_history_instructions_content_id TEXT,
    character_note_content_id TEXT,
    character_note_depth INTEGER,
    character_note_role TEXT,
    talkativeness INTEGER NOT NULL DEFAULT 50,
    avatar_uri TEXT,
    creator_name TEXT,
    character_version TEXT,
    creator_notes_content_id TEXT,
    enabled INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    config_json TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (description_content_id) REFERENCES content_objects(id),
    FOREIGN KEY (personality_content_id) REFERENCES content_objects(id),
    FOREIGN KEY (scenario_content_id) REFERENCES content_objects(id),
    FOREIGN KEY (example_messages_content_id) REFERENCES content_objects(id),
    FOREIGN KEY (main_prompt_override_content_id) REFERENCES content_objects(id),
    FOREIGN KEY (post_history_instructions_content_id) REFERENCES content_objects(id),
    FOREIGN KEY (character_note_content_id) REFERENCES content_objects(id),
    FOREIGN KEY (creator_notes_content_id) REFERENCES content_objects(id)
);

CREATE INDEX IF NOT EXISTS idx_agents_enabled_sort
    ON agents(enabled, sort_order);
CREATE INDEX IF NOT EXISTS idx_agents_updated
    ON agents(updated_at DESC);

CREATE TABLE IF NOT EXISTS agent_greetings (
    id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL,
    greeting_type TEXT NOT NULL DEFAULT 'alternate',
    name TEXT,
    primary_content_id TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    config_json TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (agent_id) REFERENCES agents(id),
    FOREIGN KEY (primary_content_id) REFERENCES content_objects(id)
);

CREATE INDEX IF NOT EXISTS idx_agent_greetings_agent
    ON agent_greetings(agent_id, enabled, sort_order);

CREATE TABLE IF NOT EXISTS agent_media (
    id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL,
    media_type TEXT NOT NULL,
    content_id TEXT NOT NULL,
    name TEXT,
    enabled INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    config_json TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (agent_id) REFERENCES agents(id),
    FOREIGN KEY (content_id) REFERENCES content_objects(id)
);

CREATE INDEX IF NOT EXISTS idx_agent_media_agent
    ON agent_media(agent_id, enabled, sort_order);
