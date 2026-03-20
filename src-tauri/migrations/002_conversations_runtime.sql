CREATE TABLE IF NOT EXISTS conversations (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    conversation_mode TEXT NOT NULL DEFAULT 'single',
    archived INTEGER NOT NULL DEFAULT 0,
    pinned INTEGER NOT NULL DEFAULT 0,
    config_json TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_conversations_pinned_updated
    ON conversations(pinned, updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_conversations_archived_updated
    ON conversations(archived, updated_at DESC);

CREATE TABLE IF NOT EXISTS conversation_participants (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    participant_type TEXT NOT NULL DEFAULT 'agent',
    agent_id TEXT,
    display_name TEXT,
    enabled INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    config_json TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id),
    FOREIGN KEY (agent_id) REFERENCES agents(id)
);

CREATE INDEX IF NOT EXISTS idx_conversation_participants_conversation
    ON conversation_participants(conversation_id, enabled, sort_order);

CREATE TABLE IF NOT EXISTS agent_preset_bindings (
    id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL,
    preset_id TEXT NOT NULL,
    binding_type TEXT NOT NULL DEFAULT 'available',
    enabled INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    config_json TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (agent_id) REFERENCES agents(id),
    FOREIGN KEY (preset_id) REFERENCES presets(id)
);

CREATE INDEX IF NOT EXISTS idx_agent_preset_bindings_agent
    ON agent_preset_bindings(agent_id, enabled, sort_order);
CREATE INDEX IF NOT EXISTS idx_agent_preset_bindings_preset
    ON agent_preset_bindings(preset_id);
CREATE INDEX IF NOT EXISTS idx_agent_preset_bindings_default
    ON agent_preset_bindings(agent_id, binding_type, enabled, sort_order);
CREATE UNIQUE INDEX IF NOT EXISTS ux_agent_preset_bindings_agent_preset
    ON agent_preset_bindings(agent_id, preset_id);
CREATE UNIQUE INDEX IF NOT EXISTS ux_agent_preset_bindings_default
    ON agent_preset_bindings(agent_id)
    WHERE binding_type = 'default' AND enabled = 1;

CREATE TABLE IF NOT EXISTS agent_lorebook_bindings (
    id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL,
    lorebook_id TEXT NOT NULL,
    binding_type TEXT NOT NULL DEFAULT 'available',
    enabled INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    config_json TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (agent_id) REFERENCES agents(id),
    FOREIGN KEY (lorebook_id) REFERENCES lorebooks(id)
);

CREATE INDEX IF NOT EXISTS idx_agent_lorebook_bindings_agent
    ON agent_lorebook_bindings(agent_id, enabled, sort_order);
CREATE INDEX IF NOT EXISTS idx_agent_lorebook_bindings_lorebook
    ON agent_lorebook_bindings(lorebook_id);
CREATE INDEX IF NOT EXISTS idx_agent_lorebook_bindings_default
    ON agent_lorebook_bindings(agent_id, binding_type, enabled, sort_order);
CREATE UNIQUE INDEX IF NOT EXISTS ux_agent_lorebook_bindings_agent_lorebook
    ON agent_lorebook_bindings(agent_id, lorebook_id);
CREATE UNIQUE INDEX IF NOT EXISTS ux_agent_lorebook_bindings_default
    ON agent_lorebook_bindings(agent_id)
    WHERE binding_type = 'default' AND enabled = 1;

CREATE TABLE IF NOT EXISTS agent_user_profile_bindings (
    id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL,
    user_profile_id TEXT NOT NULL,
    binding_type TEXT NOT NULL DEFAULT 'available',
    enabled INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    config_json TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (agent_id) REFERENCES agents(id),
    FOREIGN KEY (user_profile_id) REFERENCES user_profiles(id)
);

CREATE INDEX IF NOT EXISTS idx_agent_user_profile_bindings_agent
    ON agent_user_profile_bindings(agent_id, enabled, sort_order);
CREATE INDEX IF NOT EXISTS idx_agent_user_profile_bindings_user_profile
    ON agent_user_profile_bindings(user_profile_id);
CREATE INDEX IF NOT EXISTS idx_agent_user_profile_bindings_default
    ON agent_user_profile_bindings(agent_id, binding_type, enabled, sort_order);
CREATE UNIQUE INDEX IF NOT EXISTS ux_agent_user_profile_bindings_agent_user_profile
    ON agent_user_profile_bindings(agent_id, user_profile_id);
CREATE UNIQUE INDEX IF NOT EXISTS ux_agent_user_profile_bindings_default
    ON agent_user_profile_bindings(agent_id)
    WHERE binding_type = 'default' AND enabled = 1;

CREATE TABLE IF NOT EXISTS agent_channel_bindings (
    id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL,
    channel_id TEXT NOT NULL,
    channel_model_id TEXT,
    binding_type TEXT NOT NULL DEFAULT 'available',
    enabled INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    config_json TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (agent_id) REFERENCES agents(id),
    FOREIGN KEY (channel_id) REFERENCES api_channels(id),
    FOREIGN KEY (channel_model_id) REFERENCES api_channel_models(id)
);

CREATE INDEX IF NOT EXISTS idx_agent_channel_bindings_agent
    ON agent_channel_bindings(agent_id, enabled, sort_order);
CREATE INDEX IF NOT EXISTS idx_agent_channel_bindings_channel
    ON agent_channel_bindings(channel_id);
CREATE INDEX IF NOT EXISTS idx_agent_channel_bindings_model
    ON agent_channel_bindings(channel_model_id);
CREATE INDEX IF NOT EXISTS idx_agent_channel_bindings_default
    ON agent_channel_bindings(agent_id, binding_type, enabled, sort_order);
CREATE UNIQUE INDEX IF NOT EXISTS ux_agent_channel_bindings_agent_channel
    ON agent_channel_bindings(agent_id, channel_id)
    WHERE channel_model_id IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS ux_agent_channel_bindings_agent_channel_model
    ON agent_channel_bindings(agent_id, channel_id, channel_model_id)
    WHERE channel_model_id IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS ux_agent_channel_bindings_default
    ON agent_channel_bindings(agent_id)
    WHERE binding_type = 'default' AND enabled = 1;

CREATE TABLE IF NOT EXISTS conversation_preset_bindings (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    preset_id TEXT NOT NULL,
    binding_type TEXT NOT NULL DEFAULT 'available',
    enabled INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    config_json TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id),
    FOREIGN KEY (preset_id) REFERENCES presets(id)
);

