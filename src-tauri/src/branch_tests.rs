use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

use crate::commands::conversation::fork_conversation_from_message_inner;
use crate::db::{conversation, message, models::MessageRow};
use crate::providers::{LlmProvider, ProviderRegistry};
use crate::services::{chat::ChatService, versioning::VersioningService};
use crate::types::{ChatRequest, ChatResponse, ModelInfo, Role, StreamEvent, TokenUsage, ToolDef};

#[derive(Clone)]
struct MockProvider {
    name: String,
    seen_requests: Arc<Mutex<Vec<ChatRequest>>>,
}

impl MockProvider {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            seen_requests: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl LlmProvider for MockProvider {
    fn name(&self) -> &str {
        &self.name
    }

    async fn list_models(&self) -> crate::error::Result<Vec<ModelInfo>> {
        Ok(vec![ModelInfo {
            id: "mock-model".to_string(),
            name: "mock-model".to_string(),
            context_length: Some(8192),
            supports_vision: true,
            supports_function_calling: true,
        }])
    }

    async fn chat(&self, req: &ChatRequest) -> crate::error::Result<ChatResponse> {
        self.seen_requests.lock().await.push(req.clone());

        let content = req
            .messages
            .last()
            .map(|message| message.content.as_text().to_string())
            .unwrap_or_else(|| "empty".to_string());

        Ok(ChatResponse {
            content,
            tool_calls: None,
            usage: Some(TokenUsage {
                prompt_tokens: 1,
                completion_tokens: 1,
                total_tokens: 2,
            }),
            finish_reason: Some("stop".to_string()),
        })
    }

    async fn chat_stream(
        &self,
        req: &ChatRequest,
        _tx: mpsc::Sender<StreamEvent>,
    ) -> crate::error::Result<()> {
        let _ = self.chat(req).await?;
        Ok(())
    }

    fn supports_function_calling(&self) -> bool {
        true
    }

    fn format_tools(&self, _tools: &[ToolDef]) -> serde_json::Value {
        serde_json::Value::Null
    }

