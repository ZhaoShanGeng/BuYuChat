use std::{env, fs, path::PathBuf};

use buyu_lib::db::pool;
use buyu_lib::domain::content::{ContentType, ContentWriteInput};
use buyu_lib::domain::variables::{
    CreateVariableDefInput, CreateVariableLockInput, DeleteVariableValueInput,
    ReleaseVariableLockInput, SetVariableValueInput, VariableLockKind, VariableScopeType,
    VariableUnlockPolicy, VariableValueType,
};
use buyu_lib::services::{content_store, variables};
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
async fn variable_events_delete_and_restore() {
    let env = TestEnv::new().await;

    let variable_def = variables::create_variable_def(
        &env.db,
        &CreateVariableDefInput {
            var_key: "note.current".to_string(),
            name: "Current Note".to_string(),
            value_type: VariableValueType::String,
            scope_type: VariableScopeType::Conversation,
            namespace: "notes".to_string(),
            is_user_editable: true,
            is_plugin_editable: false,
            ai_can_create: false,
            ai_can_update: false,
            ai_can_delete: false,
            ai_can_lock: false,
            ai_can_unlock_own_lock: false,
            ai_can_unlock_any_lock: false,
            default_json: serde_json::Value::Null,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create variable def");

    let created = variables::set_value(
        &env.db,
        &env.store,
        &SetVariableValueInput {
            variable_def_id: variable_def.id.clone(),
            scope_type: VariableScopeType::Conversation,
            scope_id: "conv-1".to_string(),
            value_json: json!("first"),
            value_content: None,
            source_message_version_id: None,
            updated_by_kind: "user".to_string(),
            updated_by_ref_id: Some("user-1".to_string()),
        },
    )
    .await
    .expect("failed to create value");
    assert_eq!(created.event_no, 1);
    assert!(created.source_message_version_id.is_none());

    let updated = variables::set_value(
        &env.db,
        &env.store,
        &SetVariableValueInput {
            variable_def_id: variable_def.id.clone(),
            scope_type: VariableScopeType::Conversation,
            scope_id: "conv-1".to_string(),
            value_json: json!("second"),
            value_content: None,
            source_message_version_id: None,
            updated_by_kind: "user".to_string(),
            updated_by_ref_id: Some("user-1".to_string()),
        },
    )
    .await
    .expect("failed to update value");
    assert_eq!(updated.event_no, 2);

    let deleted = variables::delete_value(
        &env.db,
        &env.store,
        &DeleteVariableValueInput {
            variable_def_id: variable_def.id.clone(),
            scope_type: VariableScopeType::Conversation,
            scope_id: "conv-1".to_string(),
            source_message_version_id: None,
            updated_by_kind: "user".to_string(),
            updated_by_ref_id: Some("user-1".to_string()),
        },
    )
    .await
    .expect("failed to delete value");
    assert!(deleted.is_deleted);

    let hidden = variables::get_value(
        &env.db,
        &env.store,
        &variable_def.id,
        VariableScopeType::Conversation,
        "conv-1",
        false,
    )
    .await
    .expect("failed to load active value");
    assert!(hidden.is_none());

    let deleted_visible = variables::get_value(
        &env.db,
        &env.store,
        &variable_def.id,
        VariableScopeType::Conversation,
        "conv-1",
        true,
    )
    .await
    .expect("failed to load deleted value")
    .expect("missing deleted value");
    assert!(deleted_visible.is_deleted);

    let events = variables::list_events(&env.db, &env.store, &deleted_visible.id)
        .await
        .expect("failed to list events");
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].event_no, 3);
    assert_eq!(events[1].value_json, json!("second"));
    assert_eq!(events[2].value_json, json!("first"));

    let restored = variables::restore_value_event(
        &env.db,
        &env.store,
        &deleted_visible.id,
        &events[2].id,
        "user",
        Some("user-1"),
    )
    .await
    .expect("failed to restore event");
    assert!(!restored.is_deleted);
    assert_eq!(restored.value_json, json!("first"));
    assert_eq!(restored.event_no, 4);
}

