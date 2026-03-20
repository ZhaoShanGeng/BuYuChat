CREATE TABLE IF NOT EXISTS summary_groups (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    scope_type TEXT NOT NULL,
    scope_message_version_id TEXT,
    scope_start_node_id TEXT,
    scope_end_node_id TEXT,
    scope_summary_group_id TEXT,
    summary_kind TEXT NOT NULL,
    default_generator_preset_id TEXT,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id),
    FOREIGN KEY (scope_message_version_id) REFERENCES message_versions(id),
    FOREIGN KEY (scope_start_node_id) REFERENCES message_nodes(id),
    FOREIGN KEY (scope_end_node_id) REFERENCES message_nodes(id),
    FOREIGN KEY (scope_summary_group_id) REFERENCES summary_groups(id),
    FOREIGN KEY (default_generator_preset_id) REFERENCES presets(id)
);

CREATE INDEX IF NOT EXISTS idx_summary_groups_conversation_kind
    ON summary_groups(conversation_id, summary_kind, enabled);

CREATE TABLE IF NOT EXISTS summary_versions (
    id TEXT PRIMARY KEY,
    summary_group_id TEXT NOT NULL,
    version_index INTEGER NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 0,
    content_id TEXT NOT NULL,
    generator_type TEXT NOT NULL DEFAULT 'manual',
    generator_preset_id TEXT,
    workflow_run_id TEXT,
    generation_run_id TEXT,
    config_json TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL,
    FOREIGN KEY (summary_group_id) REFERENCES summary_groups(id),
    FOREIGN KEY (content_id) REFERENCES content_objects(id),
    FOREIGN KEY (generator_preset_id) REFERENCES presets(id),
    FOREIGN KEY (workflow_run_id) REFERENCES workflow_runs(id),
    FOREIGN KEY (generation_run_id) REFERENCES generation_runs(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_summary_versions_group_version
    ON summary_versions(summary_group_id, version_index);
CREATE UNIQUE INDEX IF NOT EXISTS ux_summary_versions_group_active
    ON summary_versions(summary_group_id)
    WHERE is_active = 1;
CREATE INDEX IF NOT EXISTS idx_summary_versions_group_active
    ON summary_versions(summary_group_id, is_active, version_index DESC);

CREATE TABLE IF NOT EXISTS summary_sources (
    id TEXT PRIMARY KEY,
    summary_group_id TEXT NOT NULL,
    summary_version_id TEXT NOT NULL,
    source_kind TEXT NOT NULL,
    source_message_version_id TEXT,
    source_start_node_id TEXT,
    source_end_node_id TEXT,
    source_summary_version_id TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (summary_group_id) REFERENCES summary_groups(id),
    FOREIGN KEY (summary_version_id) REFERENCES summary_versions(id),
    FOREIGN KEY (source_message_version_id) REFERENCES message_versions(id),
    FOREIGN KEY (source_start_node_id) REFERENCES message_nodes(id),
    FOREIGN KEY (source_end_node_id) REFERENCES message_nodes(id),
    FOREIGN KEY (source_summary_version_id) REFERENCES summary_versions(id)
);

CREATE INDEX IF NOT EXISTS idx_summary_sources_version
    ON summary_sources(summary_version_id, sort_order);

CREATE TABLE IF NOT EXISTS summary_usages (
    id TEXT PRIMARY KEY,
    summary_group_id TEXT NOT NULL,
    summary_version_id TEXT,
    usage_scope TEXT NOT NULL,
    target_kind TEXT NOT NULL,
    target_message_version_id TEXT,
    target_start_node_id TEXT,
    target_end_node_id TEXT,
    conversation_id TEXT,
    activation_mode TEXT NOT NULL DEFAULT 'manual',
    replace_from_node_id TEXT,
    replace_after_message_count INTEGER,
    replace_after_total_bytes INTEGER,
    enabled INTEGER NOT NULL DEFAULT 1,
    priority INTEGER NOT NULL DEFAULT 0,
    config_json TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (summary_group_id) REFERENCES summary_groups(id),
    FOREIGN KEY (summary_version_id) REFERENCES summary_versions(id),
    FOREIGN KEY (target_message_version_id) REFERENCES message_versions(id),
    FOREIGN KEY (target_start_node_id) REFERENCES message_nodes(id),
    FOREIGN KEY (target_end_node_id) REFERENCES message_nodes(id),
    FOREIGN KEY (conversation_id) REFERENCES conversations(id),
    FOREIGN KEY (replace_from_node_id) REFERENCES message_nodes(id)
);

CREATE INDEX IF NOT EXISTS idx_summary_usages_scope_priority
    ON summary_usages(usage_scope, enabled, priority DESC);
CREATE INDEX IF NOT EXISTS idx_summary_usages_target_message
    ON summary_usages(target_message_version_id);
CREATE INDEX IF NOT EXISTS idx_summary_usages_target_range
    ON summary_usages(target_start_node_id, target_end_node_id);
CREATE INDEX IF NOT EXISTS idx_summary_usages_conversation
    ON summary_usages(conversation_id);
CREATE INDEX IF NOT EXISTS idx_summary_usages_replace_from_node
    ON summary_usages(replace_from_node_id);
CREATE INDEX IF NOT EXISTS idx_summary_usages_activation_mode
    ON summary_usages(activation_mode, enabled, priority DESC);

CREATE TABLE IF NOT EXISTS workflow_defs (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    enabled INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    config_json TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_workflow_defs_enabled_sort
    ON workflow_defs(enabled, sort_order);

CREATE TABLE IF NOT EXISTS workflow_def_nodes (
    id TEXT PRIMARY KEY,
    workflow_def_id TEXT NOT NULL,
    node_key TEXT NOT NULL,
    name TEXT,
    node_type TEXT NOT NULL,
    agent_id TEXT,
    plugin_id TEXT,
    preset_id TEXT,
    lorebook_id TEXT,
    user_profile_id TEXT,
    api_channel_id TEXT,
    api_channel_model_id TEXT,
    workspace_mode TEXT NOT NULL DEFAULT 'inherit',
    history_read_mode TEXT NOT NULL DEFAULT 'auto',
    summary_write_mode TEXT NOT NULL DEFAULT 'none',
    message_write_mode TEXT NOT NULL DEFAULT 'none',
    visible_output_mode TEXT NOT NULL DEFAULT 'final_only',
    config_json TEXT NOT NULL DEFAULT '{}',
    FOREIGN KEY (workflow_def_id) REFERENCES workflow_defs(id),
    FOREIGN KEY (agent_id) REFERENCES agents(id),
    FOREIGN KEY (plugin_id) REFERENCES plugin_defs(id),
    FOREIGN KEY (preset_id) REFERENCES presets(id),
    FOREIGN KEY (lorebook_id) REFERENCES lorebooks(id),
    FOREIGN KEY (user_profile_id) REFERENCES user_profiles(id),
    FOREIGN KEY (api_channel_id) REFERENCES api_channels(id),
    FOREIGN KEY (api_channel_model_id) REFERENCES api_channel_models(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_workflow_def_nodes_key
    ON workflow_def_nodes(workflow_def_id, node_key);
CREATE INDEX IF NOT EXISTS idx_workflow_def_nodes_def_type
    ON workflow_def_nodes(workflow_def_id, node_type);
CREATE INDEX IF NOT EXISTS idx_workflow_def_nodes_agent
    ON workflow_def_nodes(agent_id);
CREATE INDEX IF NOT EXISTS idx_workflow_def_nodes_plugin
    ON workflow_def_nodes(plugin_id);
CREATE INDEX IF NOT EXISTS idx_workflow_def_nodes_model
    ON workflow_def_nodes(api_channel_model_id);

CREATE TABLE IF NOT EXISTS workflow_def_edges (
    id TEXT PRIMARY KEY,
    workflow_def_id TEXT NOT NULL,
    from_node_id TEXT NOT NULL,
    to_node_id TEXT NOT NULL,
    edge_type TEXT NOT NULL DEFAULT 'next',
    priority INTEGER NOT NULL DEFAULT 0,
    condition_expr TEXT,
    label TEXT,
    enabled INTEGER NOT NULL DEFAULT 1,
    config_json TEXT NOT NULL DEFAULT '{}',
    FOREIGN KEY (workflow_def_id) REFERENCES workflow_defs(id),
    FOREIGN KEY (from_node_id) REFERENCES workflow_def_nodes(id),
    FOREIGN KEY (to_node_id) REFERENCES workflow_def_nodes(id)
);

CREATE INDEX IF NOT EXISTS idx_workflow_def_edges_from
    ON workflow_def_edges(workflow_def_id, from_node_id, enabled, priority DESC);

CREATE TABLE IF NOT EXISTS workflow_runs (
    id TEXT PRIMARY KEY,
    workflow_def_id TEXT NOT NULL,
    trigger_conversation_id TEXT,
    workspace_conversation_id TEXT,
    workspace_mode TEXT NOT NULL DEFAULT 'inherit',
    trigger_message_version_id TEXT,
    entry_node_id TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    result_message_version_id TEXT,
    request_snapshot_content_id TEXT,
    result_content_id TEXT,
    config_json TEXT NOT NULL DEFAULT '{}',
    started_at INTEGER,
    finished_at INTEGER,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (workflow_def_id) REFERENCES workflow_defs(id),
    FOREIGN KEY (trigger_conversation_id) REFERENCES conversations(id),
    FOREIGN KEY (workspace_conversation_id) REFERENCES conversations(id),
    FOREIGN KEY (trigger_message_version_id) REFERENCES message_versions(id),
    FOREIGN KEY (entry_node_id) REFERENCES workflow_def_nodes(id),
    FOREIGN KEY (result_message_version_id) REFERENCES message_versions(id),
    FOREIGN KEY (request_snapshot_content_id) REFERENCES content_objects(id),
    FOREIGN KEY (result_content_id) REFERENCES content_objects(id)
);

CREATE INDEX IF NOT EXISTS idx_workflow_runs_created
    ON workflow_runs(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_workflow_runs_status
    ON workflow_runs(status, created_at DESC);

CREATE TABLE IF NOT EXISTS workflow_run_node_executions (
    id TEXT PRIMARY KEY,
    workflow_run_id TEXT NOT NULL,
    workflow_def_node_id TEXT NOT NULL,
    parent_execution_id TEXT,
    incoming_edge_id TEXT,
    branch_key TEXT,
    loop_iteration INTEGER NOT NULL DEFAULT 0,
    retry_index INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'pending',
    generation_run_id TEXT,
    input_snapshot_content_id TEXT,
    output_content_id TEXT,
    error_content_id TEXT,
    started_at INTEGER,
    finished_at INTEGER,
    created_at INTEGER NOT NULL,
    config_json TEXT NOT NULL DEFAULT '{}',
    FOREIGN KEY (workflow_run_id) REFERENCES workflow_runs(id),
    FOREIGN KEY (workflow_def_node_id) REFERENCES workflow_def_nodes(id),
    FOREIGN KEY (parent_execution_id) REFERENCES workflow_run_node_executions(id),
    FOREIGN KEY (incoming_edge_id) REFERENCES workflow_def_edges(id),
    FOREIGN KEY (generation_run_id) REFERENCES generation_runs(id),
    FOREIGN KEY (input_snapshot_content_id) REFERENCES content_objects(id),
    FOREIGN KEY (output_content_id) REFERENCES content_objects(id),
    FOREIGN KEY (error_content_id) REFERENCES content_objects(id)
);

CREATE INDEX IF NOT EXISTS idx_workflow_run_node_executions_run
    ON workflow_run_node_executions(workflow_run_id, created_at ASC);
CREATE INDEX IF NOT EXISTS idx_workflow_run_node_executions_node
    ON workflow_run_node_executions(workflow_run_id, workflow_def_node_id, status);

CREATE TABLE IF NOT EXISTS workflow_run_writes (
    id TEXT PRIMARY KEY,
    workflow_run_id TEXT NOT NULL,
    workflow_run_node_execution_id TEXT,
    write_kind TEXT NOT NULL,
    apply_mode TEXT NOT NULL DEFAULT 'create',
    content_id TEXT NOT NULL,
    target_conversation_id TEXT,
    target_message_node_id TEXT,
    target_summary_group_id TEXT,
    target_lorebook_entry_id TEXT,
    target_preset_entry_id TEXT,
    target_agent_id TEXT,
    target_user_profile_id TEXT,
    target_plugin_id TEXT,
    target_slot TEXT,
    visible_to_user INTEGER NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL,
    config_json TEXT NOT NULL DEFAULT '{}',
    FOREIGN KEY (workflow_run_id) REFERENCES workflow_runs(id),
    FOREIGN KEY (workflow_run_node_execution_id) REFERENCES workflow_run_node_executions(id),
    FOREIGN KEY (content_id) REFERENCES content_objects(id),
    FOREIGN KEY (target_conversation_id) REFERENCES conversations(id),
    FOREIGN KEY (target_message_node_id) REFERENCES message_nodes(id),
    FOREIGN KEY (target_summary_group_id) REFERENCES summary_groups(id),
    FOREIGN KEY (target_lorebook_entry_id) REFERENCES lorebook_entries(id),
    FOREIGN KEY (target_preset_entry_id) REFERENCES preset_entries(id),
    FOREIGN KEY (target_agent_id) REFERENCES agents(id),
    FOREIGN KEY (target_user_profile_id) REFERENCES user_profiles(id),
    FOREIGN KEY (target_plugin_id) REFERENCES plugin_defs(id)
);

CREATE INDEX IF NOT EXISTS idx_workflow_run_writes_run_created
    ON workflow_run_writes(workflow_run_id, created_at);
CREATE INDEX IF NOT EXISTS idx_workflow_run_writes_node_exec
    ON workflow_run_writes(workflow_run_node_execution_id);
CREATE INDEX IF NOT EXISTS idx_workflow_run_writes_kind
    ON workflow_run_writes(write_kind);
CREATE INDEX IF NOT EXISTS idx_workflow_run_writes_visible
    ON workflow_run_writes(visible_to_user, created_at);
CREATE INDEX IF NOT EXISTS idx_workflow_run_writes_target_message
    ON workflow_run_writes(target_message_node_id);
CREATE INDEX IF NOT EXISTS idx_workflow_run_writes_target_summary_group
    ON workflow_run_writes(target_summary_group_id);
CREATE INDEX IF NOT EXISTS idx_workflow_run_writes_target_lorebook_entry
    ON workflow_run_writes(target_lorebook_entry_id);
CREATE INDEX IF NOT EXISTS idx_workflow_run_writes_target_preset_entry
    ON workflow_run_writes(target_preset_entry_id);
CREATE INDEX IF NOT EXISTS idx_workflow_run_writes_target_agent
    ON workflow_run_writes(target_agent_id);
CREATE INDEX IF NOT EXISTS idx_workflow_run_writes_target_user_profile
    ON workflow_run_writes(target_user_profile_id);
CREATE INDEX IF NOT EXISTS idx_workflow_run_writes_target_plugin
    ON workflow_run_writes(target_plugin_id);
CREATE INDEX IF NOT EXISTS idx_workflow_run_writes_target_slot
    ON workflow_run_writes(target_slot);
