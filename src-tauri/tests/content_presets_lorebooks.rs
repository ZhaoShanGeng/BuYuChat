use std::{env, fs, path::PathBuf};

use buyu_lib::db::pool;
use buyu_lib::domain::api_channels::{CreateApiChannelInput, UpsertApiChannelModelInput};
use buyu_lib::domain::common::ChannelBindingInput;
use buyu_lib::domain::content::{ContentType, ContentWriteInput};
use buyu_lib::domain::lorebooks::{
    CreateLorebookEntryInput, CreateLorebookInput, LorebookMatchInput,
};
use buyu_lib::domain::messages::MessageRole;
use buyu_lib::domain::presets::{CreatePresetEntryInput, CreatePresetInput};
use buyu_lib::services::{api_channels, content, content_store, lorebooks, presets};
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
async fn schema_and_content_service_smoke() {
    let env = TestEnv::new().await;

    let tables: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM sqlite_master
        WHERE type = 'table'
          AND name IN (
            'api_channels',
            'api_channel_models',
            'content_objects',
            'content_chunks',
            'presets',
            'preset_entries',
            'preset_channel_bindings',
            'lorebooks',
            'lorebook_entries',
            'lorebook_entry_keys'
          )
        "#,
    )
    .fetch_one(&env.db)
    .await
    .expect("failed to count tables");
    assert_eq!(
        tables, 10,
        "expected key tables to be created by migrations"
    );

    let inline = content::create_content(
        &env.db,
        &env.store,
        &ContentWriteInput {
            content_type: ContentType::Text,
            mime_type: Some("text/plain".to_string()),
            text_content: Some("hello buyu".to_string()),
            source_file_path: None,
            primary_storage_uri: None,
            size_bytes_hint: None,
            preview_text: None,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create inline content");
    assert!(matches!(
        inline.storage_kind,
        buyu_lib::domain::content::ContentStorageKind::Inline
    ));
    assert_eq!(inline.text_content.as_deref(), Some("hello buyu"));

    let chunk_text = "a".repeat((content_store::INLINE_TEXT_THRESHOLD_BYTES as usize) + 1024);
    let chunked = content::create_content(
        &env.db,
        &env.store,
        &ContentWriteInput {
            content_type: ContentType::Text,
            mime_type: Some("text/plain".to_string()),
            text_content: Some(chunk_text.clone()),
            source_file_path: None,
            primary_storage_uri: None,
            size_bytes_hint: None,
            preview_text: Some("large text".to_string()),
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create chunked content");
    assert!(matches!(
        chunked.storage_kind,
        buyu_lib::domain::content::ContentStorageKind::Chunked
    ));
    assert!(chunked.chunk_count > 0);
    assert!(chunked.text_content.is_none());

    let chunked_body = content::get_content(&env.db, &env.store, &chunked.content_id, true)
        .await
        .expect("failed to reload chunked content");
    assert_eq!(
        chunked_body.text_content.as_deref(),
        Some(chunk_text.as_str())
    );

    let file_path = env.store.root().join("source-image.bin");
    fs::write(&file_path, vec![7_u8; 8192]).expect("failed to write test file");
    let file_ref = content::create_content(
        &env.db,
        &env.store,
        &ContentWriteInput {
            content_type: ContentType::Image,
            mime_type: Some("image/png".to_string()),
            text_content: None,
            source_file_path: Some(file_path.to_string_lossy().to_string()),
            primary_storage_uri: None,
            size_bytes_hint: None,
            preview_text: Some("preview".to_string()),
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create file_ref content");
    assert!(matches!(
        file_ref.storage_kind,
        buyu_lib::domain::content::ContentStorageKind::FileRef
    ));
    let stored_path = PathBuf::from(
        file_ref
            .primary_storage_uri
            .clone()
            .expect("missing primary storage uri"),
    );
    assert!(
        stored_path.exists(),
        "imported file should exist in content store"
    );
}

#[tokio::test]
async fn preset_service_smoke() {
    let env = TestEnv::new().await;

    let channel = api_channels::create_channel(
        &env.db,
        &CreateApiChannelInput {
            name: "Main".to_string(),
            channel_type: "openai_compatible".to_string(),
            base_url: "https://example.com/v1".to_string(),
            auth_type: "bearer".to_string(),
            api_key: Some("test-key".to_string()),
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

    let preset = presets::create_preset(
        &env.db,
        &env.store,
        &CreatePresetInput {
            name: "Default".to_string(),
            description: Some("preset".to_string()),
            enabled: true,
            sort_order: 0,
            config_json: json!({"mode": "chat"}),
        },
    )
    .await
    .expect("failed to create preset");

    let entry_a = presets::create_entry(
        &env.db,
        &env.store,
        &CreatePresetEntryInput {
            preset_id: preset.preset.id.clone(),
            name: "System A".to_string(),
            role: MessageRole::System,
            primary_content: ContentWriteInput {
                content_type: ContentType::Text,
                mime_type: Some("text/plain".to_string()),
                text_content: Some("You are helpful.".to_string()),
                source_file_path: None,
                primary_storage_uri: None,
                size_bytes_hint: None,
                preview_text: None,
                config_json: json!({}),
            },
            position_type: "relative".to_string(),
            list_order: 5,
            depth: None,
            depth_order: 0,
            triggers_json: json!(["normal"]),
            enabled: true,
            is_pinned: false,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create preset entry a");

    let entry_b = presets::create_entry(
        &env.db,
        &env.store,
        &CreatePresetEntryInput {
            preset_id: preset.preset.id.clone(),
            name: "System B".to_string(),
            role: MessageRole::System,
            primary_content: ContentWriteInput {
                content_type: ContentType::Text,
                mime_type: Some("text/plain".to_string()),
                text_content: Some("Stay concise.".to_string()),
                source_file_path: None,
                primary_storage_uri: None,
                size_bytes_hint: None,
                preview_text: None,
                config_json: json!({}),
            },
            position_type: "relative".to_string(),
            list_order: 10,
            depth: None,
            depth_order: 0,
            triggers_json: json!(["normal"]),
            enabled: true,
            is_pinned: false,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create preset entry b");

    presets::reorder_entries(
        &env.db,
        &preset.preset.id,
        &[entry_b.id.clone(), entry_a.id.clone()],
    )
    .await
    .expect("failed to reorder preset entries");

    let binding = presets::bind_channel(
        &env.db,
        &preset.preset.id,
        &ChannelBindingInput {
            channel_id: channel.id.clone(),
            channel_model_id: Some(model.id.clone()),
            binding_type: "default".to_string(),
            enabled: true,
            sort_order: 0,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to bind preset channel");

    let detail = presets::get_preset_detail(&env.db, &env.store, &preset.preset.id)
        .await
        .expect("failed to reload preset detail");
    assert_eq!(detail.entries.len(), 2);
    assert_eq!(detail.entries[0].id, entry_b.id);
    assert_eq!(detail.entries[1].id, entry_a.id);
    assert_eq!(detail.channel_bindings.len(), 1);
    assert_eq!(detail.channel_bindings[0].id, binding.id);
    assert_eq!(
        detail.channel_bindings[0].channel_model_id.as_deref(),
        Some(model.id.as_str())
    );
}

#[tokio::test]
async fn lorebook_service_smoke() {
    let env = TestEnv::new().await;

    let lorebook = lorebooks::create_lorebook(
        &env.db,
        &env.store,
        &CreateLorebookInput {
            name: "World".to_string(),
            description: Some("test lorebook".to_string()),
            scan_depth: 2,
            token_budget: Some(2048),
            insertion_strategy: "sorted_evenly".to_string(),
            enabled: true,
            sort_order: 0,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create lorebook");

    let entry = lorebooks::create_entry(
        &env.db,
        &env.store,
        &CreateLorebookEntryInput {
            lorebook_id: lorebook.lorebook.id.clone(),
            title: Some("Dragon".to_string()),
            primary_content: ContentWriteInput {
                content_type: ContentType::Text,
                mime_type: Some("text/plain".to_string()),
                text_content: Some("Dragons rule the northern mountains.".to_string()),
                source_file_path: None,
                primary_storage_uri: None,
                size_bytes_hint: None,
                preview_text: None,
                config_json: json!({}),
            },
            activation_strategy: "keyword".to_string(),
            keyword_logic: "and_any".to_string(),
            insertion_position: "after_char_defs".to_string(),
            insertion_order: 100,
            insertion_depth: None,
            insertion_role: Some(MessageRole::System),
            outlet_name: None,
            entry_scope: "shared".to_string(),
            enabled: true,
            sort_order: 0,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create lorebook entry");

    lorebooks::replace_keys(
        &env.db,
        &entry.id,
        &[
            "dragon".to_string(),
            "dragon".to_string(),
            "north".to_string(),
            " ".to_string(),
        ],
    )
    .await
    .expect("failed to replace lorebook keys");

    let detail = lorebooks::get_lorebook_detail(&env.db, &env.store, &lorebook.lorebook.id)
        .await
        .expect("failed to reload lorebook detail");
    assert_eq!(detail.entries.len(), 1);
    assert_eq!(detail.entries[0].keys.len(), 2);

    let matched = lorebooks::match_entries(
        &env.db,
        &env.store,
        &LorebookMatchInput {
            conversation_id: None,
            lorebook_id: lorebook.lorebook.id,
            recent_messages: vec!["A dragon appears in the north.".to_string()],
            max_entries: 10,
            include_disabled: false,
        },
    )
    .await
    .expect("failed to match lorebook entries");
    assert_eq!(matched.len(), 1);
    assert_eq!(matched[0].lorebook_entry_id, entry.id);
    assert!(matched[0].matched_keys.iter().any(|key| key == "dragon"));
}