#[tokio::test]
async fn variable_ai_permissions_and_locks() {
    let env = TestEnv::new().await;

    let variable_def = variables::create_variable_def(
        &env.db,
        &CreateVariableDefInput {
            var_key: "state.reply".to_string(),
            name: "Reply State".to_string(),
            value_type: VariableValueType::Json,
            scope_type: VariableScopeType::MessageVersion,
            namespace: "state".to_string(),
            is_user_editable: true,
            is_plugin_editable: false,
            ai_can_create: true,
            ai_can_update: true,
            ai_can_delete: false,
            ai_can_lock: true,
            ai_can_unlock_own_lock: true,
            ai_can_unlock_any_lock: false,
            default_json: json!({}),
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create ai variable def");

    let created = variables::set_value(
        &env.db,
        &env.store,
        &SetVariableValueInput {
            variable_def_id: variable_def.id.clone(),
            scope_type: VariableScopeType::MessageVersion,
            scope_id: "msg-v1".to_string(),
            value_json: json!({"status": "draft"}),
            value_content: None,
            source_message_version_id: None,
            updated_by_kind: "ai".to_string(),
            updated_by_ref_id: Some("assistant-a".to_string()),
        },
    )
    .await
    .expect("ai should be allowed to create value");
    assert_eq!(created.event_no, 1);

    let lock = variables::create_lock(
        &env.db,
        &CreateVariableLockInput {
            variable_def_id: variable_def.id.clone(),
            scope_type: VariableScopeType::MessageVersion,
            scope_id: "msg-v1".to_string(),
            lock_kind: VariableLockKind::Update,
            owner_kind: "ai".to_string(),
            owner_ref_id: Some("assistant-a".to_string()),
            unlock_policy: VariableUnlockPolicy::Owner,
            config_json: json!({"reason": "freeze"}),
        },
    )
    .await
    .expect("ai should be allowed to lock value");

    let blocked = variables::set_value(
        &env.db,
        &env.store,
        &SetVariableValueInput {
            variable_def_id: variable_def.id.clone(),
            scope_type: VariableScopeType::MessageVersion,
            scope_id: "msg-v1".to_string(),
            value_json: json!({"status": "mutated"}),
            value_content: None,
            source_message_version_id: None,
            updated_by_kind: "ai".to_string(),
            updated_by_ref_id: Some("assistant-a".to_string()),
        },
    )
    .await;
    assert!(blocked.is_err());

    let locks = variables::list_locks(&env.db, &created.id)
        .await
        .expect("failed to list locks");
    assert_eq!(locks.len(), 1);
    assert_eq!(locks[0].id, lock.id);

    variables::release_lock(
        &env.db,
        &ReleaseVariableLockInput {
            variable_lock_id: lock.id.clone(),
            released_by_kind: "ai".to_string(),
            released_by_ref_id: Some("assistant-a".to_string()),
        },
    )
    .await
    .expect("ai should be allowed to unlock its own lock");

    let updated = variables::set_value(
        &env.db,
        &env.store,
        &SetVariableValueInput {
            variable_def_id: variable_def.id.clone(),
            scope_type: VariableScopeType::MessageVersion,
            scope_id: "msg-v1".to_string(),
            value_json: json!({"status": "updated"}),
            value_content: None,
            source_message_version_id: None,
            updated_by_kind: "ai".to_string(),
            updated_by_ref_id: Some("assistant-a".to_string()),
        },
    )
    .await
    .expect("ai should be allowed to update after unlocking");
    assert_eq!(updated.event_no, 2);

    let delete_attempt = variables::delete_value(
        &env.db,
        &env.store,
        &DeleteVariableValueInput {
            variable_def_id: variable_def.id.clone(),
            scope_type: VariableScopeType::MessageVersion,
            scope_id: "msg-v1".to_string(),
            source_message_version_id: None,
            updated_by_kind: "ai".to_string(),
            updated_by_ref_id: Some("assistant-a".to_string()),
        },
    )
    .await;
    assert!(delete_attempt.is_err());
}

#[tokio::test]
async fn variable_content_ref_scope_round_trip() {
    let env = TestEnv::new().await;

    let variable_def = variables::create_variable_def(
        &env.db,
        &CreateVariableDefInput {
            var_key: "memory.block".to_string(),
            name: "Memory Block".to_string(),
            value_type: VariableValueType::ContentRef,
            scope_type: VariableScopeType::Node,
            namespace: "memory".to_string(),
            is_user_editable: true,
            is_plugin_editable: true,
            ai_can_create: false,
            ai_can_update: false,
            ai_can_delete: false,
            ai_can_lock: false,
            ai_can_unlock_own_lock: false,
            ai_can_unlock_any_lock: false,
            default_json: serde_json::Value::Null,
            config_json: json!({}),
        },
    )
    .await
    .expect("failed to create content ref variable");

    let value = variables::set_value(
        &env.db,
        &env.store,
        &SetVariableValueInput {
            variable_def_id: variable_def.id.clone(),
            scope_type: VariableScopeType::Node,
            scope_id: "node-1".to_string(),
            value_json: serde_json::Value::Null,
            value_content: Some(ContentWriteInput {
                content_type: ContentType::Json,
                mime_type: Some("application/json".to_string()),
                text_content: Some("{\"memories\":[\"a\",\"b\"]}".to_string()),
                source_file_path: None,
                primary_storage_uri: None,
                size_bytes_hint: None,
                preview_text: None,
                config_json: json!({}),
            }),
            source_message_version_id: None,
            updated_by_kind: "plugin".to_string(),
            updated_by_ref_id: Some("plugin-1".to_string()),
        },
    )
    .await
    .expect("failed to set content ref variable");

    let loaded = variables::get_value(
        &env.db,
        &env.store,
        &variable_def.id,
        VariableScopeType::Node,
        "node-1",
        false,
    )
    .await
    .expect("failed to reload content ref value")
    .expect("missing content ref value");
    assert!(loaded.value_content.is_some());
    assert_eq!(
        loaded
            .value_content
            .as_ref()
            .and_then(|content| content.text_content.as_deref()),
        Some("{\"memories\":[\"a\",\"b\"]}")
    );
    assert_eq!(loaded.id, value.id);
}