CREATE INDEX IF NOT EXISTS idx_conversation_preset_bindings_conversation
    ON conversation_preset_bindings(conversation_id, enabled, sort_order);
CREATE INDEX IF NOT EXISTS idx_conversation_preset_bindings_preset
    ON conversation_preset_bindings(preset_id);
CREATE INDEX IF NOT EXISTS idx_conversation_preset_bindings_active
    ON conversation_preset_bindings(conversation_id, binding_type, enabled, sort_order);
CREATE UNIQUE INDEX IF NOT EXISTS ux_conversation_preset_bindings_conversation_preset
    ON conversation_preset_bindings(conversation_id, preset_id);
CREATE UNIQUE INDEX IF NOT EXISTS ux_conversation_preset_bindings_active
    ON conversation_preset_bindings(conversation_id)
    WHERE binding_type = 'active' AND enabled = 1;

CREATE TABLE IF NOT EXISTS conversation_lorebook_bindings (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    lorebook_id TEXT NOT NULL,
    binding_type TEXT NOT NULL DEFAULT 'available',
    enabled INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    config_json TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id),
    FOREIGN KEY (lorebook_id) REFERENCES lorebooks(id)
);

CREATE INDEX IF NOT EXISTS idx_conversation_lorebook_bindings_conversation
    ON conversation_lorebook_bindings(conversation_id, enabled, sort_order);
CREATE INDEX IF NOT EXISTS idx_conversation_lorebook_bindings_lorebook
    ON conversation_lorebook_bindings(lorebook_id);
CREATE INDEX IF NOT EXISTS idx_conversation_lorebook_bindings_active
    ON conversation_lorebook_bindings(conversation_id, binding_type, enabled, sort_order);