    async fn health_check(&self) -> crate::error::Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn send_message_includes_system_prompt() {
    let db_path = std::env::temp_dir().join(format!("omnichat-branch-test-{}.db", Uuid::now_v7()));
    let db = crate::db::init_pool(&db_path)
        .await
        .expect("db init failed");

    let providers = ProviderRegistry::new_shared();
    let provider = Arc::new(MockProvider::new("mock"));
    providers.register(provider.clone()).await;

    let conv = conversation::create(&db, "mock-model", "mock", None)
        .await
        .expect("create conversation failed");
    conversation::update_system_prompt(&db, &conv.id, Some("SYSTEM TEST"))
        .await
        .expect("update system prompt failed");

    let chat_service = ChatService::new(db.clone(), providers.clone());
    chat_service
        .send_message_no_emit(&conv.id, "hello".to_string(), None)
        .await
        .expect("send message failed");

    let requests = provider.seen_requests.lock().await;
    let request = requests.last().expect("no request captured");
    assert_eq!(request.system_prompt.as_deref(), Some("SYSTEM TEST"));
    assert_eq!(request.messages.len(), 1);

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn switching_old_branch_preserves_other_records() {
    let db_path = std::env::temp_dir().join(format!("omnichat-branch-test-{}.db", Uuid::now_v7()));
    let db = crate::db::init_pool(&db_path)
        .await
        .expect("db init failed");

    let conv = conversation::create(&db, "mock-model", "mock", None)
        .await
        .expect("create conversation failed");

    let root_group = Uuid::now_v7().to_string();
    let assistant_group = Uuid::now_v7().to_string();

    let user_v1 = MessageRow {
        id: Uuid::now_v7().to_string(),
        conversation_id: conv.id.clone(),
        parent_message_id: None,
        version_group_id: root_group.clone(),
        version_index: 1,
        is_active: false,
        role: Role::User.as_str().to_string(),
        content: Some("u1".to_string()),
        created_at: 1,
        ..Default::default()
    };
    let user_v2 = MessageRow {
        id: Uuid::now_v7().to_string(),
        conversation_id: conv.id.clone(),
        parent_message_id: None,
        version_group_id: root_group.clone(),
        version_index: 2,
        is_active: true,
        role: Role::User.as_str().to_string(),
        content: Some("u2".to_string()),
        created_at: 3,
        ..Default::default()
    };
    let assistant_v1 = MessageRow {
        id: Uuid::now_v7().to_string(),
        conversation_id: conv.id.clone(),
        parent_message_id: Some(user_v2.id.clone()),
        version_group_id: assistant_group.clone(),
        version_index: 1,
        is_active: false,
        role: Role::Assistant.as_str().to_string(),
        content: Some("a1".to_string()),
        created_at: 2,
        ..Default::default()
    };
    let assistant_v2 = MessageRow {
        id: Uuid::now_v7().to_string(),
        conversation_id: conv.id.clone(),
        parent_message_id: Some(user_v2.id.clone()),
        version_group_id: assistant_group,
        version_index: 2,
        is_active: true,
        role: Role::Assistant.as_str().to_string(),
        content: Some("a2".to_string()),
        created_at: 4,
        ..Default::default()
    };

    for row in [&user_v1, &user_v2, &assistant_v1, &assistant_v2] {
        message::insert(&db, row)
            .await
            .expect("insert message failed");
    }

    let active_before = message::list_active(&db, &conv.id)
        .await
        .expect("list active before failed");
    assert_eq!(
        active_before
            .iter()
            .map(|row| row.content.clone().unwrap_or_default())
            .collect::<Vec<_>>(),
        vec!["u2".to_string(), "a2".to_string()]
    );

    let versioning = VersioningService::new(db.clone());
    versioning
        .switch_version(&root_group, 1)
        .await
        .expect("switch version failed");

    let active_after = message::list_active(&db, &conv.id)
        .await
        .expect("list active after failed");
    assert_eq!(
        active_after
            .iter()
            .map(|row| row.content.clone().unwrap_or_default())
            .collect::<Vec<_>>(),
        vec!["u1".to_string(), "a2".to_string()]
    );

    let all_rows = message::list_all(&db, &conv.id)
        .await
        .expect("list all failed");
    assert_eq!(all_rows.len(), 4);
    assert!(all_rows.iter().any(|row| row.id == assistant_v2.id));

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn switching_message_version_does_not_switch_ancestor_chain() {
    let db_path =
        std::env::temp_dir().join(format!("omnichat-switch-version-{}.db", Uuid::now_v7()));
    let db = crate::db::init_pool(&db_path)
        .await
        .expect("db init failed");

    let conv = conversation::create(&db, "mock-model", "mock", None)
        .await
        .expect("create conversation failed");

    let user_group = Uuid::now_v7().to_string();
    let assistant_group = Uuid::now_v7().to_string();

    let user_v1 = MessageRow {
        id: Uuid::now_v7().to_string(),
        conversation_id: conv.id.clone(),
        parent_message_id: None,
        version_group_id: user_group.clone(),
        version_index: 1,
        is_active: false,
        role: Role::User.as_str().to_string(),
        content: Some("u1".to_string()),
        created_at: 1,
        ..Default::default()
    };
    let user_v2 = MessageRow {
        id: Uuid::now_v7().to_string(),
        conversation_id: conv.id.clone(),
        parent_message_id: None,
        version_group_id: user_group.clone(),
        version_index: 2,
        is_active: true,
        role: Role::User.as_str().to_string(),
        content: Some("u2".to_string()),
        created_at: 2,
        ..Default::default()
    };
    let assistant_v1 = MessageRow {
        id: Uuid::now_v7().to_string(),
        conversation_id: conv.id.clone(),
        parent_message_id: Some(user_v1.id.clone()),
        version_group_id: assistant_group.clone(),
        version_index: 1,
        is_active: false,
        role: Role::Assistant.as_str().to_string(),
        content: Some("a1".to_string()),
        created_at: 3,
        ..Default::default()
    };
    let assistant_v2 = MessageRow {
        id: Uuid::now_v7().to_string(),
        conversation_id: conv.id.clone(),
        parent_message_id: Some(user_v2.id.clone()),
        version_group_id: assistant_group,
        version_index: 2,
        is_active: true,
        role: Role::Assistant.as_str().to_string(),
        content: Some("a2".to_string()),
        created_at: 4,
        ..Default::default()
    };

    for row in [&user_v1, &user_v2, &assistant_v1, &assistant_v2] {
        message::insert(&db, row)
            .await
            .expect("insert message failed");
    }

    VersioningService::new(db.clone())
        .switch_version(&assistant_v2.version_group_id, 1)
        .await
        .expect("switch version failed");

    let active_rows = message::list_active(&db, &conv.id)
        .await
        .expect("list active failed");
    assert_eq!(
        active_rows
            .iter()
            .map(|row| row.content.clone().unwrap_or_default())
            .collect::<Vec<_>>(),
        vec!["u2".to_string(), "a1".to_string()]
    );

    let switched_row = message::get(&db, &assistant_v1.id)
        .await
        .expect("load switched row failed");
    assert_eq!(
        switched_row.parent_message_id.as_deref(),
        Some(user_v2.id.as_str())
    );

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn editing_message_preserves_following_messages() {
    let db_path = std::env::temp_dir().join(format!("omnichat-edit-test-{}.db", Uuid::now_v7()));
    let db = crate::db::init_pool(&db_path)
        .await
        .expect("db init failed");

    let conv = conversation::create(&db, "mock-model", "mock", None)
        .await
        .expect("create conversation failed");

    let user = MessageRow {
        id: Uuid::now_v7().to_string(),
        conversation_id: conv.id.clone(),
        parent_message_id: None,
        version_group_id: Uuid::now_v7().to_string(),
        version_index: 1,
        is_active: true,
        role: Role::User.as_str().to_string(),
        content: Some("before".to_string()),
        created_at: 1,
        ..Default::default()
    };
    let assistant = MessageRow {
        id: Uuid::now_v7().to_string(),
        conversation_id: conv.id.clone(),
        parent_message_id: Some(user.id.clone()),
        version_group_id: Uuid::now_v7().to_string(),
        version_index: 1,
        is_active: true,
        role: Role::Assistant.as_str().to_string(),
        content: Some("after".to_string()),
        created_at: 2,
        ..Default::default()
    };

    message::insert(&db, &user)
        .await
        .expect("insert user failed");
    message::insert(&db, &assistant)
        .await
        .expect("insert assistant failed");

    let chat = ChatService::new(db.clone(), ProviderRegistry::new_shared());
    let edited_id = chat
        .save_message_edit(&conv.id, &user.id, "edited".to_string())
        .await
        .expect("save edit failed");

    let active_rows = message::list_active(&db, &conv.id)
        .await
        .expect("list active failed");
    assert_eq!(
        active_rows
            .iter()
            .map(|row| row.content.clone().unwrap_or_default())
            .collect::<Vec<_>>(),
        vec!["edited".to_string(), "after".to_string()]
    );

    let assistant_after = message::get(&db, &assistant.id)
        .await
        .expect("load assistant failed");
    assert_eq!(
        assistant_after.parent_message_id.as_deref(),
        Some(edited_id.as_str())
    );

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn fork_conversation_keeps_all_versions_on_visible_path() {
    let db_path = std::env::temp_dir().join(format!("omnichat-fork-test-{}.db", Uuid::now_v7()));
    let db = crate::db::init_pool(&db_path)
        .await
        .expect("db init failed");

    let conv = conversation::create_with_fields(
        &db,
        "源会话",
        "mock-model",
        "mock",
        None,
        Some("fork system"),
    )
    .await
    .expect("create conversation failed");

    let user_group = Uuid::now_v7().to_string();
    let assistant_group = Uuid::now_v7().to_string();

    let user = MessageRow {
        id: Uuid::now_v7().to_string(),
        conversation_id: conv.id.clone(),
        parent_message_id: None,
        version_group_id: user_group,
        version_index: 1,
        is_active: true,
        role: Role::User.as_str().to_string(),
        content: Some("question".to_string()),
        created_at: 1,
        ..Default::default()
    };
    let assistant_v1 = MessageRow {
        id: Uuid::now_v7().to_string(),
        conversation_id: conv.id.clone(),
        parent_message_id: Some(user.id.clone()),
        version_group_id: assistant_group.clone(),
        version_index: 1,
        is_active: false,
        role: Role::Assistant.as_str().to_string(),
        content: Some("answer v1".to_string()),
        created_at: 2,
        ..Default::default()
    };
    let assistant_v2 = MessageRow {
        id: Uuid::now_v7().to_string(),
        conversation_id: conv.id.clone(),
        parent_message_id: Some(user.id.clone()),
        version_group_id: assistant_group,
        version_index: 2,
        is_active: true,
        role: Role::Assistant.as_str().to_string(),
        content: Some("answer v2".to_string()),
        created_at: 3,
        ..Default::default()
    };

    message::insert(&db, &user)
        .await
        .expect("insert user failed");
    message::insert(&db, &assistant_v1)
        .await
        .expect("insert assistant v1 failed");
    message::insert(&db, &assistant_v2)
        .await
        .expect("insert assistant v2 failed");

    let forked = fork_conversation_from_message_inner(&db, &conv.id, &assistant_v2.id)
        .await
        .expect("fork conversation failed");

    let forked_active = message::list_active(&db, &forked.id)
        .await
        .expect("list active forked failed");
    assert_eq!(
        forked_active
            .iter()
            .map(|row| row.content.clone().unwrap_or_default())
            .collect::<Vec<_>>(),
        vec!["question".to_string(), "answer v2".to_string()]
    );

    let forked_all = message::list_all(&db, &forked.id)
        .await
        .expect("list all forked failed");
    let forked_assistant_versions = forked_all
        .iter()
        .filter(|row| row.role == Role::Assistant.as_str())
        .count();
    assert_eq!(forked_assistant_versions, 2);

    let forked_conv = conversation::get(&db, &forked.id)
        .await
        .expect("load forked conversation failed");
    assert_eq!(forked_conv.system_prompt.as_deref(), Some("fork system"));

    let _ = std::fs::remove_file(db_path);
}
