use std::{
    env, fs,
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
};

use buyu_lib::db::{
    pool,
    repos::{
        messages as message_repo, summaries as summary_repo, variables as variable_repo,
        workflows as workflow_repo,
    },
};
use buyu_lib::domain::agents::CreateAgentInput;
use buyu_lib::domain::api_channels::{CreateApiChannelInput, UpsertApiChannelModelInput};
use buyu_lib::domain::common::ChannelBindingInput;
use buyu_lib::domain::content::{ContentType, ContentWriteInput};
use buyu_lib::domain::conversations::{ConversationParticipantInput, CreateConversationInput};
use buyu_lib::domain::messages::{ContextPolicy, CreateMessageInput, MessageRole, ViewerPolicy};
use buyu_lib::domain::plugins::CreatePluginInput;
use buyu_lib::domain::workflows::{
    CreateWorkflowDefInput, RunWorkflowInput, UpdateWorkflowDefInput, WorkflowEdgeInput,
    WorkflowNodeInput,
};
use buyu_lib::extensions::runtime::PluginRuntime;
use buyu_lib::providers::ProviderRegistry;
use buyu_lib::services::{
    agents, api_channels, content, content_store, conversations, messages, plugins, workflows,
};
use serde_json::{json, Value};

struct TestEnv {
    _root: PathBuf,
    db: sqlx::SqlitePool,
    store: content_store::ContentStore,
    providers: ProviderRegistry,
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
            providers: ProviderRegistry::with_defaults(),
        }
    }
}

struct MockHttpServer {
    addr: SocketAddr,
    base_url: String,
    captured: Arc<Mutex<Vec<String>>>,
    handle: Option<thread::JoinHandle<()>>,
}

impl MockHttpServer {
    fn new(response_body: Value) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind mock server");
        let addr = listener.local_addr().expect("failed to read mock addr");
        let captured = Arc::new(Mutex::new(Vec::new()));
        let captured_clone = Arc::clone(&captured);
        let response_text = response_body.to_string();

        let handle = thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buffer = Vec::new();
                let mut temp = [0_u8; 4096];
                loop {
                    let read = stream.read(&mut temp).expect("failed to read request");
                    if read == 0 {
                        break;
                    }
                    buffer.extend_from_slice(&temp[..read]);
                    if let Some(header_end) = find_header_end(&buffer) {
                        let content_length = parse_content_length(&buffer[..header_end]);
                        let target_len = header_end + 4 + content_length;
                        while buffer.len() < target_len {
                            let read = stream.read(&mut temp).expect("failed to read request body");
                            if read == 0 {
                                break;
                            }
                            buffer.extend_from_slice(&temp[..read]);
                        }
                        break;
                    }
                }

                let request = String::from_utf8_lossy(&buffer).to_string();
                captured_clone.lock().unwrap().push(request);

                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    response_text.len(),
                    response_text
                );
                stream
                    .write_all(response.as_bytes())
                    .expect("failed to write response");
            }
        });

        Self {
            addr,
            base_url: format!("http://{}", addr),
            captured,
            handle: Some(handle),
        }
    }
}

impl Drop for MockHttpServer {
    fn drop(&mut self) {
        let _ = TcpStream::connect(self.addr);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

#[tokio::test]
async fn workflow_definition_and_terminal_output_smoke() {
    let env = TestEnv::new().await;

    let workflow = create_workflow_def(&env, "Narration Flow").await;
    workflows::replace_workflow_nodes(
        &env.db,
        &workflow.summary.id,
        &[
            workflow_node("input", "input"),
            workflow_node("output", "output"),
        ],
    )
    .await
    .expect("failed to replace workflow nodes");

    let detail = workflows::get_workflow_def_detail(&env.db, &workflow.summary.id)
        .await
        .expect("failed to load workflow detail");
    let input_node_id = find_node_id(&detail.nodes, "input");
    let output_node_id = find_node_id(&detail.nodes, "output");

    workflows::replace_workflow_edges(
        &env.db,
        &workflow.summary.id,
        &[WorkflowEdgeInput {
            from_node_id: input_node_id.clone(),
            to_node_id: output_node_id,
            edge_type: "success".to_string(),
            priority: 0,
            condition_expr: None,
            label: Some("next".to_string()),
            enabled: true,
            config_json: json!({}),
        }],
    )
    .await
    .expect("failed to replace workflow edges");

    let run = workflows::run_workflow(
        &env.db,
        &env.store,
        &env.providers,
        &RunWorkflowInput {
            workflow_def_id: workflow.summary.id.clone(),
            conversation_id: None,
            trigger_message_version_id: None,
            responder_participant_id: None,
            isolated_conversation_title: Some("Workflow Sandbox".to_string()),
            config_json: json!({ "mode": "smoke" }),
        },
    )
    .await
    .expect("failed to run workflow");

    assert_eq!(run.status, "succeeded");
    assert_eq!(run.entry_node_id.as_deref(), Some(input_node_id.as_str()));
    assert!(run.workspace_conversation_id.is_some());
    assert!(run.result_message_version_id.is_some());

    let executions = workflows::list_workflow_run_node_executions(&env.db, &run.workflow_run_id)
        .await
        .expect("failed to list workflow run executions");
    assert_eq!(executions.len(), 2);
    assert!(executions.iter().all(|item| item.status == "succeeded"));

    let workspace_conversation_id = run
        .workspace_conversation_id
        .as_deref()
        .expect("missing workspace conversation");
    let visible = messages::list_visible_messages(&env.db, &env.store, workspace_conversation_id)
        .await
        .expect("failed to list visible messages");
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].role.as_str(), MessageRole::Assistant.as_str());
    assert_eq!(
        visible[0].primary_content.content_type.as_str(),
        ContentType::Json.as_str()
    );
}

