use std::{env, fs, path::PathBuf};

use buyu_lib::db::pool;
use buyu_lib::domain::agents::CreateAgentInput;
use buyu_lib::domain::api_channels::{CreateApiChannelInput, UpsertApiChannelModelInput};
use buyu_lib::domain::common::{ChannelBindingInput, ResourceBindingInput};
use buyu_lib::domain::content::{ContentType, ContentWriteInput};
use buyu_lib::domain::conversations::{
    ConversationParticipantInput, CreateConversationInput, UpdateConversationMetaInput,
};
use buyu_lib::domain::lorebooks::CreateLorebookInput;
use buyu_lib::domain::messages::MessageRole;
use buyu_lib::domain::presets::CreatePresetInput;
use buyu_lib::domain::user_profiles::CreateUserProfileInput;
use buyu_lib::services::{
    agents, api_channels, content_store, context_builder, conversations, lorebooks, presets,
    user_profiles,
};
use serde_json::json;

struct TestEnv {
    _root: PathBuf,
    db: sqlx::SqlitePool,
    store: content_store::ContentStore,
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
        }
    }
}

#[tokio::test]
async fn conversation_service_smoke() {
    let env = TestEnv::new().await;

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

    let profile = user_profiles::create_user_profile(
        &env.db,
        &env.store,
        &CreateUserProfileInput {
            name: "Hero".to_string(),
            title: None,
            description_content: Some(ContentWriteInput {
                content_type: ContentType::Text,
                mime_type: Some("text/plain".to_string()),
                text_content: Some("Brave hero".to_string()),
                source_file_path: None,
                primary_storage_uri: None,
                size_bytes_hint: None,
                preview_text: None,
                config_json: json!({}),
            }),
            avatar_uri: None,
            injection_position: "prompt_manager".to_string(),
            injection_depth: Some(1),
            injection_role: Some(MessageRole::User),
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
            description_content: Some(ContentWriteInput {
                content_type: ContentType::Text,
                mime_type: Some("text/plain".to_string()),
                text_content: Some("Guide".to_string()),
                source_file_path: None,
                primary_storage_uri: None,
                size_bytes_hint: None,
                preview_text: None,
                config_json: json!({}),
            }),
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

    let channel = api_channels::create_channel(
        &env.db,
        &CreateApiChannelInput {
            name: "Main".to_string(),
            channel_type: "openai_compatible".to_string(),
            base_url: "https://example.com/v1".to_string(),
            auth_type: "bearer".to_string(),
            api_key: Some("key".to_string()),
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

    let conversation = conversations::create_conversation(
        &env.db,
        &CreateConversationInput {
            title: "Thread".to_string(),
            description: Some("first".to_string()),
            conversation_mode: "single".to_string(),
            archived: false,
            pinned: true,
            participants: vec![ConversationParticipantInput {
                agent_id: Some(agent.summary.id.clone()),
                display_name: Some("Nora Alias".to_string()),
                participant_type: "agent".to_string(),
                enabled: true,
                sort_order: 0,
                config_json: json!({"role": "lead"}),
            }],
            config_json: json!({"summary_mode": "auto"}),
        },
    )
    .await
    .expect("failed to create conversation");
    assert_eq!(conversation.summary.title, "Thread");
    assert_eq!(conversation.participants.len(), 1);
    assert_eq!(
        conversation.participants[0].display_name.as_deref(),
        Some("Nora Alias")
    );

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

    let renamed = conversations::rename_conversation(&env.db, &conversation.summary.id, "Thread+")
        .await
        .expect("failed to rename conversation");
    assert_eq!(renamed.summary.title, "Thread+");

    let updated = conversations::update_conversation_meta(
        &env.db,
        &conversation.summary.id,
        &UpdateConversationMetaInput {
            title: "Thread++".to_string(),
            description: Some("updated".to_string()),
            archived: true,
            pinned: false,
            config_json: json!({"summary_mode": "manual"}),
        },
    )
    .await
    .expect("failed to update conversation");
    assert_eq!(updated.summary.title, "Thread++");
    assert!(updated.summary.archived);
    assert_eq!(updated.preset_bindings.len(), 1);
    assert_eq!(updated.lorebook_bindings.len(), 1);
    assert_eq!(updated.user_profile_bindings.len(), 1);
    assert_eq!(updated.channel_bindings.len(), 1);
    assert_eq!(
        updated.channel_bindings[0].channel_model_id.as_deref(),
        Some(model.id.as_str())
    );

    conversations::replace_channels(
        &env.db,
        &conversation.summary.id,
        &[ChannelBindingInput {
            channel_id: channel.id.clone(),
            channel_model_id: Some(model.id.clone()),
            binding_type: "chat".to_string(),
            enabled: true,
            sort_order: 0,
            config_json: json!({}),
        }],
    )
    .await
    .expect("failed to bind legacy chat channel");

    let (resolved_channel, resolved_model, channel_scope, model_scope) =
        context_builder::resolve_active_channel_model(
            &env.db,
            &conversation.summary.id,
            &conversation.participants[0].id,
            None,
            None,
        )
        .await
        .expect("failed to resolve legacy chat channel binding");
    assert_eq!(resolved_channel.id, channel.id);
    assert_eq!(resolved_model.id, model.id);
    assert_eq!(channel_scope, "conversation");
    assert_eq!(model_scope, "conversation");

    let list = conversations::list_conversations(&env.db)
        .await
        .expect("failed to list conversations");
    assert_eq!(list.len(), 1);
}
