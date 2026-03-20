use std::{env, fs, path::PathBuf};

use buyu_lib::db::{
    pool,
    repos::{messages as message_repo, workflows as workflow_repo},
};
use buyu_lib::domain::content::{ContentType, ContentWriteInput};
use buyu_lib::domain::conversations::CreateConversationInput;
use buyu_lib::domain::native_capabilities::{
    CreateMcpEventInput, CreateRagRefInput, FinishToolInvocationInput, StartToolInvocationInput,
};
use buyu_lib::domain::plugins::{CreatePluginInput, UpdatePluginInput};
use buyu_lib::extensions::runtime::PluginRuntime;
use buyu_lib::services::{content_store, conversations, mcp, plugins, rag, tool_invocations};
use serde_json::json;

struct TestEnv {
    _root: PathBuf,
    db: sqlx::SqlitePool,
    store: content_store::ContentStore,
    runtime: PluginRuntime,
}

impl TestEnv {
    async fn new() -> Self {
        let root = env::temp_dir()
            .join("buyu-tests")
            .join(uuid::Uuid::now_v7().to_string());
        fs::create_dir_all(&root).expect("failed to create test root");

        let db_path = root.join("test.db");
        let db = pool::init_pool(&db_path).await.expect("failed to init db");
        let store =
            content_store::ContentStore::new(root.join("storage").join("profiles").join("test"));
        store
            .ensure_layout()
            .expect("failed to init content store layout");

        Self {
            _root: root,
            db,
            store,
            runtime: PluginRuntime::new(),
        }
    }
}

#[tokio::test]
async fn plugin_crud_and_runtime_sync() {
    let env = TestEnv::new().await;

    let created = plugins::create_plugin(
        &env.db,
        &env.runtime,
        &CreatePluginInput {
            name: "Memory Table".to_string(),
            plugin_key: "memory.table".to_string(),
            version: "0.1.0".to_string(),
            runtime_kind: "native".to_string(),
            entrypoint: Some("builtin://memory.table".to_string()),
            enabled: true,
            sort_order: 10,
            capabilities_json: json!({
                "message_block": true,
                "workflow_node": true,
            }),
            permissions_json: json!({
                "filesystem": false,
                "network": false,
            }),
            config_json: json!({"mode": "test"}),
        },
    )
    .await
    .expect("failed to create plugin");

    assert!(env.runtime.get_plugin(&created.id).is_some());
    let by_capability = plugins::list_plugins_by_capability(&env.db, "workflow_node")
        .await
        .expect("failed to list plugins by capability");
    assert_eq!(by_capability.len(), 1);
    assert_eq!(by_capability[0].id, created.id);

    let updated = plugins::update_plugin(
        &env.db,
        &env.runtime,
        &created.id,
        &UpdatePluginInput {
            name: "Memory Table".to_string(),
            version: "0.2.0".to_string(),
            runtime_kind: "native".to_string(),
            entrypoint: Some("builtin://memory.table".to_string()),
            enabled: false,
            sort_order: 20,
            capabilities_json: json!({
                "message_block": true,
            }),
            permissions_json: json!({
                "filesystem": false,
            }),
            config_json: json!({"mode": "updated"}),
        },
    )
    .await
    .expect("failed to update plugin");

    assert!(!updated.enabled);
    assert!(env.runtime.get_plugin(&created.id).is_none());

    plugins::delete_plugin(&env.db, &env.runtime, &created.id)
        .await
        .expect("failed to delete plugin");
    let listed = plugins::list_plugins(&env.db)
        .await
        .expect("failed to list plugins");
    assert!(listed.is_empty());
}