#[tokio::test]
async fn workflow_agent_generation_smoke() {
    let env = TestEnv::new().await;
    let server = MockHttpServer::new(json!({
        "choices": [{
            "message": { "content": "ok from workflow" },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 13,
            "completion_tokens": 8,
            "total_tokens": 21
        }
    }));

    let channel = api_channels::create_channel(
        &env.db,
        &CreateApiChannelInput {
            name: "Main".to_string(),
            channel_type: "openai_compatible".to_string(),
            base_url: server.base_url.clone(),
            auth_type: "none".to_string(),
            api_key: None,
            models_endpoint: Some("/models".to_string()),
            chat_endpoint: Some("/chat/completions".to_string()),
            stream_endpoint: Some("/chat/completions".to_string()),
            models_mode: "hybrid".to_string(),
            enabled: true,
            sort_order: 0,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create channel");

    let model = api_channels::upsert_channel_model(
        &env.db,
        &UpsertApiChannelModelInput {
            channel_id: channel.id.clone(),
            model_id: "gpt-5.1".to_string(),
            display_name: Some("GPT-5.1".to_string()),
            model_type: Some("chat".to_string()),
            context_window: Some(128000),
            max_output_tokens: Some(8192),
            capabilities_json: json!({}),
            pricing_json: json!({}),
            default_parameters_json: json!({"temperature": 0.3}),
            sort_order: 0,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create model");

    let agent = agents::create_agent(
        &env.db,
        &env.store,
        &CreateAgentInput {
            name: "Nora".to_string(),
            title: None,
            description_content: Some(text_input("A helpful guide.")),
            personality_content: Some(text_input("Warm and direct.")),
            scenario_content: None,
            example_messages_content: None,
            main_prompt_override_content: Some(text_input("Answer clearly.")),
            post_history_instructions_content: None,
            character_note_content: None,
            creator_notes_content: None,
            character_note_depth: None,
            character_note_role: None,
            talkativeness: 50,
            avatar_uri: None,
            creator_name: None,
            character_version: None,
            enabled: true,
            sort_order: 0,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create agent");

    let conversation = conversations::create_conversation(
        &env.db,
        &CreateConversationInput {
            title: "Workflow Thread".to_string(),
            description: None,
            conversation_mode: "single".to_string(),
            archived: false,
            pinned: false,
            participants: vec![
                ConversationParticipantInput {
                    agent_id: None,
                    display_name: Some("User".to_string()),
                    participant_type: "human".to_string(),
                    enabled: true,
                    sort_order: 0,
                    config_json: json!({}),
                },
                ConversationParticipantInput {
                    agent_id: Some(agent.summary.id.clone()),
                    display_name: Some("Nora".to_string()),
                    participant_type: "agent".to_string(),
                    enabled: true,
                    sort_order: 1,
                    config_json: json!({}),
                },
            ],
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create conversation");

    let human_participant_id = conversation
        .participants
        .iter()
        .find(|item| item.participant_type == "human")
        .expect("missing human participant")
        .id
        .clone();
    let agent_participant_id = conversation
        .participants
        .iter()
        .find(|item| item.agent_id.as_deref() == Some(agent.summary.id.as_str()))
        .expect("missing agent participant")
        .id
        .clone();

    conversations::replace_channels(
        &env.db,
        &conversation.summary.id,
        &[ChannelBindingInput {
            channel_id: channel.id.clone(),
            channel_model_id: Some(model.id.clone()),
            binding_type: "active".to_string(),
            enabled: true,
            sort_order: 0,
            config_json: json!({}),
        }],
    )
    .await
    .expect("failed to bind channel");

    let user_message = messages::create_user_message(
        &env.db,
        &env.store,
        &CreateMessageInput {
            conversation_id: conversation.summary.id.clone(),
            author_participant_id: human_participant_id,
            role: MessageRole::User,
            reply_to_node_id: None,
            order_after_node_id: None,
            primary_content: text_input("hello from workflow"),
            context_policy: ContextPolicy::Full,
            viewer_policy: ViewerPolicy::Full,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create user message");

    let workflow = create_workflow_def(&env, "Agent Flow").await;
    let mut agent_node = workflow_node("agent", "agent");
    agent_node.agent_id = Some(agent.summary.id.clone());
    agent_node.config_json = json!({
        "request_parameters_json": { "temperature": 0.2 }
    });
    workflows::replace_workflow_nodes(
        &env.db,
        &workflow.summary.id,
        &[
            workflow_node("input", "input"),
            agent_node,
            workflow_node("output", "output"),
        ],
    )
    .await
    .expect("failed to replace workflow nodes");

    let detail = workflows::get_workflow_def_detail(&env.db, &workflow.summary.id)
        .await
        .expect("failed to load workflow detail");
    let input_node_id = find_node_id(&detail.nodes, "input");
    let agent_node_id = find_node_id(&detail.nodes, "agent");
    let output_node_id = find_node_id(&detail.nodes, "output");

    workflows::replace_workflow_edges(
        &env.db,
        &workflow.summary.id,
        &[
            WorkflowEdgeInput {
                from_node_id: input_node_id,
                to_node_id: agent_node_id.clone(),
                edge_type: "success".to_string(),
                priority: 0,
                condition_expr: None,
                label: Some("to_agent".to_string()),
                enabled: true,
                config_json: json!({}),
            },
            WorkflowEdgeInput {
                from_node_id: agent_node_id,
                to_node_id: output_node_id,
                edge_type: "success".to_string(),
                priority: 0,
                condition_expr: None,
                label: Some("to_output".to_string()),
                enabled: true,
                config_json: json!({}),
            },
        ],
    )
    .await
    .expect("failed to replace workflow edges");

    let run = workflows::run_workflow(
        &env.db,
        &env.store,
        &env.providers,
        &RunWorkflowInput {
            workflow_def_id: workflow.summary.id.clone(),
            conversation_id: Some(conversation.summary.id.clone()),
            trigger_message_version_id: Some(user_message.version_id.clone()),
            responder_participant_id: Some(agent_participant_id),
            isolated_conversation_title: None,
            config_json: json!({ "mode": "agent_smoke" }),
        },
    )
    .await
    .expect("failed to run workflow");

    assert_eq!(run.status, "succeeded");
    let result_message_version_id = run
        .result_message_version_id
        .as_deref()
        .expect("missing workflow result message");
    let result = messages::get_message_version_view(&env.db, &env.store, result_message_version_id)
        .await
        .expect("failed to load workflow result message");
    assert_eq!(
        result.primary_content.text_content.as_deref(),
        Some("ok from workflow")
    );
    assert_eq!(result.role.as_str(), MessageRole::Assistant.as_str());
    assert_eq!(result.api_channel_id.as_deref(), Some(channel.id.as_str()));
    assert_eq!(
        result.api_channel_model_id.as_deref(),
        Some(model.id.as_str())
    );
    assert!(result.generation_run_id.is_some());

    let executions = workflows::list_workflow_run_node_executions(&env.db, &run.workflow_run_id)
        .await
        .expect("failed to list workflow run executions");
    assert_eq!(executions.len(), 3);
    assert!(executions.iter().all(|item| item.status == "succeeded"));

    let visible = messages::list_visible_messages(&env.db, &env.store, &conversation.summary.id)
        .await
        .expect("failed to list visible messages");
    assert_eq!(visible.len(), 2);
    assert_eq!(
        visible[1].primary_content.text_content.as_deref(),
        Some("ok from workflow")
    );

    let captured = server.captured.lock().unwrap();
    let request_text = captured.first().expect("missing captured request");
    assert!(request_text.contains("/chat/completions"));
    assert!(request_text.contains("hello from workflow"));
}

#[tokio::test]
async fn workflow_writeback_summary_and_variable_smoke() {
    let env = TestEnv::new().await;

    let agent = agents::create_agent(
        &env.db,
        &env.store,
        &CreateAgentInput {
            name: "Writer".to_string(),
            title: None,
            description_content: Some(text_input("Writes workflow outputs.")),
            personality_content: None,
            scenario_content: None,
            example_messages_content: None,
            main_prompt_override_content: None,
            post_history_instructions_content: None,
            character_note_content: None,
            creator_notes_content: None,
            character_note_depth: None,
            character_note_role: None,
            talkativeness: 40,
            avatar_uri: None,
            creator_name: None,
            character_version: None,
            enabled: true,
            sort_order: 0,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create agent");

    let conversation = conversations::create_conversation(
        &env.db,
        &CreateConversationInput {
            title: "Writeback Thread".to_string(),
            description: None,
            conversation_mode: "single".to_string(),
            archived: false,
            pinned: false,
            participants: vec![
                ConversationParticipantInput {
                    agent_id: None,
                    display_name: Some("User".to_string()),
                    participant_type: "human".to_string(),
                    enabled: true,
                    sort_order: 0,
                    config_json: json!({}),
                },
                ConversationParticipantInput {
                    agent_id: Some(agent.summary.id.clone()),
                    display_name: Some("Writer".to_string()),
                    participant_type: "agent".to_string(),
                    enabled: true,
                    sort_order: 1,
                    config_json: json!({}),
                },
            ],
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create conversation");

    let human_participant_id = conversation
        .participants
        .iter()
        .find(|item| item.participant_type == "human")
        .expect("missing human participant")
        .id
        .clone();
    let agent_participant_id = conversation
        .participants
        .iter()
        .find(|item| item.agent_id.as_deref() == Some(agent.summary.id.as_str()))
        .expect("missing agent participant")
        .id
        .clone();

    let user_message = messages::create_user_message(
        &env.db,
        &env.store,
        &CreateMessageInput {
            conversation_id: conversation.summary.id.clone(),
            author_participant_id: human_participant_id,
            role: MessageRole::User,
            reply_to_node_id: None,
            order_after_node_id: None,
            primary_content: text_input("remember this state"),
            context_policy: ContextPolicy::Full,
            viewer_policy: ViewerPolicy::Full,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create user message");

    let workflow = create_workflow_def(&env, "Writeback Flow").await;
    let mut writeback_node = workflow_node("writeback", "writeback");
    writeback_node.agent_id = Some(agent.summary.id.clone());
    writeback_node.summary_write_mode = "create_and_activate".to_string();
    writeback_node.message_write_mode = "append_hidden".to_string();
    writeback_node.config_json = json!({
        "message_role": "assistant",
        "variable": {
            "var_key": "workflow.note",
            "scope_type": "workflow_run",
            "name": "Workflow Note",
            "namespace": "workflow",
            "value_type": "string"
        }
    });
    workflows::replace_workflow_nodes(
        &env.db,
        &workflow.summary.id,
        &[
            workflow_node("input", "input"),
            writeback_node,
            workflow_node("output", "output"),
        ],
    )
    .await
    .expect("failed to replace workflow nodes");

    let detail = workflows::get_workflow_def_detail(&env.db, &workflow.summary.id)
        .await
        .expect("failed to load workflow detail");
    let input_node_id = find_node_id(&detail.nodes, "input");
    let writeback_node_id = find_node_id(&detail.nodes, "writeback");
    let output_node_id = find_node_id(&detail.nodes, "output");

    workflows::replace_workflow_edges(
        &env.db,
        &workflow.summary.id,
        &[
            WorkflowEdgeInput {
                from_node_id: input_node_id,
                to_node_id: writeback_node_id.clone(),
                edge_type: "success".to_string(),
                priority: 0,
                condition_expr: None,
                label: Some("to_writeback".to_string()),
                enabled: true,
                config_json: json!({}),
            },
            WorkflowEdgeInput {
                from_node_id: writeback_node_id,
                to_node_id: output_node_id,
                edge_type: "success".to_string(),
                priority: 0,
                condition_expr: None,
                label: Some("to_output".to_string()),
                enabled: true,
                config_json: json!({}),
            },
        ],
    )
    .await
    .expect("failed to replace workflow edges");

    let run = workflows::run_workflow(
        &env.db,
        &env.store,
        &env.providers,
        &RunWorkflowInput {
            workflow_def_id: workflow.summary.id.clone(),
            conversation_id: Some(conversation.summary.id.clone()),
            trigger_message_version_id: Some(user_message.version_id.clone()),
            responder_participant_id: Some(agent_participant_id),
            isolated_conversation_title: None,
            config_json: json!({ "mode": "writeback_smoke" }),
        },
    )
    .await
    .expect("failed to run workflow");

    assert_eq!(run.status, "succeeded");
    let writes = workflow_repo::list_workflow_run_writes(&env.db, &run.workflow_run_id)
        .await
        .expect("failed to list workflow writes");
    assert!(writes
        .iter()
        .any(|item| item.write_kind == "message_hidden"));
    assert!(writes
        .iter()
        .any(|item| item.write_kind == "summary_version"));
    assert!(writes
        .iter()
        .any(|item| item.write_kind == "variable_value"));
    assert!(writes
        .iter()
        .any(|item| item.write_kind == "message_visible"));

    let summary_groups =
        summary_repo::list_summary_groups_by_conversation(&env.db, &conversation.summary.id)
            .await
            .expect("failed to list summary groups");
    assert_eq!(summary_groups.len(), 1);
    let summary_versions = summary_repo::list_summary_versions(&env.db, &summary_groups[0].id)
        .await
        .expect("failed to list summary versions");
    assert_eq!(summary_versions.len(), 1);
    assert!(summary_versions[0].is_active);
    let summary_content =
        content::get_content(&env.db, &env.store, &summary_versions[0].content_id, true)
            .await
            .expect("failed to load summary content");
    assert_eq!(
        summary_content.text_content.as_deref(),
        Some("remember this state")
    );

    let variable_def = variable_repo::get_variable_def_by_key(&env.db, "workflow.note")
        .await
        .expect("failed to query variable def")
        .expect("missing workflow variable definition");
    let variable_value = variable_repo::get_variable_value(
        &env.db,
        &variable_def.id,
        "workflow_run",
        &run.workflow_run_id,
        false,
    )
    .await
    .expect("failed to query variable value")
    .expect("missing workflow variable value");
    assert!(variable_value.value_json.contains("remember this state"));

    let final_message_version_id = run
        .result_message_version_id
        .as_deref()
        .expect("missing final message version id");
    let final_message =
        messages::get_message_version_view(&env.db, &env.store, final_message_version_id)
            .await
            .expect("failed to load final message");
    assert_eq!(
        final_message.primary_content.text_content.as_deref(),
        Some("remember this state")
    );
    assert_eq!(final_message.role.as_str(), MessageRole::Assistant.as_str());
}

#[tokio::test]
async fn workflow_router_branches_to_matching_edge() {
    let env = TestEnv::new().await;
    let conversation = conversations::create_conversation(
        &env.db,
        &CreateConversationInput {
            title: "Router Thread".to_string(),
            description: None,
            conversation_mode: "single".to_string(),
            archived: false,
            pinned: false,
            participants: vec![ConversationParticipantInput {
                agent_id: None,
                display_name: Some("User".to_string()),
                participant_type: "human".to_string(),
                enabled: true,
                sort_order: 0,
                config_json: json!({}),
            }],
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create conversation");
    let human_participant_id = conversation.participants[0].id.clone();
    let user_message = messages::create_user_message(
        &env.db,
        &env.store,
        &CreateMessageInput {
            conversation_id: conversation.summary.id.clone(),
            author_participant_id: human_participant_id,
            role: MessageRole::User,
            reply_to_node_id: None,
            order_after_node_id: None,
            primary_content: text_input("take the second branch"),
            context_policy: ContextPolicy::Full,
            viewer_policy: ViewerPolicy::Full,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create user message");

    let workflow = create_workflow_def(&env, "Router Flow").await;
    workflows::replace_workflow_nodes(
        &env.db,
        &workflow.summary.id,
        &[
            workflow_node("input", "input"),
            workflow_node("router", "router"),
            workflow_node("first", "output"),
            workflow_node("second", "output"),
        ],
    )
    .await
    .expect("failed to replace workflow nodes");

    let detail = workflows::get_workflow_def_detail(&env.db, &workflow.summary.id)
        .await
        .expect("failed to load workflow detail");
    let input_node_id = find_node_id(&detail.nodes, "input");
    let router_node_id = find_node_id(&detail.nodes, "router");
    let first_node_id = find_node_id(&detail.nodes, "first");
    let second_node_id = find_node_id(&detail.nodes, "second");

    workflows::replace_workflow_edges(
        &env.db,
        &workflow.summary.id,
        &[
            WorkflowEdgeInput {
                from_node_id: input_node_id,
                to_node_id: router_node_id.clone(),
                edge_type: "success".to_string(),
                priority: 0,
                condition_expr: None,
                label: Some("to_router".to_string()),
                enabled: true,
                config_json: json!({}),
            },
            WorkflowEdgeInput {
                from_node_id: router_node_id.clone(),
                to_node_id: first_node_id.clone(),
                edge_type: "success".to_string(),
                priority: 0,
                condition_expr: Some("text_contains:first".to_string()),
                label: Some("first".to_string()),
                enabled: true,
                config_json: json!({}),
            },
            WorkflowEdgeInput {
                from_node_id: router_node_id,
                to_node_id: second_node_id.clone(),
                edge_type: "success".to_string(),
                priority: 1,
                condition_expr: Some("text_contains:second".to_string()),
                label: Some("second".to_string()),
                enabled: true,
                config_json: json!({}),
            },
        ],
    )
    .await
    .expect("failed to replace workflow edges");

    let run = workflows::run_workflow(
        &env.db,
        &env.store,
        &env.providers,
        &RunWorkflowInput {
            workflow_def_id: workflow.summary.id.clone(),
            conversation_id: Some(conversation.summary.id.clone()),
            trigger_message_version_id: Some(user_message.version_id.clone()),
            responder_participant_id: None,
            isolated_conversation_title: None,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to run workflow");

    assert_eq!(run.status, "succeeded");
    assert_eq!(
        run.result_message_version_id.as_deref(),
        Some(user_message.version_id.as_str())
    );
    let executions =
        workflow_repo::list_workflow_run_node_executions(&env.db, &run.workflow_run_id)
            .await
            .expect("failed to list workflow executions");
    assert_eq!(executions.len(), 3);
    assert!(executions
        .iter()
        .any(|item| item.workflow_def_node_id == second_node_id));
    assert!(!executions
        .iter()
        .any(|item| item.workflow_def_node_id == first_node_id));
}

#[tokio::test]
async fn workflow_merge_joins_all_inputs() {
    let env = TestEnv::new().await;
    let conversation = conversations::create_conversation(
        &env.db,
        &CreateConversationInput {
            title: "Merge Thread".to_string(),
            description: None,
            conversation_mode: "single".to_string(),
            archived: false,
            pinned: false,
            participants: vec![ConversationParticipantInput {
                agent_id: None,
                display_name: Some("User".to_string()),
                participant_type: "human".to_string(),
                enabled: true,
                sort_order: 0,
                config_json: json!({}),
            }],
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create conversation");
    let human_participant_id = conversation.participants[0].id.clone();
    let user_message = messages::create_user_message(
        &env.db,
        &env.store,
        &CreateMessageInput {
            conversation_id: conversation.summary.id.clone(),
            author_participant_id: human_participant_id.clone(),
            role: MessageRole::User,
            reply_to_node_id: None,
            order_after_node_id: None,
            primary_content: text_input("merge this"),
            context_policy: ContextPolicy::Full,
            viewer_policy: ViewerPolicy::Full,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create user message");

    let workflow = create_workflow_def(&env, "Merge Flow").await;
    workflows::replace_workflow_nodes(
        &env.db,
        &workflow.summary.id,
        &[
            workflow_node("input", "input"),
            workflow_node("branch_a", "router"),
            workflow_node("branch_b", "router"),
            workflow_node("merge", "merge"),
            workflow_node("output", "output"),
        ],
    )
    .await
    .expect("failed to replace workflow nodes");

    let detail = workflows::get_workflow_def_detail(&env.db, &workflow.summary.id)
        .await
        .expect("failed to load workflow detail");
    let input_node_id = find_node_id(&detail.nodes, "input");
    let branch_a_id = find_node_id(&detail.nodes, "branch_a");
    let branch_b_id = find_node_id(&detail.nodes, "branch_b");
    let merge_node_id = find_node_id(&detail.nodes, "merge");
    let output_node_id = find_node_id(&detail.nodes, "output");

    workflows::replace_workflow_edges(
        &env.db,
        &workflow.summary.id,
        &[
            WorkflowEdgeInput {
                from_node_id: input_node_id.clone(),
                to_node_id: branch_a_id.clone(),
                edge_type: "success".to_string(),
                priority: 0,
                condition_expr: None,
                label: Some("a".to_string()),
                enabled: true,
                config_json: json!({}),
            },
            WorkflowEdgeInput {
                from_node_id: input_node_id,
                to_node_id: branch_b_id.clone(),
                edge_type: "success".to_string(),
                priority: 1,
                condition_expr: None,
                label: Some("b".to_string()),
                enabled: true,
                config_json: json!({}),
            },
            WorkflowEdgeInput {
                from_node_id: branch_a_id,
                to_node_id: merge_node_id.clone(),
                edge_type: "success".to_string(),
                priority: 0,
                condition_expr: None,
                label: Some("merge_a".to_string()),
                enabled: true,
                config_json: json!({}),
            },
            WorkflowEdgeInput {
                from_node_id: branch_b_id,
                to_node_id: merge_node_id.clone(),
                edge_type: "success".to_string(),
                priority: 1,
                condition_expr: None,
                label: Some("merge_b".to_string()),
                enabled: true,
                config_json: json!({}),
            },
            WorkflowEdgeInput {
                from_node_id: merge_node_id.clone(),
                to_node_id: output_node_id,
                edge_type: "success".to_string(),
                priority: 0,
                condition_expr: None,
                label: Some("done".to_string()),
                enabled: true,
                config_json: json!({}),
            },
        ],
    )
    .await
    .expect("failed to replace workflow edges");

    let run = workflows::run_workflow(
        &env.db,
        &env.store,
        &env.providers,
        &RunWorkflowInput {
            workflow_def_id: workflow.summary.id.clone(),
            conversation_id: Some(conversation.summary.id.clone()),
            trigger_message_version_id: Some(user_message.version_id.clone()),
            responder_participant_id: Some(human_participant_id),
            isolated_conversation_title: None,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to run workflow");

    assert_eq!(run.status, "succeeded");
    let executions =
        workflow_repo::list_workflow_run_node_executions(&env.db, &run.workflow_run_id)
            .await
            .expect("failed to list workflow executions");
    assert_eq!(executions.len(), 5);
    assert!(executions
        .iter()
        .any(|item| item.workflow_def_node_id == merge_node_id));

    let final_message = messages::get_message_version_view(
        &env.db,
        &env.store,
        run.result_message_version_id
            .as_deref()
            .expect("missing result message"),
    )
    .await
    .expect("failed to load final message");
    let final_text = final_message
        .primary_content
        .text_content
        .clone()
        .expect("missing merged text");
    assert!(final_text.contains("[Input 1]"));
    assert!(final_text.contains("[Input 2]"));
    assert!(final_text.contains("merge this"));
}

#[tokio::test]
async fn workflow_loop_respects_max_iterations() {
    let env = TestEnv::new().await;
    let workflow = create_workflow_def(&env, "Loop Flow").await;
    let mut loop_node = workflow_node("loop", "loop");
    loop_node.config_json = json!({ "max_iterations": 2 });
    workflows::replace_workflow_nodes(
        &env.db,
        &workflow.summary.id,
        &[workflow_node("input", "input"), loop_node],
    )
    .await
    .expect("failed to replace workflow nodes");

    let detail = workflows::get_workflow_def_detail(&env.db, &workflow.summary.id)
        .await
        .expect("failed to load workflow detail");
    let input_node_id = find_node_id(&detail.nodes, "input");
    let loop_node_id = find_node_id(&detail.nodes, "loop");

    workflows::replace_workflow_edges(
        &env.db,
        &workflow.summary.id,
        &[
            WorkflowEdgeInput {
                from_node_id: input_node_id,
                to_node_id: loop_node_id.clone(),
                edge_type: "success".to_string(),
                priority: 0,
                condition_expr: None,
                label: Some("start".to_string()),
                enabled: true,
                config_json: json!({}),
            },
            WorkflowEdgeInput {
                from_node_id: loop_node_id.clone(),
                to_node_id: loop_node_id,
                edge_type: "loop_back".to_string(),
                priority: 0,
                condition_expr: Some("true".to_string()),
                label: Some("again".to_string()),
                enabled: true,
                config_json: json!({}),
            },
        ],
    )
    .await
    .expect("failed to replace workflow edges");

    let run = workflows::run_workflow(
        &env.db,
        &env.store,
        &env.providers,
        &RunWorkflowInput {
            workflow_def_id: workflow.summary.id.clone(),
            conversation_id: None,
            trigger_message_version_id: None,
            responder_participant_id: None,
            isolated_conversation_title: None,
            config_json: json!({}),
        },
    )
    .await
    .expect("workflow run should finalize as failed instead of bubbling error");

    assert_eq!(run.status, "failed");
    let run_row = workflow_repo::get_workflow_run(&env.db, &run.workflow_run_id)
        .await
        .expect("failed to load workflow run");
    assert_eq!(run_row.status, "failed");
    assert!(run_row.config_json.contains("max_iterations"));

    let executions =
        workflow_repo::list_workflow_run_node_executions(&env.db, &run.workflow_run_id)
            .await
            .expect("failed to list workflow executions");
    assert_eq!(executions.len(), 4);
}

#[tokio::test]
async fn workflow_native_nodes_feed_agent_context() {
    let env = TestEnv::new().await;
    let plugin_runtime = PluginRuntime::new();
    let server = MockHttpServer::new(json!({
        "choices": [{
            "message": { "content": "native flow ok" },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 17,
            "completion_tokens": 9,
            "total_tokens": 26
        }
    }));

    let channel = api_channels::create_channel(
        &env.db,
        &CreateApiChannelInput {
            name: "Main".to_string(),
            channel_type: "openai_compatible".to_string(),
            base_url: server.base_url.clone(),
            auth_type: "none".to_string(),
            api_key: None,
            models_endpoint: Some("/models".to_string()),
            chat_endpoint: Some("/chat/completions".to_string()),
            stream_endpoint: Some("/chat/completions".to_string()),
            models_mode: "hybrid".to_string(),
            enabled: true,
            sort_order: 0,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create channel");

    let model = api_channels::upsert_channel_model(
        &env.db,
        &UpsertApiChannelModelInput {
            channel_id: channel.id.clone(),
            model_id: "gpt-5.1".to_string(),
            display_name: Some("GPT-5.1".to_string()),
            model_type: Some("chat".to_string()),
            context_window: Some(128000),
            max_output_tokens: Some(8192),
            capabilities_json: json!({}),
            pricing_json: json!({}),
            default_parameters_json: json!({"temperature": 0.2}),
            sort_order: 0,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create model");

    let plugin = plugins::create_plugin(
        &env.db,
        &plugin_runtime,
        &CreatePluginInput {
            name: "Memory Widget".to_string(),
            plugin_key: "memory.widget".to_string(),
            version: "0.1.0".to_string(),
            runtime_kind: "builtin".to_string(),
            entrypoint: None,
            enabled: true,
            sort_order: 0,
            capabilities_json: json!(["workflow_node"]),
            permissions_json: json!({}),
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create plugin");

    let agent = agents::create_agent(
        &env.db,
        &env.store,
        &CreateAgentInput {
            name: "Nora".to_string(),
            title: None,
            description_content: Some(text_input("A helpful guide.")),
            personality_content: Some(text_input("Warm and direct.")),
            scenario_content: None,
            example_messages_content: None,
            main_prompt_override_content: Some(text_input("Answer clearly.")),
            post_history_instructions_content: None,
            character_note_content: None,
            creator_notes_content: None,
            character_note_depth: None,
            character_note_role: None,
            talkativeness: 50,
            avatar_uri: None,
            creator_name: None,
            character_version: None,
            enabled: true,
            sort_order: 0,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create agent");

    let conversation = conversations::create_conversation(
        &env.db,
        &CreateConversationInput {
            title: "Native Flow".to_string(),
            description: None,
            conversation_mode: "single".to_string(),
            archived: false,
            pinned: false,
            participants: vec![
                ConversationParticipantInput {
                    agent_id: None,
                    display_name: Some("User".to_string()),
                    participant_type: "human".to_string(),
                    enabled: true,
                    sort_order: 0,
                    config_json: json!({}),
                },
                ConversationParticipantInput {
                    agent_id: Some(agent.summary.id.clone()),
                    display_name: Some("Nora".to_string()),
                    participant_type: "agent".to_string(),
                    enabled: true,
                    sort_order: 1,
                    config_json: json!({}),
                },
            ],
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create conversation");

    let human_participant_id = conversation
        .participants
        .iter()
        .find(|item| item.participant_type == "human")
        .expect("missing human participant")
        .id
        .clone();
    let agent_participant_id = conversation
        .participants
        .iter()
        .find(|item| item.agent_id.as_deref() == Some(agent.summary.id.as_str()))
        .expect("missing agent participant")
        .id
        .clone();

    conversations::replace_channels(
        &env.db,
        &conversation.summary.id,
        &[ChannelBindingInput {
            channel_id: channel.id.clone(),
            channel_model_id: Some(model.id.clone()),
            binding_type: "active".to_string(),
            enabled: true,
            sort_order: 0,
            config_json: json!({}),
        }],
    )
    .await
    .expect("failed to bind channel");

    let user_message = messages::create_user_message(
        &env.db,
        &env.store,
        &CreateMessageInput {
            conversation_id: conversation.summary.id.clone(),
            author_participant_id: human_participant_id,
            role: MessageRole::User,
            reply_to_node_id: None,
            order_after_node_id: None,
            primary_content: text_input("start native chain"),
            context_policy: ContextPolicy::Full,
            viewer_policy: ViewerPolicy::Full,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create user message");

    let workflow = create_workflow_def(&env, "Native Capability Flow").await;
    let mut tool_node = workflow_node("tool", "tool");
    tool_node.plugin_id = Some(plugin.id.clone());
    tool_node.config_json = json!({
        "tool_kind": "plugin",
        "tool_name": "memory_lookup",
        "request": { "use_input": true },
        "response": { "text": "tool says hello back" },
    });

    let mut rag_node = workflow_node("rag", "rag");
    rag_node.config_json = json!({
        "source_uri": "memory://hero-sheet",
        "document_title": "Hero Sheet",
        "chunk_key": "hero-sheet#1",
        "score": 0.91,
        "excerpt": { "text": "rag remembers the user is a hero" },
        "included_in_request": true,
    });

    let mut mcp_node = workflow_node("mcp", "mcp");
    mcp_node.config_json = json!({
        "server_name": "memory-server",
        "event_kind": "tool_result",
        "method_name": "lookup",
        "payload": { "text": "mcp payload from memory server" },
        "status": "succeeded",
    });

    let mut plugin_node = workflow_node("plugin", "plugin");
    plugin_node.plugin_id = Some(plugin.id.clone());
    plugin_node.config_json = json!({
        "response": { "text": "plugin payload marker" },
    });

    let mut agent_node = workflow_node("agent", "agent");
    agent_node.agent_id = Some(agent.summary.id.clone());
    agent_node.config_json = json!({
        "request_parameters_json": { "temperature": 0.2 }
    });

    workflows::replace_workflow_nodes(
        &env.db,
        &workflow.summary.id,
        &[
            workflow_node("input", "input"),
            tool_node,
            rag_node,
            mcp_node,
            plugin_node,
            agent_node,
            workflow_node("output", "output"),
        ],
    )
    .await
    .expect("failed to replace workflow nodes");

    let detail = workflows::get_workflow_def_detail(&env.db, &workflow.summary.id)
        .await
        .expect("failed to load workflow detail");
    let input_node_id = find_node_id(&detail.nodes, "input");
    let tool_node_id = find_node_id(&detail.nodes, "tool");
    let rag_node_id = find_node_id(&detail.nodes, "rag");
    let mcp_node_id = find_node_id(&detail.nodes, "mcp");
    let plugin_node_id = find_node_id(&detail.nodes, "plugin");
    let agent_node_id = find_node_id(&detail.nodes, "agent");
    let output_node_id = find_node_id(&detail.nodes, "output");

    workflows::replace_workflow_edges(
        &env.db,
        &workflow.summary.id,
        &[
            WorkflowEdgeInput {
                from_node_id: input_node_id,
                to_node_id: tool_node_id.clone(),
                edge_type: "success".to_string(),
                priority: 0,
                condition_expr: None,
                label: Some("to_tool".to_string()),
                enabled: true,
                config_json: json!({}),
            },
            WorkflowEdgeInput {
                from_node_id: tool_node_id,
                to_node_id: rag_node_id.clone(),
                edge_type: "success".to_string(),
                priority: 0,
                condition_expr: None,
                label: Some("to_rag".to_string()),
                enabled: true,
                config_json: json!({}),
            },
            WorkflowEdgeInput {
                from_node_id: rag_node_id,
                to_node_id: mcp_node_id.clone(),
                edge_type: "success".to_string(),
                priority: 0,
                condition_expr: None,
                label: Some("to_mcp".to_string()),
                enabled: true,
                config_json: json!({}),
            },
            WorkflowEdgeInput {
                from_node_id: mcp_node_id,
                to_node_id: plugin_node_id.clone(),
                edge_type: "success".to_string(),
                priority: 0,
                condition_expr: None,
                label: Some("to_plugin".to_string()),
                enabled: true,
                config_json: json!({}),
            },
            WorkflowEdgeInput {
                from_node_id: plugin_node_id,
                to_node_id: agent_node_id.clone(),
                edge_type: "success".to_string(),
                priority: 0,
                condition_expr: None,
                label: Some("to_agent".to_string()),
                enabled: true,
                config_json: json!({}),
            },
            WorkflowEdgeInput {
                from_node_id: agent_node_id.clone(),
                to_node_id: output_node_id,
                edge_type: "success".to_string(),
                priority: 0,
                condition_expr: None,
                label: Some("to_output".to_string()),
                enabled: true,
                config_json: json!({}),
            },
        ],
    )
    .await
    .expect("failed to replace workflow edges");

    let run = workflows::run_workflow(
        &env.db,
        &env.store,
        &env.providers,
        &RunWorkflowInput {
            workflow_def_id: workflow.summary.id.clone(),
            conversation_id: Some(conversation.summary.id.clone()),
            trigger_message_version_id: Some(user_message.version_id.clone()),
            responder_participant_id: Some(agent_participant_id),
            isolated_conversation_title: None,
            config_json: json!({ "mode": "native_smoke" }),
        },
    )
    .await
    .expect("failed to run workflow");

    assert_eq!(run.status, "succeeded");
    let execution_results =
        workflows::list_workflow_run_node_executions(&env.db, &run.workflow_run_id)
            .await
            .expect("failed to list workflow executions");
    assert_eq!(execution_results.len(), 7);
    let execution_rows =
        workflow_repo::list_workflow_run_node_executions(&env.db, &run.workflow_run_id)
            .await
            .expect("failed to load workflow execution rows");
    let agent_execution = execution_rows
        .iter()
        .find(|item| item.workflow_def_node_id == agent_node_id)
        .expect("missing agent execution");
    let generation_run_id = agent_execution
        .generation_run_id
        .as_deref()
        .expect("agent node should produce generation run");

    let tool_invocations = buyu_lib::services::tool_invocations::list_tool_invocations_by_run(
        &env.db,
        &env.store,
        None,
        Some(&run.workflow_run_id),
    )
    .await
    .expect("failed to list tool invocations");
    assert_eq!(tool_invocations.len(), 1);

    let rag_refs = buyu_lib::services::rag::list_rag_refs_by_run(
        &env.db,
        &env.store,
        None,
        Some(&run.workflow_run_id),
    )
    .await
    .expect("failed to list rag refs");
    assert_eq!(rag_refs.len(), 1);

    let mcp_events = buyu_lib::services::mcp::list_mcp_events_by_run(
        &env.db,
        &env.store,
        None,
        Some(&run.workflow_run_id),
    )
    .await
    .expect("failed to list mcp events");
    assert_eq!(mcp_events.len(), 1);

    let context_items = message_repo::list_generation_run_context_items(&env.db, generation_run_id)
        .await
        .expect("failed to list generation context items");
    assert!(context_items
        .iter()
        .any(|item| item.source_tool_invocation_id.as_deref()
            == Some(tool_invocations[0].id.as_str())));
    assert!(context_items
        .iter()
        .any(|item| item.source_rag_ref_id.as_deref() == Some(rag_refs[0].id.as_str())));
    assert!(context_items
        .iter()
        .any(|item| item.source_mcp_event_id.as_deref() == Some(mcp_events[0].id.as_str())));
    assert!(context_items
        .iter()
        .any(|item| item.source_plugin_id.as_deref() == Some(plugin.id.as_str())));

    let result = messages::get_message_version_view(
        &env.db,
        &env.store,
        run.result_message_version_id
            .as_deref()
            .expect("missing workflow result message"),
    )
    .await
    .expect("failed to load workflow result message");
    assert_eq!(
        result.primary_content.text_content.as_deref(),
        Some("native flow ok")
    );

    let captured = server.captured.lock().unwrap();
    let request_text = captured.first().expect("missing captured request");
    assert!(request_text.contains("tool says hello back"));
    assert!(request_text.contains("rag remembers the user is a hero"));
    assert!(request_text.contains("mcp payload from memory server"));
    assert!(request_text.contains("plugin payload marker"));
}

async fn create_workflow_def(
    env: &TestEnv,
    name: &str,
) -> buyu_lib::domain::workflows::WorkflowDefDetail {
    let created = workflows::create_workflow_def(
        &env.db,
        &CreateWorkflowDefInput {
            name: name.to_string(),
            description: Some("workflow test".to_string()),
            enabled: true,
            sort_order: 0,
            config_json: json!({ "retry_policy": "none" }),
        },
    )
    .await
    .expect("failed to create workflow def");
    workflows::update_workflow_def(
        &env.db,
        &created.summary.id,
        &UpdateWorkflowDefInput {
            name: created.summary.name.clone(),
            description: created.summary.description.clone(),
            enabled: true,
            sort_order: 0,
            config_json: json!({ "retry_policy": "manual" }),
        },
    )
    .await
    .expect("failed to update workflow def")
}

fn workflow_node(node_key: &str, node_type: &str) -> WorkflowNodeInput {
    WorkflowNodeInput {
        node_key: node_key.to_string(),
        name: Some(node_key.to_string()),
        node_type: node_type.to_string(),
        agent_id: None,
        plugin_id: None,
        preset_id: None,
        lorebook_id: None,
        user_profile_id: None,
        api_channel_id: None,
        api_channel_model_id: None,
        workspace_mode: "inherit".to_string(),
        history_read_mode: "full".to_string(),
        summary_write_mode: "none".to_string(),
        message_write_mode: "none".to_string(),
        visible_output_mode: "final_only".to_string(),
        config_json: json!({}),
    }
}

fn find_node_id(
    nodes: &[buyu_lib::domain::workflows::WorkflowNodeDetail],
    node_key: &str,
) -> String {
    nodes
        .iter()
        .find(|item| item.node_key == node_key)
        .unwrap_or_else(|| panic!("missing workflow node '{node_key}'"))
        .id
        .clone()
}

fn text_input(text: &str) -> ContentWriteInput {
    ContentWriteInput {
        content_type: ContentType::Text,
        mime_type: Some("text/plain".to_string()),
        text_content: Some(text.to_string()),
        source_file_path: None,
        primary_storage_uri: None,
        size_bytes_hint: None,
        preview_text: None,
        config_json: json!({}),
    }
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer.windows(4).position(|window| window == b"\r\n\r\n")
}

fn parse_content_length(header_bytes: &[u8]) -> usize {
    let header_text = String::from_utf8_lossy(header_bytes);
    header_text
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            if name.trim().eq_ignore_ascii_case("content-length") {
                value.trim().parse::<usize>().ok()
            } else {
                None
            }
        })
        .unwrap_or(0)
}