CREATE UNIQUE INDEX IF NOT EXISTS ux_conversation_lorebook_bindings_conversation_lorebook
    ON conversation_lorebook_bindings(conversation_id, lorebook_id);
CREATE UNIQUE INDEX IF NOT EXISTS ux_conversation_lorebook_bindings_active
    ON conversation_lorebook_bindings(conversation_id)
    WHERE binding_type = 'active' AND enabled = 1;

CREATE TABLE IF NOT EXISTS conversation_user_profile_bindings (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    user_profile_id TEXT NOT NULL,
    binding_type TEXT NOT NULL DEFAULT 'available',
    enabled INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    config_json TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id),
    FOREIGN KEY (user_profile_id) REFERENCES user_profiles(id)
);

CREATE INDEX IF NOT EXISTS idx_conversation_user_profile_bindings_conversation
    ON conversation_user_profile_bindings(conversation_id, enabled, sort_order);
CREATE INDEX IF NOT EXISTS idx_conversation_user_profile_bindings_user_profile
    ON conversation_user_profile_bindings(user_profile_id);
CREATE INDEX IF NOT EXISTS idx_conversation_user_profile_bindings_active
    ON conversation_user_profile_bindings(conversation_id, binding_type, enabled, sort_order);
CREATE UNIQUE INDEX IF NOT EXISTS ux_conversation_user_profile_bindings_conversation_user_profile
    ON conversation_user_profile_bindings(conversation_id, user_profile_id);
CREATE UNIQUE INDEX IF NOT EXISTS ux_conversation_user_profile_bindings_active
    ON conversation_user_profile_bindings(conversation_id)
    WHERE binding_type = 'active' AND enabled = 1;

CREATE TABLE IF NOT EXISTS conversation_channel_bindings (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    channel_id TEXT NOT NULL,
    channel_model_id TEXT,
    binding_type TEXT NOT NULL DEFAULT 'available',
    enabled INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    config_json TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id),
    FOREIGN KEY (channel_id) REFERENCES api_channels(id),
    FOREIGN KEY (channel_model_id) REFERENCES api_channel_models(id)
);

CREATE INDEX IF NOT EXISTS idx_conversation_channel_bindings_conversation
    ON conversation_channel_bindings(conversation_id, enabled, sort_order);
CREATE INDEX IF NOT EXISTS idx_conversation_channel_bindings_channel
    ON conversation_channel_bindings(channel_id);
CREATE INDEX IF NOT EXISTS idx_conversation_channel_bindings_model
    ON conversation_channel_bindings(channel_model_id);
CREATE INDEX IF NOT EXISTS idx_conversation_channel_bindings_active
    ON conversation_channel_bindings(conversation_id, binding_type, enabled, sort_order);
CREATE UNIQUE INDEX IF NOT EXISTS ux_conversation_channel_bindings_conversation_channel
    ON conversation_channel_bindings(conversation_id, channel_id)
    WHERE channel_model_id IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS ux_conversation_channel_bindings_conversation_channel_model
    ON conversation_channel_bindings(conversation_id, channel_id, channel_model_id)
    WHERE channel_model_id IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS ux_conversation_channel_bindings_active
    ON conversation_channel_bindings(conversation_id)
    WHERE binding_type = 'active' AND enabled = 1;

