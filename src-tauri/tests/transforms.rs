use std::{env, fs, path::PathBuf};

use buyu_lib::db::pool;
use buyu_lib::domain::content::{ContentType, ContentWriteInput};
use buyu_lib::domain::conversations::CreateConversationInput;
use buyu_lib::domain::transforms::{
    ApplyTransformsInput, CreateTransformPipelineInput, TransformBindingInput, TransformStage,
    TransformStepInput, UpdateTransformPipelineInput,
};
use buyu_lib::services::{content, content_store, conversations, transforms};
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
async fn transform_pipeline_crud_and_replace_steps() {
    let env = TestEnv::new().await;

    let description = content::create_content(
        &env.db,
        &env.store,
        &ContentWriteInput {
            content_type: ContentType::Text,
            mime_type: Some("text/plain".to_string()),
            text_content: Some("Pipeline description".to_string()),
            source_file_path: None,
            primary_storage_uri: None,
            size_bytes_hint: None,
            preview_text: None,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create description content");

    let pipeline = transforms::create_transform_pipeline(
        &env.db,
        &env.store,
        &CreateTransformPipelineInput {
            name: "Viewer cleaner".to_string(),
            pipeline_key: "viewer.clean".to_string(),
            pipeline_kind: "regex".to_string(),
            description_content_id: Some(description.content_id.clone()),
            enabled: true,
            sort_order: 10,
            config_json: json!({"max_depth": 6}),
        },
    )
    .await
    .expect("failed to create pipeline");
    assert_eq!(pipeline.pipeline_key, "viewer.clean");
    assert!(pipeline.description_content.is_some());

    let steps = transforms::replace_transform_steps(
        &env.db,
        &pipeline.id,
        &[
            TransformStepInput {
                step_order: 0,
                step_type: "regex_replace".to_string(),
                pattern: Some("foo".to_string()),
                replacement_template: Some("bar".to_string()),
                regex_flags: "".to_string(),
                max_replacements: None,
                stop_on_match: false,
                child_pipeline_id: None,
                config_json: json!({}),
            },
            TransformStepInput {
                step_order: 1,
                step_type: "regex_match_gate".to_string(),
                pattern: Some("bar".to_string()),
                replacement_template: None,
                regex_flags: "".to_string(),
                max_replacements: None,
                stop_on_match: false,
                child_pipeline_id: None,
                config_json: json!({}),
            },
        ],
    )
    .await
    .expect("failed to replace steps");
    assert_eq!(steps.len(), 2);

    let updated = transforms::update_transform_pipeline(
        &env.db,
        &env.store,
        &pipeline.id,
        &UpdateTransformPipelineInput {
            name: "Viewer cleaner v2".to_string(),
            pipeline_kind: "composite".to_string(),
            description_content_id: Some(description.content_id.clone()),
            enabled: false,
            sort_order: 20,
            config_json: json!({"max_depth": 8}),
        },
    )
    .await
    .expect("failed to update pipeline");
    assert_eq!(updated.name, "Viewer cleaner v2");
    assert!(!updated.enabled);

    let listed = transforms::list_transform_pipelines(&env.db, &env.store)
        .await
        .expect("failed to list pipelines");
    assert_eq!(listed.len(), 1);

    let listed_steps = transforms::list_transform_steps(&env.db, &pipeline.id)
        .await
        .expect("failed to list transform steps");
    assert_eq!(listed_steps.len(), 2);
}

#[tokio::test]
async fn transform_binding_validation_and_stage_selection() {
    let env = TestEnv::new().await;

    let pipeline = transforms::create_transform_pipeline(
        &env.db,
        &env.store,
        &CreateTransformPipelineInput {
            name: "Request cleaner".to_string(),
            pipeline_key: "request.clean".to_string(),
            pipeline_kind: "regex".to_string(),
            description_content_id: None,
            enabled: true,
            sort_order: 0,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create pipeline");

    transforms::replace_transform_steps(
        &env.db,
        &pipeline.id,
        &[TransformStepInput {
            step_order: 0,
            step_type: "regex_replace".to_string(),
            pattern: Some("alpha".to_string()),
            replacement_template: Some("beta".to_string()),
            regex_flags: "".to_string(),
            max_replacements: None,
            stop_on_match: false,
            child_pipeline_id: None,
            config_json: json!({}),
        }],
    )
    .await
    .expect("failed to replace steps");

    let invalid = transforms::create_transform_binding(
        &env.db,
        &TransformBindingInput {
            pipeline_id: pipeline.id.clone(),
            conversation_id: Some("conv-1".to_string()),
            agent_id: Some("agent-1".to_string()),
            preset_id: None,
            workflow_def_node_id: None,
            apply_viewer: true,
            apply_request: false,
            apply_file: false,
            enabled: true,
            sort_order: 0,
            config_json: json!({}),
        },
    )
    .await;
    assert!(invalid.is_err());

    let conversation = conversations::create_conversation(
        &env.db,
        &CreateConversationInput {
            title: "Transform conversation".to_string(),
            description: None,
            conversation_mode: "chat".to_string(),
            archived: false,
            pinned: false,
            participants: Vec::new(),
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create conversation");

    let binding = transforms::create_transform_binding(
        &env.db,
        &TransformBindingInput {
            pipeline_id: pipeline.id.clone(),
            conversation_id: Some(conversation.summary.id.clone()),
            agent_id: None,
            preset_id: None,
            workflow_def_node_id: None,
            apply_viewer: false,
            apply_request: true,
            apply_file: false,
            enabled: true,
            sort_order: 5,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create binding");
    assert!(binding.apply_request);

    let source = content::create_content(
        &env.db,
        &env.store,
        &ContentWriteInput {
            content_type: ContentType::Text,
            mime_type: Some("text/plain".to_string()),
            text_content: Some("alpha".to_string()),
            source_file_path: None,
            primary_storage_uri: None,
            size_bytes_hint: None,
            preview_text: None,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create source content");

    let viewer_result = transforms::apply_transforms(
        &env.db,
        &env.store,
        &ApplyTransformsInput {
            stage: TransformStage::Viewer,
            conversation_id: Some(conversation.summary.id.clone()),
            agent_id: None,
            preset_id: None,
            workflow_def_node_id: None,
            source_content: source.clone(),
            generation_run_id: None,
            workflow_run_id: None,
        },
    )
    .await
    .expect("viewer transforms should succeed");
    assert!(!viewer_result.changed);

    let request_result = transforms::apply_transforms(
        &env.db,
        &env.store,
        &ApplyTransformsInput {
            stage: TransformStage::Request,
            conversation_id: Some(conversation.summary.id.clone()),
            agent_id: None,
            preset_id: None,
            workflow_def_node_id: None,
            source_content: source,
            generation_run_id: None,
            workflow_run_id: None,
        },
    )
    .await
    .expect("request transforms should succeed");
    assert!(request_result.changed);
    assert_eq!(request_result.content.text_content.as_deref(), Some("beta"));
}

#[tokio::test]
async fn transform_nested_viewer_pipeline_and_file_stage_round_trip() {
    let env = TestEnv::new().await;

    let conversation = conversations::create_conversation(
        &env.db,
        &CreateConversationInput {
            title: "Nested conversation".to_string(),
            description: None,
            conversation_mode: "chat".to_string(),
            archived: false,
            pinned: false,
            participants: Vec::new(),
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create conversation");

    let child = transforms::create_transform_pipeline(
        &env.db,
        &env.store,
        &CreateTransformPipelineInput {
            name: "Strip secret".to_string(),
            pipeline_key: "strip.secret".to_string(),
            pipeline_kind: "regex".to_string(),
            description_content_id: None,
            enabled: true,
            sort_order: 0,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create child pipeline");
    transforms::replace_transform_steps(
        &env.db,
        &child.id,
        &[TransformStepInput {
            step_order: 0,
            step_type: "regex_remove".to_string(),
            pattern: Some("\\[secret\\].*?\\[/secret\\]".to_string()),
            replacement_template: None,
            regex_flags: "s".to_string(),
            max_replacements: None,
            stop_on_match: false,
            child_pipeline_id: None,
            config_json: json!({}),
        }],
    )
    .await
    .expect("failed to replace child steps");

    let parent = transforms::create_transform_pipeline(
        &env.db,
        &env.store,
        &CreateTransformPipelineInput {
            name: "Viewer clean".to_string(),
            pipeline_key: "viewer.clean.main".to_string(),
            pipeline_kind: "composite".to_string(),
            description_content_id: None,
            enabled: true,
            sort_order: 0,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create parent pipeline");
    transforms::replace_transform_steps(
        &env.db,
        &parent.id,
        &[
            TransformStepInput {
                step_order: 0,
                step_type: "regex_replace".to_string(),
                pattern: Some("foo".to_string()),
                replacement_template: Some("bar".to_string()),
                regex_flags: "".to_string(),
                max_replacements: None,
                stop_on_match: false,
                child_pipeline_id: None,
                config_json: json!({}),
            },
            TransformStepInput {
                step_order: 1,
                step_type: "pipeline_ref".to_string(),
                pattern: None,
                replacement_template: None,
                regex_flags: "".to_string(),
                max_replacements: None,
                stop_on_match: false,
                child_pipeline_id: Some(child.id.clone()),
                config_json: json!({}),
            },
        ],
    )
    .await
    .expect("failed to replace parent steps");

    transforms::create_transform_binding(
        &env.db,
        &TransformBindingInput {
            pipeline_id: parent.id.clone(),
            conversation_id: Some(conversation.summary.id.clone()),
            agent_id: None,
            preset_id: None,
            workflow_def_node_id: None,
            apply_viewer: true,
            apply_request: false,
            apply_file: false,
            enabled: true,
            sort_order: 1,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to bind viewer pipeline");

    let source = content::create_content(
        &env.db,
        &env.store,
        &ContentWriteInput {
            content_type: ContentType::Text,
            mime_type: Some("text/plain".to_string()),
            text_content: Some("foo [secret]hidden[/secret]".to_string()),
            source_file_path: None,
            primary_storage_uri: None,
            size_bytes_hint: None,
            preview_text: None,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create source content");

    let viewer_result = transforms::apply_transforms(
        &env.db,
        &env.store,
        &ApplyTransformsInput {
            stage: TransformStage::Viewer,
            conversation_id: Some(conversation.summary.id.clone()),
            agent_id: None,
            preset_id: None,
            workflow_def_node_id: None,
            source_content: source.clone(),
            generation_run_id: None,
            workflow_run_id: None,
        },
    )
    .await
    .expect("failed to apply viewer transforms");
    assert!(viewer_result.changed);
    assert_eq!(viewer_result.content.text_content.as_deref(), Some("bar "));
    assert_eq!(viewer_result.applied_pipeline_ids.len(), 2);

    let original = content::get_content(&env.db, &env.store, &source.content_id, true)
        .await
        .expect("failed to reload original");
    assert_eq!(
        original.text_content.as_deref(),
        Some("foo [secret]hidden[/secret]")
    );

    let file_pipeline = transforms::create_transform_pipeline(
        &env.db,
        &env.store,
        &CreateTransformPipelineInput {
            name: "File clean".to_string(),
            pipeline_key: "file.clean".to_string(),
            pipeline_kind: "regex".to_string(),
            description_content_id: None,
            enabled: true,
            sort_order: 2,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create file pipeline");

    transforms::replace_transform_steps(
        &env.db,
        &file_pipeline.id,
        &[TransformStepInput {
            step_order: 0,
            step_type: "regex_replace".to_string(),
            pattern: Some("foo".to_string()),
            replacement_template: Some("dog".to_string()),
            regex_flags: "".to_string(),
            max_replacements: None,
            stop_on_match: false,
            child_pipeline_id: None,
            config_json: json!({}),
        }],
    )
    .await
    .expect("failed to replace file steps");

    transforms::create_transform_binding(
        &env.db,
        &TransformBindingInput {
            pipeline_id: file_pipeline.id.clone(),
            conversation_id: Some(conversation.summary.id.clone()),
            agent_id: None,
            preset_id: None,
            workflow_def_node_id: None,
            apply_viewer: false,
            apply_request: false,
            apply_file: true,
            enabled: true,
            sort_order: 3,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to bind file pipeline");

    let file_result = transforms::apply_transforms(
        &env.db,
        &env.store,
        &ApplyTransformsInput {
            stage: TransformStage::File,
            conversation_id: Some(conversation.summary.id.clone()),
            agent_id: None,
            preset_id: None,
            workflow_def_node_id: None,
            source_content: source.clone(),
            generation_run_id: None,
            workflow_run_id: None,
        },
    )
    .await
    .expect("failed to apply file transforms");
    assert!(file_result.changed);
    assert_ne!(file_result.content.content_id, source.content_id);
    assert_eq!(
        file_result.content.text_content.as_deref(),
        Some("dog [secret]hidden[/secret]")
    );
}
