//! 会话仓储层的 SQLite 集成测试。

mod helpers;

use buyu_lib::{
    models::{ConversationPatch, NewConversation},
    repo::conversation_repo::{ConversationRepo, SqlxConversationRepo},
    utils::ids::new_uuid_v7,
};
use sqlx::Row;

/// 向测试数据库插入一组会话绑定依赖。
async fn insert_dependencies(
    db: &sqlx::SqlitePool,
    agent_id: &str,
    channel_id: &str,
    model_id: &str,
) {
    sqlx::query(
        r#"
        INSERT INTO agents (id, name, enabled, created_at, updated_at)
        VALUES (?1, ?2, 1, 100, 100)
        "#,
    )
    .bind(agent_id)
    .bind("助手")
    .execute(db)
    .await
    .unwrap();

    sqlx::query(
        r#"
        INSERT INTO api_channels (
            id, name, channel_type, base_url, enabled, created_at, updated_at
        ) VALUES (?1, ?2, 'openai_compatible', 'https://api.openai.com', 1, 100, 100)
        "#,
    )
    .bind(channel_id)
    .bind("OpenAI")
    .execute(db)
    .await
    .unwrap();

    sqlx::query(
        r#"
        INSERT INTO api_channel_models (id, channel_id, model_id, display_name)
        VALUES (?1, ?2, 'gpt-4o', 'GPT-4o')
        "#,
    )
    .bind(model_id)
    .bind(channel_id)
    .execute(db)
    .await
    .unwrap();
}

/// 构造仓储层测试使用的会话样本。
fn sample_new_conversation(
    id: String,
    agent_id: &str,
    channel_id: &str,
    model_id: &str,
    updated_at: i64,
) -> NewConversation {
    NewConversation {
        id,
        title: "测试会话".to_string(),
        agent_id: Some(agent_id.to_string()),
        channel_id: Some(channel_id.to_string()),
        channel_model_id: Some(model_id.to_string()),
        archived: false,
        pinned: false,
        enabled_tools: None,
        created_at: updated_at,
        updated_at,
    }
}

/// 插入会话后应能正常读取详情。
#[tokio::test]
async fn test_insert_conversation_persists_bindings() {
    let state = helpers::test_state().await;
    let repo = SqlxConversationRepo::new(state.db.clone());
    let agent_id = new_uuid_v7();
    let channel_id = new_uuid_v7();
    let model_id = new_uuid_v7();

    insert_dependencies(&state.db, &agent_id, &channel_id, &model_id).await;

    let created = repo
        .insert(&sample_new_conversation(
            new_uuid_v7(),
            &agent_id,
            &channel_id,
            &model_id,
            100,
        ))
        .await
        .unwrap();

    assert_eq!(created.agent_id.as_deref(), Some(agent_id.as_str()));
    assert_eq!(created.channel_id.as_deref(), Some(channel_id.as_str()));
    assert_eq!(created.channel_model_id.as_deref(), Some(model_id.as_str()));
}

/// 会话列表应按 pinned 优先、updated_at 倒序排序。
#[tokio::test]
async fn test_list_conversations_orders_by_pinned_then_updated_at() {
    let state = helpers::test_state().await;
    let repo = SqlxConversationRepo::new(state.db.clone());
    let agent_id = new_uuid_v7();
    let channel_id = new_uuid_v7();
    let model_id = new_uuid_v7();

    insert_dependencies(&state.db, &agent_id, &channel_id, &model_id).await;

    let first = repo
        .insert(&sample_new_conversation(
            new_uuid_v7(),
            &agent_id,
            &channel_id,
            &model_id,
            100,
        ))
        .await
        .unwrap();
    let second = repo
        .insert(&sample_new_conversation(
            new_uuid_v7(),
            &agent_id,
            &channel_id,
            &model_id,
            200,
        ))
        .await
        .unwrap();

    repo.update(
        &first.id,
        &ConversationPatch {
            pinned: Some(true),
            updated_at: 300,
            ..ConversationPatch::default()
        },
    )
    .await
    .unwrap();

    let conversations = repo.list(false).await.unwrap();

    assert_eq!(conversations[0].id, first.id);
    assert_eq!(conversations[1].id, second.id);
}

/// 删除会话后，其下 message_nodes 应被级联删除。
#[tokio::test]
async fn test_delete_conversation_cascades_message_nodes() {
    let state = helpers::test_state().await;
    let repo = SqlxConversationRepo::new(state.db.clone());
    let agent_id = new_uuid_v7();
    let channel_id = new_uuid_v7();
    let model_id = new_uuid_v7();

    insert_dependencies(&state.db, &agent_id, &channel_id, &model_id).await;

    let created = repo
        .insert(&sample_new_conversation(
            new_uuid_v7(),
            &agent_id,
            &channel_id,
            &model_id,
            100,
        ))
        .await
        .unwrap();

    sqlx::query(
        r#"
        INSERT INTO message_nodes (id, conversation_id, role, order_key, created_at)
        VALUES (?1, ?2, 'user', '0000000000000100-0-aaaa1111', 100)
        "#,
    )
    .bind(new_uuid_v7())
    .bind(&created.id)
    .execute(&state.db)
    .await
    .unwrap();

    repo.delete(&created.id).await.unwrap();

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM message_nodes")
        .fetch_one(&state.db)
        .await
        .unwrap();
    assert_eq!(count, 0);
}

/// 删除模型后，会话上的 channel_model_id 应由外键自动置空。
#[tokio::test]
async fn test_model_deletion_sets_channel_model_binding_to_null() {
    let state = helpers::test_state().await;
    let repo = SqlxConversationRepo::new(state.db.clone());
    let agent_id = new_uuid_v7();
    let channel_id = new_uuid_v7();
    let model_id = new_uuid_v7();

    insert_dependencies(&state.db, &agent_id, &channel_id, &model_id).await;

    let created = repo
        .insert(&sample_new_conversation(
            new_uuid_v7(),
            &agent_id,
            &channel_id,
            &model_id,
            100,
        ))
        .await
        .unwrap();

    sqlx::query("DELETE FROM api_channel_models WHERE id = ?1")
        .bind(&model_id)
        .execute(&state.db)
        .await
        .unwrap();

    let row = sqlx::query("SELECT channel_model_id FROM conversations WHERE id = ?1")
        .bind(&created.id)
        .fetch_one(&state.db)
        .await
        .unwrap();
    let channel_model_id: Option<String> = row.try_get("channel_model_id").unwrap();

    assert_eq!(channel_model_id, None);
}
