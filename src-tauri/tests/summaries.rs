use std::{
    env, fs,
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
};

use buyu_lib::db::pool;
use buyu_lib::domain::agents::CreateAgentInput;
use buyu_lib::domain::api_channels::{
    CreateApiChannelInput, UpdateApiChannelInput, UpsertApiChannelModelInput,
};
use buyu_lib::domain::common::ChannelBindingInput;
use buyu_lib::domain::content::{ContentType, ContentWriteInput};
use buyu_lib::domain::conversations::{ConversationParticipantInput, CreateConversationInput};
use buyu_lib::domain::messages::{
    BuildGenerationContextInput, ContextPolicy, CreateMessageInput, MessageRole, ViewerPolicy,
};
use buyu_lib::domain::summaries::{SummaryTargetKind, SummaryUsageScope, UpsertSummaryUsageInput};
use buyu_lib::providers::ProviderRegistry;
use buyu_lib::services::{
    agents, api_channels, content_store, context_builder, conversations, messages, summaries,
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
                    if find_header_end(&buffer).is_some() {
                        break;
                    }
                }

                captured_clone
                    .lock()
                    .unwrap()
                    .push(String::from_utf8_lossy(&buffer).to_string());

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
async fn summary_generation_and_context_replacement_smoke() {
    let env = TestEnv::new().await;
    let server_one = MockHttpServer::new(json!({
        "choices": [{
            "message": { "content": "summary one" },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 9,
            "completion_tokens": 4,
            "total_tokens": 13
        }
    }));

    let channel = api_channels::create_channel(
        &env.db,
        &CreateApiChannelInput {
            name: "Main".to_string(),
            channel_type: "openai_compatible".to_string(),
            base_url: server_one.base_url.clone(),
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
            max_output_tokens: Some(4096),
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
            title: "Summary Thread".to_string(),
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

    let first = messages::create_user_message(
        &env.db,
        &env.store,
        &CreateMessageInput {
            conversation_id: conversation.summary.id.clone(),
            author_participant_id: participant_id.clone(),
            role: MessageRole::User,
            reply_to_node_id: None,
            order_after_node_id: None,
            primary_content: text_input("The hero met a dragon at dawn."),
            context_policy: ContextPolicy::Full,
            viewer_policy: ViewerPolicy::Full,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create first message");

    let second = messages::create_user_message(
        &env.db,
        &env.store,
        &CreateMessageInput {
            conversation_id: conversation.summary.id.clone(),
            author_participant_id: participant_id.clone(),
            role: MessageRole::User,
            reply_to_node_id: Some(first.node_id.clone()),
            order_after_node_id: Some(first.node_id.clone()),
            primary_content: text_input("The hero decided to negotiate instead of fighting."),
            context_policy: ContextPolicy::Full,
            viewer_policy: ViewerPolicy::Full,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create second message");

    let first_summary =
        summaries::generate_node_summary(&env.db, &env.store, &env.providers, &first.node_id, None)
            .await
            .expect("failed to generate first summary");
    assert_eq!(
        first_summary.content.text_content.as_deref(),
        Some("summary one")
    );

    let server_two = MockHttpServer::new(json!({
        "choices": [{
            "message": { "content": "summary two" },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 5,
            "total_tokens": 15
        }
    }));
    let _updated_channel = api_channels::update_channel(
        &env.db,
        &channel.id,
        &UpdateApiChannelInput {
            name: channel.name.clone(),
            channel_type: channel.channel_type.clone(),
            base_url: server_two.base_url.clone(),
            auth_type: channel.auth_type.clone(),
            api_key: channel.api_key.clone(),
            models_endpoint: channel.models_endpoint.clone(),
            chat_endpoint: channel.chat_endpoint.clone(),
            stream_endpoint: channel.stream_endpoint.clone(),
            models_mode: channel.models_mode.clone(),
            enabled: channel.enabled,
            sort_order: channel.sort_order,
            config_json: channel.config_json.clone(),
        },
    )
    .await
    .expect("failed to update channel");

    let second_summary =
        summaries::generate_node_summary(&env.db, &env.store, &env.providers, &first.node_id, None)
            .await
            .expect("failed to generate second summary");
    assert_eq!(
        second_summary.content.text_content.as_deref(),
        Some("summary two")
    );
    assert!(second_summary.version_index > first_summary.version_index);

    let groups = summaries::list_summary_groups(&env.db, &env.store, &conversation.summary.id)
        .await
        .expect("failed to list summary groups");
    assert_eq!(groups.len(), 1);
    assert_eq!(
        groups[0]
            .active_version
            .as_ref()
            .map(|item| item.id.as_str()),
        Some(second_summary.id.as_str())
    );

    let switched =
        summaries::switch_active_summary(&env.db, &env.store, &groups[0].id, &first_summary.id)
            .await
            .expect("failed to switch active summary");
    assert_eq!(switched.id, first_summary.id);

    summaries::upsert_summary_usage(
        &env.db,
        &UpsertSummaryUsageInput {
            usage_id: None,
            summary_group_id: groups[0].id.clone(),
            summary_version_id: None,
            usage_scope: SummaryUsageScope::Request,
            target_kind: SummaryTargetKind::MessageVersion,
            target_message_version_id: Some(first.version_id.clone()),
            target_start_node_id: None,
            target_end_node_id: None,
            conversation_id: Some(conversation.summary.id.clone()),
            activation_mode: buyu_lib::domain::summaries::SummaryActivationMode::Manual,
            replace_from_node_id: None,
            replace_after_message_count: None,
            replace_after_total_bytes: None,
            enabled: true,
            priority: 100,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to upsert summary usage");

    let built = context_builder::build_generation_context(
        &env.db,
        &env.store,
        &BuildGenerationContextInput {
            conversation_id: conversation.summary.id.clone(),
            responder_participant_id: participant_id,
            trigger_message_version_id: Some(second.version_id.clone()),
            override_api_channel_id: None,
            override_api_channel_model_id: None,
            request_parameters_json: None,
        },
    )
    .await
    .expect("failed to build context");

    let first_item = built
        .items
        .iter()
        .find(|item| item.source_message_version_id.as_deref() == Some(first.version_id.as_str()))
        .expect("missing first message context item");
    assert_eq!(
        first_item.source_summary_version_id.as_deref(),
        Some(first_summary.id.as_str())
    );
    assert_eq!(
        first_item.rendered_content.text_content.as_deref(),
        Some("summary one")
    );

    let second_item = built
        .items
        .iter()
        .find(|item| item.source_message_version_id.as_deref() == Some(second.version_id.as_str()))
        .expect("missing second message context item");
    assert_eq!(second_item.source_summary_version_id, None);
    assert_eq!(
        second_item.rendered_content.text_content.as_deref(),
        Some("The hero decided to negotiate instead of fighting.")
    );

    let captured_one = server_one.captured.lock().unwrap();
    assert!(captured_one
        .first()
        .expect("missing first summary request")
        .contains("The hero met a dragon at dawn."));
    let captured_two = server_two.captured.lock().unwrap();
    assert!(captured_two
        .first()
        .expect("missing second summary request")
        .contains("The hero met a dragon at dawn."));
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