#[tokio::test]
async fn native_capabilities_generation_run_round_trip() {
    let env = TestEnv::new().await;

    let plugin = plugins::create_plugin(
        &env.db,
        &env.runtime,
        &CreatePluginInput {
            name: "Audit Tool".to_string(),
            plugin_key: "audit.tool".to_string(),
            version: "1.0.0".to_string(),
            runtime_kind: "native".to_string(),
            entrypoint: Some("builtin://audit.tool".to_string()),
            enabled: true,
            sort_order: 0,
            capabilities_json: json!({"tool": true}),
            permissions_json: json!({}),
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create plugin");

    let conversation = conversations::create_conversation(
        &env.db,
        &CreateConversationInput {
            title: "Audit Conversation".to_string(),
            description: None,
            conversation_mode: "single".to_string(),
            archived: false,
            pinned: false,
            participants: Vec::new(),
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create conversation");

    let generation_run = message_repo::create_generation_run(
        &env.db,
        &message_repo::CreateGenerationRunRecord {
            conversation_id: &conversation.summary.id,
            trigger_node_id: None,
            trigger_message_version_id: None,
            responder_participant_id: None,
            api_channel_id: None,
            api_channel_model_id: None,
            preset_id: None,
            preset_source_scope: None,
            lorebook_id: None,
            lorebook_source_scope: None,
            user_profile_id: None,
            user_profile_source_scope: None,
            api_channel_source_scope: None,
            api_channel_model_source_scope: None,
            run_type: "chat",
            request_parameters_json: "{}",
            request_payload_content_id: None,
        },
    )
    .await
    .expect("failed to create generation run");

    let tool = tool_invocations::start_tool_invocation(
        &env.db,
        &env.store,
        &StartToolInvocationInput {
            generation_run_id: Some(generation_run.id.clone()),
            workflow_run_node_execution_id: None,
            message_version_id: None,
            tool_kind: "plugin".to_string(),
            tool_name: "memory_lookup".to_string(),
            plugin_id: Some(plugin.id.clone()),
            request_content: Some(text_input("lookup request", ContentType::ToolRequest)),
            config_json: json!({"source": "test"}),
        },
    )
    .await
    .expect("failed to start tool invocation");
    assert_eq!(tool.status, "running");

    let finished_tool = tool_invocations::finish_tool_invocation(
        &env.db,
        &env.store,
        &tool.id,
        &FinishToolInvocationInput {
            status: "succeeded".to_string(),
            response_content: Some(text_input("lookup response", ContentType::ToolResponse)),
            config_json: json!({"cached": true}),
        },
    )
    .await
    .expect("failed to finish tool invocation");
    assert_eq!(finished_tool.status, "succeeded");

    let rag_ref = rag::record_rag_ref(
        &env.db,
        &env.store,
        &CreateRagRefInput {
            generation_run_id: Some(generation_run.id.clone()),
            workflow_run_node_execution_id: None,
            source_uri: Some("memory://chunk/1".to_string()),
            document_title: Some("Memory".to_string()),
            chunk_key: Some("chunk-1".to_string()),
            score: Some(0.98),
            excerpt_content: Some(text_input("remember this", ContentType::RagExcerpt)),
            included_in_request: true,
            config_json: json!({"rank": 1}),
        },
    )
    .await
    .expect("failed to record rag ref");
    assert_eq!(rag_ref.chunk_key.as_deref(), Some("chunk-1"));

    let mcp_event = mcp::record_mcp_event(
        &env.db,
        &env.store,
        &CreateMcpEventInput {
            generation_run_id: Some(generation_run.id.clone()),
            workflow_run_node_execution_id: None,
            server_name: "memory-server".to_string(),
            event_kind: "request".to_string(),
            method_name: Some("memory.lookup".to_string()),
            payload_content: Some(text_input("{\"key\":\"topic\"}", ContentType::McpPayload)),
            status: "ok".to_string(),
            config_json: json!({"request_id": "req-1"}),
        },
    )
    .await
    .expect("failed to record mcp event");
    assert_eq!(mcp_event.server_name, "memory-server");

    let tool_list = tool_invocations::list_tool_invocations_by_run(
        &env.db,
        &env.store,
        Some(&generation_run.id),
        None,
    )
    .await
    .expect("failed to list tool invocations");
    assert_eq!(tool_list.len(), 1);
    assert_eq!(tool_list[0].tool_name, "memory_lookup");
    assert_eq!(
        tool_list[0]
            .request_content
            .as_ref()
            .and_then(|content| content.preview_text.clone())
            .as_deref(),
        Some("lookup request")
    );

    let rag_list = rag::list_rag_refs_by_run(&env.db, &env.store, Some(&generation_run.id), None)
        .await
        .expect("failed to list rag refs");
    assert_eq!(rag_list.len(), 1);
    assert_eq!(rag_list[0].document_title.as_deref(), Some("Memory"));

    let mcp_list = mcp::list_mcp_events_by_run(&env.db, &env.store, Some(&generation_run.id), None)
        .await
        .expect("failed to list mcp events");
    assert_eq!(mcp_list.len(), 1);
    assert_eq!(mcp_list[0].method_name.as_deref(), Some("memory.lookup"));
}

#[tokio::test]
async fn native_capabilities_support_workflow_run_filters() {
    let env = TestEnv::new().await;

    let workflow_def = workflow_repo::create_workflow_def(
        &env.db,
        &workflow_repo::CreateWorkflowDefRecord {
            name: "Audit Workflow",
            description: Some("workflow audit"),
            enabled: true,
            sort_order: 0,
            config_json: "{}",
        },
    )
    .await
    .expect("failed to create workflow def");

    let mut tx = env.db.begin().await.expect("failed to begin tx");
    let nodes = workflow_repo::replace_workflow_def_nodes(
        &mut tx,
        &workflow_def.id,
        &[workflow_repo::WorkflowDefNodeRecord {
            node_key: "audit",
            name: Some("audit"),
            node_type: "plugin",
            agent_id: None,
            plugin_id: None,
            preset_id: None,
            lorebook_id: None,
            user_profile_id: None,
            api_channel_id: None,
            api_channel_model_id: None,
            workspace_mode: "inherit",
            history_read_mode: "full",
            summary_write_mode: "none",
            message_write_mode: "none",
            visible_output_mode: "final_only",
            config_json: "{}",
        }],
    )
    .await
    .expect("failed to create workflow node");
    tx.commit().await.expect("failed to commit tx");

    let workflow_run = workflow_repo::create_workflow_run(
        &env.db,
        &workflow_repo::CreateWorkflowRunRecord {
            workflow_def_id: &workflow_def.id,
            trigger_conversation_id: None,
            workspace_conversation_id: None,
            workspace_mode: "inherit",
            trigger_message_version_id: None,
            entry_node_id: Some(&nodes[0].id),
            status: "running",
            result_message_version_id: None,
            request_snapshot_content_id: None,
            result_content_id: None,
            config_json: "{}",
            started_at: None,
            finished_at: None,
        },
    )
    .await
    .expect("failed to create workflow run");

    let execution = workflow_repo::create_workflow_run_node_execution(
        &env.db,
        &workflow_repo::CreateWorkflowRunNodeExecutionRecord {
            workflow_run_id: &workflow_run.id,
            workflow_def_node_id: &nodes[0].id,
            parent_execution_id: None,
            incoming_edge_id: None,
            branch_key: None,
            loop_iteration: 0,
            retry_index: 0,
            status: "running",
            generation_run_id: None,
            input_snapshot_content_id: None,
            output_content_id: None,
            error_content_id: None,
            started_at: None,
            finished_at: None,
            config_json: "{}",
        },
    )
    .await
    .expect("failed to create workflow execution");

    let tool = tool_invocations::start_tool_invocation(
        &env.db,
        &env.store,
        &StartToolInvocationInput {
            generation_run_id: None,
            workflow_run_node_execution_id: Some(execution.id.clone()),
            message_version_id: None,
            tool_kind: "workflow_tool".to_string(),
            tool_name: "audit".to_string(),
            plugin_id: None,
            request_content: None,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create workflow-scoped tool invocation");

    let listed = tool_invocations::list_tool_invocations_by_run(
        &env.db,
        &env.store,
        None,
        Some(&workflow_run.id),
    )
    .await
    .expect("failed to list workflow-scoped tool invocations");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, tool.id);
}

fn text_input(text: &str, content_type: ContentType) -> ContentWriteInput {
    ContentWriteInput {
        content_type,
        mime_type: Some("text/plain".to_string()),
        text_content: Some(text.to_string()),
        source_file_path: None,
        primary_storage_uri: None,
        size_bytes_hint: None,
        preview_text: None,
        config_json: json!({}),
    }
}
