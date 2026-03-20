use std::{env, fs, path::PathBuf};

use buyu_lib::db::pool;
use buyu_lib::domain::agents::{
    AddAgentMediaInput, CreateAgentGreetingInput, CreateAgentInput, UpdateAgentGreetingInput,
    UpdateAgentInput,
};
use buyu_lib::domain::api_channels::{CreateApiChannelInput, UpsertApiChannelModelInput};
use buyu_lib::domain::common::{ChannelBindingInput, ResourceBindingInput};
use buyu_lib::domain::content::{ContentType, ContentWriteInput};
use buyu_lib::domain::lorebooks::CreateLorebookInput;
use buyu_lib::domain::messages::MessageRole;
use buyu_lib::domain::presets::CreatePresetInput;
use buyu_lib::domain::user_profiles::{CreateUserProfileInput, UpdateUserProfileInput};
use buyu_lib::services::{agents, api_channels, content_store, lorebooks, presets, user_profiles};
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
async fn user_profile_and_agent_service_smoke() {
    let env = TestEnv::new().await;

    let profile = user_profiles::create_user_profile(
        &env.db,
        &env.store,
        &CreateUserProfileInput {
            name: "Hero".to_string(),
            title: Some("Lead".to_string()),
            description_content: Some(ContentWriteInput {
                content_type: ContentType::Markdown,
                mime_type: Some("text/markdown".to_string()),
                text_content: Some("I am the protagonist.".to_string()),
                source_file_path: None,
                primary_storage_uri: None,
                size_bytes_hint: None,
                preview_text: None,
                config_json: json!({}),
            }),
            avatar_uri: Some("avatar://hero".to_string()),
            injection_position: "prompt_manager".to_string(),
            injection_depth: Some(2),
            injection_role: Some(MessageRole::User),
            enabled: true,
            sort_order: 10,
            config_json: json!({"kind": "persona"}),
        },
    )
    .await
    .expect("failed to create user profile");
    assert_eq!(profile.summary.name, "Hero");
    assert_eq!(
        profile
            .description_content
            .as_ref()
            .and_then(|content| content.text_content.as_deref()),
        Some("I am the protagonist.")
    );

    let updated_profile = user_profiles::update_user_profile(
        &env.db,
        &env.store,
        &profile.summary.id,
        &UpdateUserProfileInput {
            name: "Heroine".to_string(),
            title: Some("Lead+".to_string()),
            description_content: None,
            avatar_uri: Some("avatar://heroine".to_string()),
            injection_position: "prompt_manager".to_string(),
            injection_depth: Some(3),
            injection_role: Some(MessageRole::Assistant),
            enabled: true,
            sort_order: 11,
            config_json: json!({"kind": "persona", "updated": true}),
        },
    )
    .await
    .expect("failed to update user profile");
    assert_eq!(updated_profile.summary.name, "Heroine");
    assert!(updated_profile.description_content.is_none());

    let preset = presets::create_preset(
        &env.db,
        &env.store,
        &CreatePresetInput {
            name: "Story".to_string(),
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
            name: "Lore".to_string(),
            description: None,
            scan_depth: 3,
            token_budget: Some(2048),
            insertion_strategy: "sorted_evenly".to_string(),
            enabled: true,
            sort_order: 0,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create lorebook");

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
            capabilities_json: json!({"reasoning": true}),
            pricing_json: json!({}),
            default_parameters_json: json!({"temperature": 0.7}),
            sort_order: 0,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create channel model");

    let agent = agents::create_agent(
        &env.db,
        &env.store,
        &CreateAgentInput {
            name: "Nora".to_string(),
            title: Some("Guide".to_string()),
            description_content: Some(ContentWriteInput {
                content_type: ContentType::Text,
                mime_type: Some("text/plain".to_string()),
                text_content: Some("A gentle guide.".to_string()),
                source_file_path: None,
                primary_storage_uri: None,
                size_bytes_hint: None,
                preview_text: None,
                config_json: json!({}),
            }),
            personality_content: Some(ContentWriteInput {
                content_type: ContentType::Text,
                mime_type: Some("text/plain".to_string()),
                text_content: Some("Warm and calm.".to_string()),
                source_file_path: None,
                primary_storage_uri: None,
                size_bytes_hint: None,
                preview_text: None,
                config_json: json!({}),
            }),
            scenario_content: None,
            example_messages_content: None,
            main_prompt_override_content: None,
            post_history_instructions_content: None,
            character_note_content: None,
            creator_notes_content: None,
            character_note_depth: Some(1),
            character_note_role: Some(MessageRole::System),
            talkativeness: 60,
            avatar_uri: Some("avatar://nora".to_string()),
            creator_name: Some("Admin".to_string()),
            character_version: Some("1.0".to_string()),
            enabled: true,
            sort_order: 0,
            config_json: json!({"theme": "guide"}),
        },
    )
    .await
    .expect("failed to create agent");
    assert_eq!(agent.summary.name, "Nora");
    assert_eq!(
        agent
            .description_content
            .as_ref()
            .and_then(|content| content.text_content.as_deref()),
        Some("A gentle guide.")
    );

    let greeting = agents::create_greeting(
        &env.db,
        &env.store,
        &agent.summary.id,
        &CreateAgentGreetingInput {
            greeting_type: "default".to_string(),
            name: Some("Welcome".to_string()),
            primary_content: ContentWriteInput {
                content_type: ContentType::Text,
                mime_type: Some("text/plain".to_string()),
                text_content: Some("Welcome to BuYu.".to_string()),
                source_file_path: None,
                primary_storage_uri: None,
                size_bytes_hint: None,
                preview_text: None,
                config_json: json!({}),
            },
            enabled: true,
            sort_order: 0,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create agent greeting");
    assert_eq!(
        greeting.primary_content.text_content.as_deref(),
        Some("Welcome to BuYu.")
    );

    let updated_greeting = agents::update_greeting(
        &env.db,
        &env.store,
        &greeting.id,
        &UpdateAgentGreetingInput {
            greeting_type: "default".to_string(),
            name: Some("Welcome+".to_string()),
            primary_content: ContentWriteInput {
                content_type: ContentType::Markdown,
                mime_type: Some("text/markdown".to_string()),
                text_content: Some("**Welcome back**".to_string()),
                source_file_path: None,
                primary_storage_uri: None,
                size_bytes_hint: None,
                preview_text: None,
                config_json: json!({}),
            },
            enabled: true,
            sort_order: 1,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to update greeting");
    assert_eq!(
        updated_greeting.primary_content.text_content.as_deref(),
        Some("**Welcome back**")
    );

    let media_path = env.store.root().join("agent-media.bin");
    fs::write(&media_path, vec![9_u8; 4096]).expect("failed to write media file");
    let media = agents::add_media(
        &env.db,
        &env.store,
        &agent.summary.id,
        &AddAgentMediaInput {
            media_type: "image".to_string(),
            name: Some("Portrait".to_string()),
            content: ContentWriteInput {
                content_type: ContentType::Image,
                mime_type: Some("image/png".to_string()),
                text_content: None,
                source_file_path: Some(media_path.to_string_lossy().to_string()),
                primary_storage_uri: None,
                size_bytes_hint: None,
                preview_text: Some("portrait".to_string()),
                config_json: json!({}),
            },
            enabled: true,
            sort_order: 0,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to add media");
    assert!(media.content.primary_storage_uri.is_some());
    assert!(media.content.text_content.is_none());

    agents::replace_default_presets(
        &env.db,
        &agent.summary.id,
        &[ResourceBindingInput {
            resource_id: preset.preset.id.clone(),
            binding_type: "default".to_string(),
            enabled: true,
            sort_order: 0,
            config_json: json!({}),
        }],
    )
    .await
    .expect("failed to bind preset");

    agents::replace_default_lorebooks(
        &env.db,
        &agent.summary.id,
        &[ResourceBindingInput {
            resource_id: lorebook.lorebook.id.clone(),
            binding_type: "default".to_string(),
            enabled: true,
            sort_order: 0,
            config_json: json!({}),
        }],
    )
    .await
    .expect("failed to bind lorebook");

    agents::replace_default_user_profiles(
        &env.db,
        &agent.summary.id,
        &[ResourceBindingInput {
            resource_id: updated_profile.summary.id.clone(),
            binding_type: "default".to_string(),
            enabled: true,
            sort_order: 0,
            config_json: json!({}),
        }],
    )
    .await
    .expect("failed to bind user profile");

    agents::replace_default_channels(
        &env.db,
        &agent.summary.id,
        &[ChannelBindingInput {
            channel_id: channel.id.clone(),
            channel_model_id: Some(model.id.clone()),
            binding_type: "default".to_string(),
            enabled: true,
            sort_order: 0,
            config_json: json!({}),
        }],
    )
    .await
    .expect("failed to bind channel");

    let detail = agents::get_agent_detail(&env.db, &env.store, &agent.summary.id)
        .await
        .expect("failed to get agent detail");
    assert_eq!(detail.greetings.len(), 1);
    assert_eq!(detail.media.len(), 1);
    assert_eq!(detail.preset_bindings.len(), 1);
    assert_eq!(detail.lorebook_bindings.len(), 1);
    assert_eq!(detail.user_profile_bindings.len(), 1);
    assert_eq!(detail.channel_bindings.len(), 1);
    assert_eq!(
        detail.channel_bindings[0].channel_model_id.as_deref(),
        Some(model.id.as_str())
    );

    let updated_agent = agents::update_agent(
        &env.db,
        &env.store,
        &agent.summary.id,
        &UpdateAgentInput {
            name: "Nora+".to_string(),
            title: Some("Guide+".to_string()),
            description_content: Some(ContentWriteInput {
                content_type: ContentType::Text,
                mime_type: Some("text/plain".to_string()),
                text_content: Some("A sharper guide.".to_string()),
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
            character_note_depth: Some(2),
            character_note_role: Some(MessageRole::Assistant),
            talkativeness: 70,
            avatar_uri: Some("avatar://nora-plus".to_string()),
            creator_name: Some("Admin".to_string()),
            character_version: Some("1.1".to_string()),
            enabled: true,
            sort_order: 2,
            config_json: json!({"theme": "guide-plus"}),
        },
    )
    .await
    .expect("failed to update agent");
    assert_eq!(updated_agent.summary.name, "Nora+");
    assert_eq!(
        updated_agent
            .description_content
            .as_ref()
            .and_then(|content| content.text_content.as_deref()),
        Some("A sharper guide.")
    );
    assert!(updated_agent.personality_content.is_none());
}
