use std::{env, fs, path::PathBuf};

use buyu_lib::db::pool;
use buyu_lib::domain::agents::CreateAgentInput;
use buyu_lib::domain::api_channels::{CreateApiChannelInput, UpsertApiChannelModelInput};
use buyu_lib::domain::common::ChannelBindingInput;
use buyu_lib::domain::content::{ContentType, ContentWriteInput};
use buyu_lib::domain::conversations::{ConversationParticipantInput, CreateConversationInput};
use buyu_lib::domain::messages::{
    ContextPolicy, CreateMessageInput, GenerateReplyInput, MessageRole, ViewerPolicy,
};
use buyu_lib::providers::ProviderRegistry;
use buyu_lib::services::{
    agents, api_channels, content_store, conversations, generation, messages,
};
use serde_json::json;

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

#[tokio::test]
#[ignore = "manual live integration requiring real API credentials"]
async fn live_fetch_models_and_generate_reply() {
    let base_url = env::var("BUYU_LIVE_BASE_URL").expect("missing BUYU_LIVE_BASE_URL");
    let api_key = env::var("BUYU_LIVE_API_KEY").expect("missing BUYU_LIVE_API_KEY");
    let model_id = env::var("BUYU_LIVE_MODEL").expect("missing BUYU_LIVE_MODEL");

    let env = TestEnv::new().await;
    let provider = env
        .providers
        .get("openai_compatible")
        .expect("missing openai_compatible provider");

    let channel = api_channels::create_channel(
        &env.db,
        &CreateApiChannelInput {
            name: "Live".to_string(),
            channel_type: "openai_compatible".to_string(),
            base_url,
            auth_type: "bearer".to_string(),
            api_key: Some(api_key),
            models_endpoint: Some("/v1/models".to_string()),
            chat_endpoint: Some("/v1/chat/completions".to_string()),
            stream_endpoint: Some("/v1/chat/completions".to_string()),
            models_mode: "hybrid".to_string(),
            enabled: true,
            sort_order: 0,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create live channel");

    let models = provider
        .list_models(&channel)
        .await
        .expect("failed to fetch real models");
    assert!(
        models.iter().any(|item| item.model_id == model_id),
        "target model was not returned by remote models endpoint"
    );

    let channel_model = api_channels::upsert_channel_model(
        &env.db,
        &UpsertApiChannelModelInput {
            channel_id: channel.id.clone(),
            model_id: model_id.clone(),
            display_name: Some(model_id.clone()),
            model_type: Some("chat".to_string()),
            context_window: None,
            max_output_tokens: None,
            capabilities_json: json!({}),
            pricing_json: json!({}),
            default_parameters_json: json!({"temperature": 0.2}),
            sort_order: 0,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to upsert live model");

    let agent = agents::create_agent(
        &env.db,
        &env.store,
        &CreateAgentInput {
            name: "Live Agent".to_string(),
            title: None,
            description_content: Some(text_input("A concise assistant.")),
            personality_content: None,
            scenario_content: None,
            example_messages_content: None,
            main_prompt_override_content: Some(text_input("请简洁回答。")),
            post_history_instructions_content: None,
            character_note_content: None,
            creator_notes_content: None,
            character_note_depth: None,
            character_note_role: None,
            talkativeness: 30,
            avatar_uri: None,
            creator_name: None,
            character_version: None,
            enabled: true,
            sort_order: 0,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create live agent");

    let conversation = conversations::create_conversation(
        &env.db,
        &CreateConversationInput {
            title: "Live Conversation".to_string(),
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
                    display_name: Some("Live Agent".to_string()),
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
            channel_model_id: Some(channel_model.id.clone()),
            binding_type: "active".to_string(),
            enabled: true,
            sort_order: 0,
            config_json: json!({}),
        }],
    )
    .await
    .expect("failed to bind live channel");

    let user_message = messages::create_user_message(
        &env.db,
        &env.store,
        &CreateMessageInput {
            conversation_id: conversation.summary.id.clone(),
            author_participant_id: human_participant_id,
            role: MessageRole::User,
            reply_to_node_id: None,
            order_after_node_id: None,
            primary_content: text_input("请只回复四个字：测试通过"),
            context_policy: ContextPolicy::Full,
            viewer_policy: ViewerPolicy::Full,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create user message");

    let reply = generation::generate_reply(
        &env.db,
        &env.store,
        &env.providers,
        &GenerateReplyInput {
            conversation_id: conversation.summary.id.clone(),
            responder_participant_id: agent_participant_id,
            trigger_message_version_id: Some(user_message.version_id),
            override_api_channel_id: None,
            override_api_channel_model_id: None,
            request_parameters_json: Some(json!({ "temperature": 0.2 })),
            create_hidden_message: false,
        },
    )
    .await
    .expect("failed to generate real reply");

    let text = reply
        .primary_content
        .text_content
        .clone()
        .unwrap_or_default();
    assert!(
        !text.trim().is_empty(),
        "live reply text should not be empty"
    );
    assert_eq!(reply.api_channel_id.as_deref(), Some(channel.id.as_str()));
    assert_eq!(
        reply.api_channel_model_id.as_deref(),
        Some(channel_model.id.as_str())
    );
    assert!(reply.generation_run_id.is_some());
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
