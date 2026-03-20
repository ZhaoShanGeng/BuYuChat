CREATE TABLE IF NOT EXISTS plugin_defs (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    plugin_key TEXT NOT NULL,
    version TEXT NOT NULL,
    runtime_kind TEXT NOT NULL,
    entrypoint TEXT,
    capabilities_json TEXT NOT NULL DEFAULT '{}',
    permissions_json TEXT NOT NULL DEFAULT '{}',
    enabled INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    config_json TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_plugin_defs_key
    ON plugin_defs(plugin_key);
CREATE INDEX IF NOT EXISTS idx_plugin_defs_enabled_sort
    ON plugin_defs(enabled, sort_order);
CREATE UNIQUE INDEX IF NOT EXISTS ux_plugin_defs_key
    ON plugin_defs(plugin_key);

CREATE TABLE IF NOT EXISTS tool_invocations (
    id TEXT PRIMARY KEY,
    generation_run_id TEXT,
    workflow_run_node_execution_id TEXT,
    message_version_id TEXT,
    tool_kind TEXT NOT NULL,
    tool_name TEXT NOT NULL,
    plugin_id TEXT,
    request_content_id TEXT,
    response_content_id TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    started_at INTEGER,
    finished_at INTEGER,
    created_at INTEGER NOT NULL,
    config_json TEXT NOT NULL DEFAULT '{}',
    FOREIGN KEY (generation_run_id) REFERENCES generation_runs(id),
    FOREIGN KEY (workflow_run_node_execution_id) REFERENCES workflow_run_node_executions(id),
    FOREIGN KEY (message_version_id) REFERENCES message_versions(id),
    FOREIGN KEY (plugin_id) REFERENCES plugin_defs(id),
    FOREIGN KEY (request_content_id) REFERENCES content_objects(id),
    FOREIGN KEY (response_content_id) REFERENCES content_objects(id)
);

CREATE INDEX IF NOT EXISTS idx_tool_invocations_run
    ON tool_invocations(generation_run_id, created_at);
CREATE INDEX IF NOT EXISTS idx_tool_invocations_workflow_exec
    ON tool_invocations(workflow_run_node_execution_id);
CREATE INDEX IF NOT EXISTS idx_tool_invocations_message
    ON tool_invocations(message_version_id);
CREATE INDEX IF NOT EXISTS idx_tool_invocations_tool
    ON tool_invocations(tool_name, status);

CREATE TABLE IF NOT EXISTS rag_refs (
    id TEXT PRIMARY KEY,
    generation_run_id TEXT,
    workflow_run_node_execution_id TEXT,
    source_uri TEXT,
    document_title TEXT,
    chunk_key TEXT,
    score REAL,
    excerpt_content_id TEXT,
    included_in_request INTEGER NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL,
    config_json TEXT NOT NULL DEFAULT '{}',
    FOREIGN KEY (generation_run_id) REFERENCES generation_runs(id),
    FOREIGN KEY (workflow_run_node_execution_id) REFERENCES workflow_run_node_executions(id),
    FOREIGN KEY (excerpt_content_id) REFERENCES content_objects(id)
);

CREATE INDEX IF NOT EXISTS idx_rag_refs_run
    ON rag_refs(generation_run_id, created_at);
CREATE INDEX IF NOT EXISTS idx_rag_refs_workflow_exec
    ON rag_refs(workflow_run_node_execution_id);
CREATE INDEX IF NOT EXISTS idx_rag_refs_chunk_key
    ON rag_refs(chunk_key);
CREATE INDEX IF NOT EXISTS idx_rag_refs_included
    ON rag_refs(generation_run_id, included_in_request);

CREATE TABLE IF NOT EXISTS mcp_events (
    id TEXT PRIMARY KEY,
    generation_run_id TEXT,
    workflow_run_node_execution_id TEXT,
    server_name TEXT NOT NULL,
    event_kind TEXT NOT NULL,
    method_name TEXT,
    payload_content_id TEXT,
    status TEXT NOT NULL DEFAULT 'ok',
    created_at INTEGER NOT NULL,
    config_json TEXT NOT NULL DEFAULT '{}',
    FOREIGN KEY (generation_run_id) REFERENCES generation_runs(id),
    FOREIGN KEY (workflow_run_node_execution_id) REFERENCES workflow_run_node_executions(id),
    FOREIGN KEY (payload_content_id) REFERENCES content_objects(id)
);

CREATE INDEX IF NOT EXISTS idx_mcp_events_run
    ON mcp_events(generation_run_id, created_at);
CREATE INDEX IF NOT EXISTS idx_mcp_events_workflow_exec
    ON mcp_events(workflow_run_node_execution_id);
CREATE INDEX IF NOT EXISTS idx_mcp_events_server_kind
    ON mcp_events(server_name, event_kind, created_at);

CREATE TABLE IF NOT EXISTS variable_defs (
    id TEXT PRIMARY KEY,
    var_key TEXT NOT NULL,
    name TEXT NOT NULL,
    value_type TEXT NOT NULL,
    scope_type TEXT NOT NULL,
    namespace TEXT NOT NULL DEFAULT 'default',
    is_user_editable INTEGER NOT NULL DEFAULT 1,
    is_plugin_editable INTEGER NOT NULL DEFAULT 0,
    ai_can_create INTEGER NOT NULL DEFAULT 0,
    ai_can_update INTEGER NOT NULL DEFAULT 0,
    ai_can_delete INTEGER NOT NULL DEFAULT 0,
    ai_can_lock INTEGER NOT NULL DEFAULT 0,
    ai_can_unlock_own_lock INTEGER NOT NULL DEFAULT 0,
    ai_can_unlock_any_lock INTEGER NOT NULL DEFAULT 0,
    default_json TEXT NOT NULL DEFAULT 'null',
    config_json TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_variable_defs_scope
    ON variable_defs(scope_type, namespace);
CREATE INDEX IF NOT EXISTS idx_variable_defs_namespace
    ON variable_defs(namespace, name);
CREATE UNIQUE INDEX IF NOT EXISTS ux_variable_defs_var_key
    ON variable_defs(var_key);

CREATE TABLE IF NOT EXISTS variable_values (
    id TEXT PRIMARY KEY,
    variable_def_id TEXT NOT NULL,
    scope_type TEXT NOT NULL,
    scope_id TEXT NOT NULL,
    value_json TEXT NOT NULL DEFAULT 'null',
    value_content_id TEXT,
    source_message_version_id TEXT,
    updated_by_kind TEXT NOT NULL,
    updated_by_ref_id TEXT,
    event_no INTEGER NOT NULL DEFAULT 1,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (variable_def_id) REFERENCES variable_defs(id),
    FOREIGN KEY (value_content_id) REFERENCES content_objects(id),
    FOREIGN KEY (source_message_version_id) REFERENCES message_versions(id)
);

CREATE INDEX IF NOT EXISTS idx_variable_values_scope
    ON variable_values(scope_type, scope_id);
CREATE INDEX IF NOT EXISTS idx_variable_values_def_scope
    ON variable_values(variable_def_id, scope_type, scope_id);
CREATE INDEX IF NOT EXISTS idx_variable_values_updated
    ON variable_values(updated_at DESC);
CREATE UNIQUE INDEX IF NOT EXISTS ux_variable_values_def_scope
    ON variable_values(variable_def_id, scope_type, scope_id);

CREATE TABLE IF NOT EXISTS variable_events (
    id TEXT PRIMARY KEY,
    variable_value_id TEXT NOT NULL,
    event_no INTEGER NOT NULL,
    event_kind TEXT NOT NULL DEFAULT 'set',
    value_json TEXT NOT NULL DEFAULT 'null',
    value_content_id TEXT,
    source_message_version_id TEXT,
    updated_by_kind TEXT NOT NULL,
    updated_by_ref_id TEXT,
    created_at INTEGER NOT NULL,
    config_json TEXT NOT NULL DEFAULT '{}',
    FOREIGN KEY (variable_value_id) REFERENCES variable_values(id),
    FOREIGN KEY (value_content_id) REFERENCES content_objects(id),
    FOREIGN KEY (source_message_version_id) REFERENCES message_versions(id)
);

CREATE INDEX IF NOT EXISTS idx_variable_events_value
    ON variable_events(variable_value_id, event_no DESC);
CREATE INDEX IF NOT EXISTS idx_variable_events_source_message
    ON variable_events(source_message_version_id);
CREATE UNIQUE INDEX IF NOT EXISTS ux_variable_events_value_event
    ON variable_events(variable_value_id, event_no);

CREATE TABLE IF NOT EXISTS variable_locks (
    id TEXT PRIMARY KEY,
    variable_value_id TEXT NOT NULL,
    lock_kind TEXT NOT NULL,
    owner_kind TEXT NOT NULL,
    owner_ref_id TEXT,
    unlock_policy TEXT NOT NULL DEFAULT 'owner',
    active INTEGER NOT NULL DEFAULT 1,
    config_json TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (variable_value_id) REFERENCES variable_values(id)
);

CREATE INDEX IF NOT EXISTS idx_variable_locks_value
    ON variable_locks(variable_value_id, active, lock_kind);
CREATE UNIQUE INDEX IF NOT EXISTS ux_variable_locks_active_kind
    ON variable_locks(variable_value_id, lock_kind)
    WHERE active = 1;

CREATE TABLE IF NOT EXISTS transform_pipelines (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    pipeline_key TEXT NOT NULL,
    pipeline_kind TEXT NOT NULL,
    description_content_id TEXT,
    enabled INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    config_json TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (description_content_id) REFERENCES content_objects(id)
);

CREATE INDEX IF NOT EXISTS idx_transform_pipelines_key
    ON transform_pipelines(pipeline_key);
CREATE INDEX IF NOT EXISTS idx_transform_pipelines_enabled_sort
    ON transform_pipelines(enabled, sort_order);
CREATE UNIQUE INDEX IF NOT EXISTS ux_transform_pipelines_key
    ON transform_pipelines(pipeline_key);

CREATE TABLE IF NOT EXISTS transform_steps (
    id TEXT PRIMARY KEY,
    pipeline_id TEXT NOT NULL,
    step_order INTEGER NOT NULL,
    step_type TEXT NOT NULL,
    pattern TEXT,
    replacement_template TEXT,
    regex_flags TEXT NOT NULL DEFAULT '',
    max_replacements INTEGER,
    stop_on_match INTEGER NOT NULL DEFAULT 0,
    child_pipeline_id TEXT,
    config_json TEXT NOT NULL DEFAULT '{}',
    FOREIGN KEY (pipeline_id) REFERENCES transform_pipelines(id),
    FOREIGN KEY (child_pipeline_id) REFERENCES transform_pipelines(id)
);

CREATE INDEX IF NOT EXISTS idx_transform_steps_pipeline_order
    ON transform_steps(pipeline_id, step_order);
CREATE INDEX IF NOT EXISTS idx_transform_steps_child_pipeline
    ON transform_steps(child_pipeline_id);
CREATE UNIQUE INDEX IF NOT EXISTS ux_transform_steps_pipeline_order
    ON transform_steps(pipeline_id, step_order);

CREATE TABLE IF NOT EXISTS transform_bindings (
    id TEXT PRIMARY KEY,
    pipeline_id TEXT NOT NULL,
    conversation_id TEXT,
    agent_id TEXT,
    preset_id TEXT,
    workflow_def_node_id TEXT,
    apply_viewer INTEGER NOT NULL DEFAULT 0,
    apply_request INTEGER NOT NULL DEFAULT 0,
    apply_file INTEGER NOT NULL DEFAULT 0,
    enabled INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    config_json TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (pipeline_id) REFERENCES transform_pipelines(id),
    FOREIGN KEY (conversation_id) REFERENCES conversations(id),
    FOREIGN KEY (agent_id) REFERENCES agents(id),
    FOREIGN KEY (preset_id) REFERENCES presets(id),
    FOREIGN KEY (workflow_def_node_id) REFERENCES workflow_def_nodes(id)
);

CREATE INDEX IF NOT EXISTS idx_transform_bindings_pipeline
    ON transform_bindings(pipeline_id, sort_order);
CREATE INDEX IF NOT EXISTS idx_transform_bindings_conversation
    ON transform_bindings(conversation_id, enabled, sort_order);
CREATE INDEX IF NOT EXISTS idx_transform_bindings_agent
    ON transform_bindings(agent_id, enabled, sort_order);
CREATE INDEX IF NOT EXISTS idx_transform_bindings_preset
    ON transform_bindings(preset_id, enabled, sort_order);
CREATE INDEX IF NOT EXISTS idx_transform_bindings_workflow_node
    ON transform_bindings(workflow_def_node_id, enabled, sort_order);
CREATE UNIQUE INDEX IF NOT EXISTS ux_transform_bindings_conversation_pipeline
    ON transform_bindings(conversation_id, pipeline_id)
    WHERE conversation_id IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS ux_transform_bindings_agent_pipeline
    ON transform_bindings(agent_id, pipeline_id)
    WHERE agent_id IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS ux_transform_bindings_preset_pipeline
    ON transform_bindings(preset_id, pipeline_id)
    WHERE preset_id IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS ux_transform_bindings_workflow_node_pipeline
    ON transform_bindings(workflow_def_node_id, pipeline_id)
    WHERE workflow_def_node_id IS NOT NULL;
