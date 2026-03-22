use std::{env, fs, path::PathBuf};

use buyu_lib::db::{pool, repos::agents as agent_repo, repos::messages as message_repo};
use buyu_lib::db::repos::{conversations as conversation_repo, workflows as workflow_repo};
use buyu_lib::domain::agents::CreateAgentInput;
use buyu_lib::domain::common::ResourceBindingInput;
use buyu_lib::domain::content::{ContentType, ContentWriteInput};
use buyu_lib::domain::conversations::{ConversationParticipantInput, CreateConversationInput};
use buyu_lib::domain::messages::{ContextPolicy, CreateMessageInput, MessageRole, ViewerPolicy};
use buyu_lib::domain::user_profiles::CreateUserProfileInput;
use buyu_lib::services::{agents, content, content_store, conversations, messages, user_profiles};
use buyu_lib::support::error::AppError;
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
async fn deleting_user_profile_cleans_references_before_delete() {
    let env = TestEnv::new().await;

    let profile = user_profiles::create_user_profile(
        &env.db,
        &env.store,
        &CreateUserProfileInput {
            name: "Persona".to_string(),
            title: Some("Lead".to_string()),
            description_content: Some(text_input("Persona description")),
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

    agents::replace_default_user_profiles(
        &env.db,
        &agent.summary.id,
        &[ResourceBindingInput {
            resource_id: profile.summary.id.clone(),
            binding_type: "default".to_string(),
            enabled: true,
            sort_order: 0,
            config_json: json!({}),
        }],
    )
    .await
    .expect("failed to bind agent user profile");

    let conversation = conversations::create_conversation(
        &env.db,
        &CreateConversationInput {
            title: "Thread".to_string(),
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
    .expect("failed to bind conversation user profile");

    let human_participant_id = conversation
        .participants
        .iter()
        .find(|participant| participant.participant_type == "human")
        .map(|participant| participant.id.clone())
        .expect("missing human participant");
    let agent_participant_id = conversation
        .participants
        .iter()
        .find(|participant| participant.participant_type == "agent")
        .map(|participant| participant.id.clone())
        .expect("missing agent participant");

    let user_message = messages::create_user_message(
        &env.db,
        &env.store,
        &CreateMessageInput {
            conversation_id: conversation.summary.id.clone(),
            author_participant_id: human_participant_id,
            role: MessageRole::User,
            reply_to_node_id: None,
            order_after_node_id: None,
            primary_content: text_input("hello"),
            context_policy: ContextPolicy::Full,
            viewer_policy: ViewerPolicy::Full,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create user message");

    let generation_run = message_repo::create_generation_run(
        &env.db,
        &message_repo::CreateGenerationRunRecord {
            conversation_id: &conversation.summary.id,
            trigger_node_id: Some(&user_message.node_id),
            trigger_message_version_id: Some(&user_message.version_id),
            responder_participant_id: Some(&agent_participant_id),
            api_channel_id: None,
            api_channel_model_id: None,
            preset_id: None,
            preset_source_scope: None,
            lorebook_id: None,
            lorebook_source_scope: None,
            user_profile_id: Some(&profile.summary.id),
            user_profile_source_scope: Some("conversation"),
            api_channel_source_scope: None,
            api_channel_model_source_scope: None,
            run_type: "reply",
            request_parameters_json: "{}",
            request_payload_content_id: None,
        },
    )
    .await
    .expect("failed to create generation run");

    let rendered_content = content::create_content(&env.db, &env.store, &text_input("context item"))
        .await
        .expect("failed to create rendered content");

    let mut tx = env.db.begin().await.expect("failed to start tx");
    message_repo::replace_generation_run_context_items(
        &mut tx,
        &generation_run.id,
        &[message_repo::GenerationRunContextItemRecord {
            sequence_no: 0,
            send_role: "system",
            rendered_content_id: &rendered_content.content_id,
            source_kind: "user_profile",
            source_message_node_id: None,
            source_message_version_id: None,
            source_summary_version_id: None,
            source_preset_entry_id: None,
            source_lorebook_entry_id: None,
            source_user_profile_id: Some(&profile.summary.id),
            source_agent_id: None,
            source_agent_greeting_id: None,
            source_tool_invocation_id: None,
            source_rag_ref_id: None,
            source_mcp_event_id: None,
            source_plugin_id: None,
            included_in_request: true,
            config_json: "{}",
        }],
    )
    .await
    .expect("failed to write generation context items");
    tx.commit().await.expect("failed to commit tx");

    let workflow_def = workflow_repo::create_workflow_def(
        &env.db,
        &workflow_repo::CreateWorkflowDefRecord {
            name: "Workflow",
            description: None,
            enabled: true,
            sort_order: 0,
            config_json: "{}",
        },
    )
    .await
    .expect("failed to create workflow def");

    let mut tx = env.db.begin().await.expect("failed to start workflow tx");
    let nodes = workflow_repo::replace_workflow_def_nodes(
        &mut tx,
        &workflow_def.id,
        &[workflow_repo::WorkflowDefNodeRecord {
            node_key: "reply",
            name: Some("Reply"),
            node_type: "chat",
            agent_id: Some(&agent.summary.id),
            plugin_id: None,
            preset_id: None,
            lorebook_id: None,
            user_profile_id: Some(&profile.summary.id),
            api_channel_id: None,
            api_channel_model_id: None,
            workspace_mode: "inherit",
            history_read_mode: "auto",
            summary_write_mode: "none",
            message_write_mode: "none",
            visible_output_mode: "final_only",
            config_json: "{}",
        }],
    )
    .await
    .expect("failed to create workflow node");
    tx.commit().await.expect("failed to commit workflow tx");

    let workflow_run = workflow_repo::create_workflow_run(
        &env.db,
        &workflow_repo::CreateWorkflowRunRecord {
            workflow_def_id: &workflow_def.id,
            trigger_conversation_id: Some(&conversation.summary.id),
            workspace_conversation_id: Some(&conversation.summary.id),
            workspace_mode: "inherit",
            trigger_message_version_id: Some(&user_message.version_id),
            entry_node_id: Some(&nodes[0].id),
            status: "pending",
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

    let write_content = content::create_content(&env.db, &env.store, &text_input("write payload"))
        .await
        .expect("failed to create workflow write content");

    let workflow_write = workflow_repo::create_workflow_run_write(
        &env.db,
        &workflow_repo::CreateWorkflowRunWriteRecord {
            workflow_run_id: &workflow_run.id,
            workflow_run_node_execution_id: None,
            write_kind: "user_profile",
            apply_mode: "update",
            content_id: &write_content.content_id,
            target_conversation_id: None,
            target_message_node_id: None,
            target_summary_group_id: None,
            target_lorebook_entry_id: None,
            target_preset_entry_id: None,
            target_agent_id: None,
            target_user_profile_id: Some(&profile.summary.id),
            target_plugin_id: None,
            target_slot: Some("profile"),
            visible_to_user: false,
            config_json: "{}",
        },
    )
    .await
    .expect("failed to create workflow write");

    user_profiles::delete_user_profile(&env.db, &profile.summary.id)
        .await
        .expect("failed to delete user profile");

    let deleted = buyu_lib::db::repos::user_profiles::get_user_profile(&env.db, &profile.summary.id)
        .await;
    assert!(matches!(
        deleted,
        Err(AppError::NotFound {
            entity: "user_profile",
            ..
        })
    ));

    let agent_bindings = agent_repo::list_agent_user_profile_bindings(&env.db, &agent.summary.id)
        .await
        .expect("failed to list agent bindings");
    assert!(agent_bindings.is_empty());

    let conversation_bindings = conversation_repo::list_conversation_user_profile_bindings(
        &env.db,
        &conversation.summary.id,
    )
    .await
    .expect("failed to list conversation bindings");
    assert!(conversation_bindings.is_empty());

    let generation_run = message_repo::get_generation_run(&env.db, &generation_run.id)
        .await
        .expect("failed to reload generation run");
    assert!(generation_run.user_profile_id.is_none());
    assert!(generation_run.user_profile_source_scope.is_none());

    let context_items =
        message_repo::list_generation_run_context_items(&env.db, &generation_run.id)
            .await
            .expect("failed to reload context items");
    assert_eq!(context_items.len(), 1);
    assert!(context_items[0].source_user_profile_id.is_none());

    let workflow_nodes = workflow_repo::list_workflow_def_nodes(&env.db, &workflow_def.id)
        .await
        .expect("failed to reload workflow nodes");
    assert_eq!(workflow_nodes.len(), 1);
    assert!(workflow_nodes[0].user_profile_id.is_none());

    let workflow_writes = workflow_repo::list_workflow_run_writes(&env.db, &workflow_run.id)
        .await
        .expect("failed to reload workflow writes");
    assert_eq!(workflow_writes.len(), 1);
    assert_eq!(workflow_writes[0].id, workflow_write.id);
    assert!(workflow_writes[0].target_user_profile_id.is_none());
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
