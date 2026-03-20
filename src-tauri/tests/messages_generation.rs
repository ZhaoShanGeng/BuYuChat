use std::{
    env, fs,
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
};

use buyu_lib::db::{pool, repos::messages as message_repo};
use buyu_lib::domain::agents::CreateAgentInput;
use buyu_lib::domain::api_channels::{CreateApiChannelInput, UpsertApiChannelModelInput};
use buyu_lib::domain::common::{ChannelBindingInput, ResourceBindingInput};
use buyu_lib::domain::content::{ContentType, ContentWriteInput};
use buyu_lib::domain::conversations::{ConversationParticipantInput, CreateConversationInput};
use buyu_lib::domain::lorebooks::{
    CreateLorebookEntryInput, CreateLorebookInput, LorebookMatchInput,
};
use buyu_lib::domain::messages::{
    BuildGenerationContextInput, ContextPolicy, CreateMessageInput, GenerateReplyInput,
    GenerationStreamEventKind, MessageRole, ViewerPolicy,
};
use buyu_lib::domain::native_capabilities::{
    CreateMcpEventInput, CreateRagRefInput, FinishToolInvocationInput, StartToolInvocationInput,
};
use buyu_lib::domain::plugins::CreatePluginInput;
use buyu_lib::domain::presets::{CreatePresetEntryInput, CreatePresetInput};
use buyu_lib::domain::user_profiles::CreateUserProfileInput;
use buyu_lib::extensions::runtime::PluginRuntime;
use buyu_lib::providers::ProviderRegistry;
use buyu_lib::services::{
    agents, api_channels, content_store, context_builder, conversations, generation, lorebooks,
    mcp, messages, plugins, presets, rag, tool_invocations, user_profiles,
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

struct MockSseServer {
    addr: SocketAddr,
    base_url: String,
    captured: Arc<Mutex<Vec<String>>>,
    handle: Option<thread::JoinHandle<()>>,
}

impl MockSseServer {
    fn new(chunks: Vec<Value>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind mock sse server");
        let addr = listener.local_addr().expect("failed to read mock sse addr");
        let captured = Arc::new(Mutex::new(Vec::new()));
        let captured_clone = Arc::clone(&captured);
        let sse_body = chunks
            .into_iter()
            .map(|chunk| format!("data: {}\n\n", chunk))
            .chain(std::iter::once("data: [DONE]\n\n".to_string()))
            .collect::<String>();

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
                    "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{}",
                    sse_body.len(),
                    sse_body
                );
                stream
                    .write_all(response.as_bytes())
                    .expect("failed to write sse response");
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

impl Drop for MockSseServer {
    fn drop(&mut self) {
        let _ = TcpStream::connect(self.addr);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

#[tokio::test]
async fn messages_context_generation_smoke() {
    let env = TestEnv::new().await;
    let plugin_runtime = PluginRuntime::new();
    let server = MockHttpServer::new(json!({
        "choices": [{
            "message": { "content": "ok from mock" },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 11,
            "completion_tokens": 7,
            "total_tokens": 18
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
            default_parameters_json: json!({"temperature": 0.4}),
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
            capabilities_json: json!(["message_context"]),
            permissions_json: json!({}),
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create plugin");

    let preset = presets::create_preset(
        &env.db,
        &env.store,
        &CreatePresetInput {
            name: "Chat".to_string(),
            description: None,
            enabled: true,
            sort_order: 0,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create preset");
    let _preset_entry = presets::create_entry(
        &env.db,
        &env.store,
        &CreatePresetEntryInput {
            preset_id: preset.preset.id.clone(),
            name: "System Intro".to_string(),
            role: MessageRole::System,
            primary_content: text_input("You are concise."),
            position_type: "prepend".to_string(),
            list_order: 0,
            depth: None,
            depth_order: 0,
            triggers_json: json!({}),
            enabled: true,
            is_pinned: false,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create preset entry");

    let lorebook = lorebooks::create_lorebook(
        &env.db,
        &env.store,
        &CreateLorebookInput {
            name: "World".to_string(),
            description: None,
            scan_depth: 2,
            token_budget: Some(1024),
            insertion_strategy: "sorted_evenly".to_string(),
            enabled: true,
            sort_order: 0,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create lorebook");
    let lorebook_entry = lorebooks::create_entry(
        &env.db,
        &env.store,
        &CreateLorebookEntryInput {
            lorebook_id: lorebook.lorebook.id.clone(),
            title: Some("Greeting".to_string()),
            primary_content: text_input("The user often says hello first."),
            activation_strategy: "keyword".to_string(),
            keyword_logic: "any".to_string(),
            insertion_position: "append".to_string(),
            insertion_order: 0,
            insertion_depth: None,
            insertion_role: Some(MessageRole::System),
            outlet_name: None,
            entry_scope: "conversation".to_string(),
            enabled: true,
            sort_order: 0,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create lorebook entry");
    lorebooks::replace_keys(&env.db, &lorebook_entry.id, &["hello".to_string()])
        .await
        .expect("failed to set lorebook keys");

    let profile = user_profiles::create_user_profile(
        &env.db,
        &env.store,
        &CreateUserProfileInput {
            name: "Hero".to_string(),
            title: None,
            description_content: Some(text_input("The user is a brave hero.")),
            avatar_uri: None,
            injection_position: "prompt_manager".to_string(),
            injection_depth: Some(1),
            injection_role: Some(MessageRole::System),
            enabled: true,
            sort_order: 0,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create user profile");

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
            post_history_instructions_content: Some(text_input("Do not ramble.")),
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
            title: "Thread".to_string(),
            description: None,
            conversation_mode: "single".to_string(),
            archived: false,
            pinned: false,
            participants: vec![ConversationParticipantInput {
                agent_id: Some(agent.summary.id.clone()),
                display_name: Some("Nora".to_string()),
                participant_type: "agent".to_string(),
                enabled: true,
                sort_order: 0,
                config_json: json!({}),
            }],
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create conversation");

    conversations::replace_presets(
        &env.db,
        &conversation.summary.id,
        &[ResourceBindingInput {
            resource_id: preset.preset.id.clone(),
            binding_type: "active".to_string(),
            enabled: true,
            sort_order: 0,
            config_json: json!({}),
        }],
    )
    .await
    .expect("failed to bind preset");
    conversations::replace_lorebooks(
        &env.db,
        &conversation.summary.id,
        &[ResourceBindingInput {
            resource_id: lorebook.lorebook.id.clone(),
            binding_type: "active".to_string(),
            enabled: true,
            sort_order: 0,
            config_json: json!({}),
        }],
    )
    .await
    .expect("failed to bind lorebook");
    conversations::replace_user_profiles(
        &env.db,
        &conversation.summary.id,
        &[ResourceBindingInput {
            resource_id: profile.summary.id.clone(),
            binding_type: "active".to_string(),
            enabled: true,
            sort_order: 0,
            config_json: json!({}),
        }],
    )
    .await
    .expect("failed to bind user profile");
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

    let participant_id = conversation.participants[0].id.clone();

    let user_message = messages::create_user_message(
        &env.db,
        &env.store,
        &CreateMessageInput {
            conversation_id: conversation.summary.id.clone(),
            author_participant_id: participant_id.clone(),
            role: MessageRole::User,
            reply_to_node_id: None,
            order_after_node_id: None,
            primary_content: text_input("hello there"),
            context_policy: ContextPolicy::Full,
            viewer_policy: ViewerPolicy::Full,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create user message");

    let seed_run = message_repo::create_generation_run(
        &env.db,
        &message_repo::CreateGenerationRunRecord {
            conversation_id: &conversation.summary.id,
            trigger_node_id: Some(&user_message.node_id),
            trigger_message_version_id: Some(&user_message.version_id),
            responder_participant_id: Some(&participant_id),
            api_channel_id: Some(&channel.id),
            api_channel_model_id: Some(&model.id),
            preset_id: Some(&preset.preset.id),
            preset_source_scope: Some("conversation"),
            lorebook_id: Some(&lorebook.lorebook.id),
            lorebook_source_scope: Some("conversation"),
            user_profile_id: Some(&profile.summary.id),
            user_profile_source_scope: Some("conversation"),
            api_channel_source_scope: Some("conversation"),
            api_channel_model_source_scope: Some("conversation"),
            run_type: "seed_native_capabilities",
            request_parameters_json: "{}",
            request_payload_content_id: None,
        },
    )
    .await
    .expect("failed to create seed generation run");

    let tool = tool_invocations::start_tool_invocation(
        &env.db,
        &env.store,
        &StartToolInvocationInput {
            generation_run_id: Some(seed_run.id.clone()),
            workflow_run_node_execution_id: None,
            message_version_id: Some(user_message.version_id.clone()),
            tool_kind: "builtin".to_string(),
            tool_name: "memory_lookup".to_string(),
            plugin_id: Some(plugin.id.clone()),
            request_content: Some(text_input("lookup memory for hello")),
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to start tool invocation");
    let tool = tool_invocations::finish_tool_invocation(
        &env.db,
        &env.store,
        &tool.id,
        &FinishToolInvocationInput {
            status: "succeeded".to_string(),
            response_content: Some(text_input("tool says hello back")),
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to finish tool invocation");

    let rag_ref = rag::record_rag_ref(
        &env.db,
        &env.store,
        &CreateRagRefInput {
            generation_run_id: Some(seed_run.id.clone()),
            workflow_run_node_execution_id: None,
            source_uri: Some("memory://hero-sheet".to_string()),
            document_title: Some("Hero Sheet".to_string()),
            chunk_key: Some("hero-sheet#1".to_string()),
            score: Some(0.88),
            excerpt_content: Some(text_input("rag remembers the user is a hero")),
            included_in_request: true,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to record rag ref");

    let mcp_event = mcp::record_mcp_event(
        &env.db,
        &env.store,
        &CreateMcpEventInput {
            generation_run_id: Some(seed_run.id.clone()),
            workflow_run_node_execution_id: None,
            server_name: "memory-server".to_string(),
            event_kind: "tool_result".to_string(),
            method_name: Some("lookup".to_string()),
            payload_content: Some(text_input("mcp payload from memory server")),
            status: "succeeded".to_string(),
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to record mcp event");

    let _attachment = messages::append_attachment(
        &env.db,
        &env.store,
        &buyu_lib::domain::messages::AddAttachmentInput {
            message_version_id: user_message.version_id.clone(),
            plugin_id: None,
            ref_role: "attachment".to_string(),
            sort_order: 0,
            content: text_input("attachment note"),
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to add attachment");

    let _tool_ref = messages::append_attachment(
        &env.db,
        &env.store,
        &buyu_lib::domain::messages::AddAttachmentInput {
            message_version_id: user_message.version_id.clone(),
            plugin_id: Some(plugin.id.clone()),
            ref_role: "tool_response".to_string(),
            sort_order: 1,
            content: text_input("tool says hello back"),
            config_json: json!({
                "tool_invocation_id": tool.id,
            }),
        },
    )
    .await
    .expect("failed to attach tool response");

    let _rag_ref_content = messages::append_attachment(
        &env.db,
        &env.store,
        &buyu_lib::domain::messages::AddAttachmentInput {
            message_version_id: user_message.version_id.clone(),
            plugin_id: None,
            ref_role: "rag_excerpt".to_string(),
            sort_order: 2,
            content: text_input("rag remembers the user is a hero"),
            config_json: json!({
                "rag_ref_id": rag_ref.id,
            }),
        },
    )
    .await
    .expect("failed to attach rag excerpt");

    let _mcp_ref = messages::append_attachment(
        &env.db,
        &env.store,
        &buyu_lib::domain::messages::AddAttachmentInput {
            message_version_id: user_message.version_id.clone(),
            plugin_id: None,
            ref_role: "mcp_payload".to_string(),
            sort_order: 3,
            content: text_input("mcp payload from memory server"),
            config_json: json!({
                "mcp_event_id": mcp_event.id,
            }),
        },
    )
    .await
    .expect("failed to attach mcp payload");

    let _plugin_ref = messages::append_attachment(
        &env.db,
        &env.store,
        &buyu_lib::domain::messages::AddAttachmentInput {
            message_version_id: user_message.version_id.clone(),
            plugin_id: Some(plugin.id.clone()),
            ref_role: "plugin_payload".to_string(),
            sort_order: 4,
            content: text_input("plugin payload marker"),
            config_json: json!({
                "plugin_id": plugin.id,
            }),
        },
    )
    .await
    .expect("failed to attach plugin payload");

    let built = context_builder::build_generation_context(
        &env.db,
        &env.store,
        &BuildGenerationContextInput {
            conversation_id: conversation.summary.id.clone(),
            responder_participant_id: participant_id.clone(),
            trigger_message_version_id: Some(user_message.version_id.clone()),
            override_api_channel_id: None,
            override_api_channel_model_id: None,
            request_parameters_json: Some(json!({"top_p": 0.9})),
        },
    )
    .await
    .expect("failed to build generation context");

    assert!(built
        .items
        .iter()
        .any(|item| item.source_kind == "preset_entry"));
    assert!(built
        .items
        .iter()
        .any(|item| item.source_kind == "user_profile"));
    assert!(built.items.iter().any(|item| item.source_kind == "agent"));
    assert!(built
        .items
        .iter()
        .any(|item| item.source_kind == "lorebook_entry"));
    assert!(built
        .items
        .iter()
        .any(|item| item.source_kind == "message_version"));
    assert!(built
        .items
        .iter()
        .any(|item| item.source_kind == "tool_invocation"
            && item.source_tool_invocation_id.as_deref() == Some(tool.id.as_str())));
    assert!(built.items.iter().any(|item| item.source_kind == "rag_ref"
        && item.source_rag_ref_id.as_deref() == Some(rag_ref.id.as_str())));
    assert!(built
        .items
        .iter()
        .any(|item| item.source_kind == "mcp_event"
            && item.source_mcp_event_id.as_deref() == Some(mcp_event.id.as_str())));
    assert!(built
        .items
        .iter()
        .any(|item| item.source_kind == "plugin_content"
            && item.source_plugin_id.as_deref() == Some(plugin.id.as_str())));
    assert_eq!(built.request_parameters_json["temperature"], json!(0.4));
    assert_eq!(built.request_parameters_json["top_p"], json!(0.9));

    let generated = generation::generate_reply(
        &env.db,
        &env.store,
        &env.providers,
        &GenerateReplyInput {
            conversation_id: conversation.summary.id.clone(),
            responder_participant_id: participant_id,
            trigger_message_version_id: Some(user_message.version_id.clone()),
            override_api_channel_id: None,
            override_api_channel_model_id: None,
            request_parameters_json: Some(json!({"temperature": 0.2})),
            create_hidden_message: false,
        },
    )
    .await
    .expect("failed to generate reply");

    assert_eq!(
        generated.primary_content.text_content.as_deref(),
        Some("ok from mock")
    );
    assert_eq!(
        generated.api_channel_id.as_deref(),
        Some(channel.id.as_str())
    );
    assert_eq!(
        generated.api_channel_model_id.as_deref(),
        Some(model.id.as_str())
    );
    assert_eq!(generated.total_tokens, Some(18));
    assert!(generated.generation_run_id.is_some());

    let visible = messages::list_visible_messages(&env.db, &env.store, &conversation.summary.id)
        .await
        .expect("failed to list visible messages");
    assert_eq!(visible.len(), 2);

    let run = message_repo::get_generation_run(
        &env.db,
        generated
            .generation_run_id
            .as_deref()
            .expect("missing generation run id"),
    )
    .await
    .expect("failed to read generation run");
    assert_eq!(run.status, "succeeded");

    let captured = server.captured.lock().unwrap();
    let request_text = captured.first().expect("missing captured request");
    assert!(request_text.contains("/chat/completions"));
    assert!(request_text.contains("You are concise."));
    assert!(request_text.contains("hello there"));
    assert!(
        request_text.contains("tool says hello back"),
        "captured request did not contain tool payload:\n{request_text}"
    );
    assert!(request_text.contains("rag remembers the user is a hero"));
    assert!(request_text.contains("mcp payload from memory server"));
    assert!(request_text.contains("plugin payload marker"));

    let run_context = message_repo::list_generation_run_context_items(
        &env.db,
        generated
            .generation_run_id
            .as_deref()
            .expect("missing generation run id"),
    )
    .await
    .expect("failed to list generation run context items");
    assert!(run_context
        .iter()
        .any(|item| item.source_tool_invocation_id.as_deref() == Some(tool.id.as_str())));
    assert!(run_context
        .iter()
        .any(|item| item.source_rag_ref_id.as_deref() == Some(rag_ref.id.as_str())));
    assert!(run_context
        .iter()
        .any(|item| item.source_mcp_event_id.as_deref() == Some(mcp_event.id.as_str())));
    assert!(run_context
        .iter()
        .any(|item| item.source_plugin_id.as_deref() == Some(plugin.id.as_str())));

    let matched = lorebooks::match_entries(
        &env.db,
        &env.store,
        &LorebookMatchInput {
            conversation_id: Some(conversation.summary.id),
            lorebook_id: lorebook.lorebook.id,
            recent_messages: vec!["hello there".to_string()],
            max_entries: 5,
            include_disabled: false,
        },
    )
    .await
    .expect("failed to match lorebook entries");
    assert_eq!(matched.len(), 1);
}

#[tokio::test]
async fn messages_streaming_generation_smoke() {
    let env = TestEnv::new().await;
    let server = MockSseServer::new(vec![
        json!({
            "choices": [{
                "delta": { "content": "stream " },
                "finish_reason": null
            }]
        }),
        json!({
            "choices": [{
                "delta": { "content": "reply" },
                "finish_reason": null
            }]
        }),
        json!({
            "choices": [{
                "delta": {},
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 9,
                "completion_tokens": 2,
                "total_tokens": 11
            }
        }),
    ]);

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
            default_parameters_json: json!({"temperature": 0.4}),
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
            personality_content: None,
            scenario_content: None,
            example_messages_content: None,
            main_prompt_override_content: None,
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
            title: "Thread".to_string(),
            description: None,
            conversation_mode: "single".to_string(),
            archived: false,
            pinned: false,
            participants: vec![ConversationParticipantInput {
                agent_id: Some(agent.summary.id.clone()),
                display_name: Some("Nora".to_string()),
                participant_type: "agent".to_string(),
                enabled: true,
                sort_order: 0,
                config_json: json!({}),
            }],
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create conversation");

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

    let participant_id = conversation.participants[0].id.clone();

    let user_message = messages::create_user_message(
        &env.db,
        &env.store,
        &CreateMessageInput {
            conversation_id: conversation.summary.id.clone(),
            author_participant_id: participant_id.clone(),
            role: MessageRole::User,
            reply_to_node_id: None,
            order_after_node_id: None,
            primary_content: text_input("hello stream"),
            context_policy: ContextPolicy::Full,
            viewer_policy: ViewerPolicy::Full,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create user message");

    let events = Arc::new(Mutex::new(Vec::new()));
    let events_capture = Arc::clone(&events);
    let generated = generation::generate_reply_streaming(
        &env.db,
        &env.store,
        &env.providers,
        &GenerateReplyInput {
            conversation_id: conversation.summary.id.clone(),
            responder_participant_id: participant_id,
            trigger_message_version_id: Some(user_message.version_id.clone()),
            override_api_channel_id: None,
            override_api_channel_model_id: None,
            request_parameters_json: None,
            create_hidden_message: false,
        },
        "stream-1",
        &mut |event| {
            events_capture.lock().unwrap().push(event);
            Ok(())
        },
    )
    .await
    .expect("failed to stream generate reply");

    assert_eq!(
        generated.primary_content.text_content.as_deref(),
        Some("stream reply")
    );

    let captured_events = events.lock().unwrap().clone();
    assert_eq!(
        captured_events.first().map(|item| item.kind),
        Some(GenerationStreamEventKind::Started)
    );
    assert!(captured_events
        .iter()
        .any(|item| item.kind == GenerationStreamEventKind::Delta
            && item.delta_text.as_deref() == Some("stream ")));
    assert!(captured_events
        .iter()
        .any(|item| item.kind == GenerationStreamEventKind::Delta
            && item.delta_text.as_deref() == Some("reply")));
    assert!(captured_events.iter().any(|item| {
        item.kind == GenerationStreamEventKind::Completed
            && item.message_version_id.as_deref() == Some(generated.version_id.as_str())
            && item.total_tokens == Some(11)
    }));

    let captured = server.captured.lock().unwrap();
    let request_text = captured.first().expect("missing captured request");
    assert!(request_text.contains("\"stream\":true"));
}

#[tokio::test]
async fn messages_file_like_parts_fallback_to_text_reference() {
    let env = TestEnv::new().await;
    let server = MockHttpServer::new(json!({
        "choices": [{
            "message": { "content": "ok from file refs" },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 13,
            "completion_tokens": 5,
            "total_tokens": 18
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
            default_parameters_json: json!({}),
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
            personality_content: None,
            scenario_content: None,
            example_messages_content: None,
            main_prompt_override_content: None,
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
            title: "Thread".to_string(),
            description: None,
            conversation_mode: "single".to_string(),
            archived: false,
            pinned: false,
            participants: vec![ConversationParticipantInput {
                agent_id: Some(agent.summary.id.clone()),
                display_name: Some("Nora".to_string()),
                participant_type: "agent".to_string(),
                enabled: true,
                sort_order: 0,
                config_json: json!({}),
            }],
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create conversation");

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

    let participant_id = conversation.participants[0].id.clone();

    let user_message = messages::create_user_message(
        &env.db,
        &env.store,
        &CreateMessageInput {
            conversation_id: conversation.summary.id.clone(),
            author_participant_id: participant_id.clone(),
            role: MessageRole::User,
            reply_to_node_id: None,
            order_after_node_id: None,
            primary_content: text_input("process my attachments"),
            context_policy: ContextPolicy::Full,
            viewer_policy: ViewerPolicy::Full,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create user message");

    let audio_path = env._root.join("sample-audio.bin");
    fs::write(&audio_path, vec![1_u8; 2048]).expect("failed to write audio sample");
    let video_path = env._root.join("sample-video.bin");
    fs::write(&video_path, vec![2_u8; 3072]).expect("failed to write video sample");
    let file_path = env._root.join("sample-file.bin");
    fs::write(&file_path, vec![3_u8; 1024]).expect("failed to write file sample");

    let _audio_ref = messages::append_attachment(
        &env.db,
        &env.store,
        &buyu_lib::domain::messages::AddAttachmentInput {
            message_version_id: user_message.version_id.clone(),
            plugin_id: None,
            ref_role: "audio".to_string(),
            sort_order: 0,
            content: file_input(ContentType::Audio, "audio/wav", &audio_path),
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to add audio ref");

    let _video_ref = messages::append_attachment(
        &env.db,
        &env.store,
        &buyu_lib::domain::messages::AddAttachmentInput {
            message_version_id: user_message.version_id.clone(),
            plugin_id: None,
            ref_role: "video".to_string(),
            sort_order: 1,
            content: file_input(ContentType::Video, "video/mp4", &video_path),
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to add video ref");

    let _file_ref = messages::append_attachment(
        &env.db,
        &env.store,
        &buyu_lib::domain::messages::AddAttachmentInput {
            message_version_id: user_message.version_id.clone(),
            plugin_id: None,
            ref_role: "file".to_string(),
            sort_order: 2,
            content: file_input(ContentType::File, "application/octet-stream", &file_path),
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to add file ref");

    let generated = generation::generate_reply(
        &env.db,
        &env.store,
        &env.providers,
        &GenerateReplyInput {
            conversation_id: conversation.summary.id.clone(),
            responder_participant_id: participant_id,
            trigger_message_version_id: Some(user_message.version_id.clone()),
            override_api_channel_id: None,
            override_api_channel_model_id: None,
            request_parameters_json: None,
            create_hidden_message: false,
        },
    )
    .await
    .expect("failed to generate reply with file-like refs");

    assert_eq!(
        generated.primary_content.text_content.as_deref(),
        Some("ok from file refs")
    );

    let captured = server.captured.lock().unwrap();
    let request_text = captured.first().expect("missing captured request");
    assert!(request_text.contains("[audio]"));
    assert!(request_text.contains("mime: audio/wav"));
    assert!(request_text.contains("[video]"));
    assert!(request_text.contains("mime: video/mp4"));
    assert!(request_text.contains("[file]"));
    assert!(request_text.contains("application/octet-stream"));
    assert!(request_text.contains("size_bytes: 2048"));
    assert!(request_text.contains("size_bytes: 3072"));
    assert!(request_text.contains("size_bytes: 1024"));
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

fn file_input(
    content_type: ContentType,
    mime_type: &str,
    path: &std::path::Path,
) -> ContentWriteInput {
    ContentWriteInput {
        content_type,
        mime_type: Some(mime_type.to_string()),
        text_content: None,
        source_file_path: Some(path.to_string_lossy().to_string()),
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