CREATE TABLE IF NOT EXISTS message_nodes (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    author_participant_id TEXT NOT NULL,
    role TEXT NOT NULL,
    reply_to_node_id TEXT,
    order_key TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id),
    FOREIGN KEY (author_participant_id) REFERENCES conversation_participants(id),
    FOREIGN KEY (reply_to_node_id) REFERENCES message_nodes(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_message_nodes_conversation_order
    ON message_nodes(conversation_id, order_key);
CREATE INDEX IF NOT EXISTS idx_message_nodes_author
    ON message_nodes(author_participant_id);
CREATE INDEX IF NOT EXISTS idx_message_nodes_reply_to
    ON message_nodes(reply_to_node_id);

CREATE TABLE IF NOT EXISTS message_versions (
    id TEXT PRIMARY KEY,
    node_id TEXT NOT NULL,
    version_index INTEGER NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 0,
    primary_content_id TEXT NOT NULL,
    context_policy TEXT NOT NULL DEFAULT 'full',
    viewer_policy TEXT NOT NULL DEFAULT 'full',
    api_channel_id TEXT,
    api_channel_model_id TEXT,
    generation_run_id TEXT,
    prompt_tokens INTEGER,
    completion_tokens INTEGER,
    total_tokens INTEGER,
    finish_reason TEXT,
    config_json TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL,
    FOREIGN KEY (node_id) REFERENCES message_nodes(id),
    FOREIGN KEY (primary_content_id) REFERENCES content_objects(id),
    FOREIGN KEY (api_channel_id) REFERENCES api_channels(id),
    FOREIGN KEY (api_channel_model_id) REFERENCES api_channel_models(id),
    FOREIGN KEY (generation_run_id) REFERENCES generation_runs(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_message_versions_node_version
    ON message_versions(node_id, version_index);
CREATE UNIQUE INDEX IF NOT EXISTS ux_message_versions_node_active
    ON message_versions(node_id)
    WHERE is_active = 1;
CREATE INDEX IF NOT EXISTS idx_message_versions_node_active
    ON message_versions(node_id, is_active, version_index DESC);
CREATE INDEX IF NOT EXISTS idx_message_versions_model
    ON message_versions(api_channel_model_id);

CREATE TABLE IF NOT EXISTS message_version_content_refs (
    id TEXT PRIMARY KEY,
    message_version_id TEXT NOT NULL,
    content_id TEXT NOT NULL,
    plugin_id TEXT,
    ref_role TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    config_json TEXT NOT NULL DEFAULT '{}',
    FOREIGN KEY (message_version_id) REFERENCES message_versions(id),
    FOREIGN KEY (content_id) REFERENCES content_objects(id),
    FOREIGN KEY (plugin_id) REFERENCES plugin_defs(id)
);

CREATE INDEX IF NOT EXISTS idx_message_version_content_refs_version
    ON message_version_content_refs(message_version_id, sort_order);
CREATE INDEX IF NOT EXISTS idx_message_version_content_refs_role
    ON message_version_content_refs(ref_role);
CREATE INDEX IF NOT EXISTS idx_message_version_content_refs_plugin
    ON message_version_content_refs(plugin_id, sort_order);

CREATE TABLE IF NOT EXISTS generation_runs (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    trigger_node_id TEXT,
    trigger_message_version_id TEXT,
    responder_participant_id TEXT,
    api_channel_id TEXT,
    api_channel_model_id TEXT,
    preset_id TEXT,
    preset_source_scope TEXT,
    lorebook_id TEXT,
    lorebook_source_scope TEXT,
    user_profile_id TEXT,
    user_profile_source_scope TEXT,
    api_channel_source_scope TEXT,
    api_channel_model_source_scope TEXT,
    run_type TEXT NOT NULL DEFAULT 'chat',
    status TEXT NOT NULL DEFAULT 'pending',
    request_parameters_json TEXT NOT NULL DEFAULT '{}',
    request_payload_content_id TEXT,
    response_payload_content_id TEXT,
    error_text TEXT,
    started_at INTEGER,
    finished_at INTEGER,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id),
    FOREIGN KEY (trigger_node_id) REFERENCES message_nodes(id),
    FOREIGN KEY (trigger_message_version_id) REFERENCES message_versions(id),
    FOREIGN KEY (responder_participant_id) REFERENCES conversation_participants(id),
    FOREIGN KEY (api_channel_id) REFERENCES api_channels(id),
    FOREIGN KEY (api_channel_model_id) REFERENCES api_channel_models(id),
    FOREIGN KEY (preset_id) REFERENCES presets(id),
    FOREIGN KEY (lorebook_id) REFERENCES lorebooks(id),
    FOREIGN KEY (user_profile_id) REFERENCES user_profiles(id),
    FOREIGN KEY (request_payload_content_id) REFERENCES content_objects(id),
    FOREIGN KEY (response_payload_content_id) REFERENCES content_objects(id)
);

CREATE INDEX IF NOT EXISTS idx_generation_runs_conversation_created
    ON generation_runs(conversation_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_generation_runs_status
    ON generation_runs(status, created_at DESC);

CREATE TABLE IF NOT EXISTS generation_run_context_items (
    id TEXT PRIMARY KEY,
    generation_run_id TEXT NOT NULL,
    sequence_no INTEGER NOT NULL,
    send_role TEXT NOT NULL,
    rendered_content_id TEXT NOT NULL,
    source_kind TEXT NOT NULL,
    source_message_node_id TEXT,
    source_message_version_id TEXT,
    source_summary_version_id TEXT,
    source_preset_entry_id TEXT,
    source_lorebook_entry_id TEXT,
    source_user_profile_id TEXT,
    source_agent_id TEXT,
    source_agent_greeting_id TEXT,
    source_tool_invocation_id TEXT,
    source_rag_ref_id TEXT,
    source_mcp_event_id TEXT,
    source_plugin_id TEXT,
    included_in_request INTEGER NOT NULL DEFAULT 1,
    config_json TEXT NOT NULL DEFAULT '{}',
    FOREIGN KEY (generation_run_id) REFERENCES generation_runs(id),
    FOREIGN KEY (rendered_content_id) REFERENCES content_objects(id),
    FOREIGN KEY (source_message_node_id) REFERENCES message_nodes(id),
    FOREIGN KEY (source_message_version_id) REFERENCES message_versions(id),
    FOREIGN KEY (source_summary_version_id) REFERENCES summary_versions(id),
    FOREIGN KEY (source_preset_entry_id) REFERENCES preset_entries(id),
    FOREIGN KEY (source_lorebook_entry_id) REFERENCES lorebook_entries(id),
    FOREIGN KEY (source_user_profile_id) REFERENCES user_profiles(id),
    FOREIGN KEY (source_agent_id) REFERENCES agents(id),
    FOREIGN KEY (source_agent_greeting_id) REFERENCES agent_greetings(id),
    FOREIGN KEY (source_tool_invocation_id) REFERENCES tool_invocations(id),
    FOREIGN KEY (source_rag_ref_id) REFERENCES rag_refs(id),
    FOREIGN KEY (source_mcp_event_id) REFERENCES mcp_events(id),
    FOREIGN KEY (source_plugin_id) REFERENCES plugin_defs(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_generation_run_context_items_run_seq
    ON generation_run_context_items(generation_run_id, sequence_no);
CREATE INDEX IF NOT EXISTS idx_generation_run_context_items_kind
    ON generation_run_context_items(source_kind);
CREATE INDEX IF NOT EXISTS idx_generation_run_context_items_message_version
    ON generation_run_context_items(source_message_version_id);
CREATE INDEX IF NOT EXISTS idx_generation_run_context_items_summary_version
    ON generation_run_context_items(source_summary_version_id);
CREATE INDEX IF NOT EXISTS idx_generation_run_context_items_preset_entry
    ON generation_run_context_items(source_preset_entry_id);
CREATE INDEX IF NOT EXISTS idx_generation_run_context_items_lorebook_entry
    ON generation_run_context_items(source_lorebook_entry_id);
CREATE INDEX IF NOT EXISTS idx_generation_run_context_items_user_profile
    ON generation_run_context_items(source_user_profile_id);
CREATE INDEX IF NOT EXISTS idx_generation_run_context_items_agent
    ON generation_run_context_items(source_agent_id);
CREATE INDEX IF NOT EXISTS idx_generation_run_context_items_agent_greeting
    ON generation_run_context_items(source_agent_greeting_id);
CREATE INDEX IF NOT EXISTS idx_generation_run_context_items_tool_invocation
    ON generation_run_context_items(source_tool_invocation_id);
CREATE INDEX IF NOT EXISTS idx_generation_run_context_items_rag_ref
    ON generation_run_context_items(source_rag_ref_id);
CREATE INDEX IF NOT EXISTS idx_generation_run_context_items_mcp_event
    ON generation_run_context_items(source_mcp_event_id);
CREATE INDEX IF NOT EXISTS idx_generation_run_context_items_plugin
    ON generation_run_context_items(source_plugin_id);
CREATE INDEX IF NOT EXISTS idx_generation_run_context_items_included
    ON generation_run_context_items(generation_run_id, included_in_request, sequence_no);
