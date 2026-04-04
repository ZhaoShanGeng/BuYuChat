//! 模型仓储层的 SQLite 集成测试。

mod helpers;

use buyu_lib::{
    models::{ChannelModelPatch, NewChannelModel},
    repo::model_repo::{ModelRepo, SqlxModelRepo},
    utils::ids::new_uuid_v7,
};
use sqlx::Row;

/// 向测试数据库插入一条渠道记录，供模型外键引用。
async fn insert_channel(db: &sqlx::SqlitePool, id: &str, name: &str) {
    sqlx::query(
        r#"
        INSERT INTO api_channels (
            id, name, channel_type, base_url, api_key, auth_type,
            models_endpoint, chat_endpoint, stream_endpoint, enabled, created_at, updated_at
        ) VALUES (?1, ?2, 'openai_compatible', 'https://api.openai.com', 'sk-test', 'bearer', '/v1/models', '/v1/chat/completions', '/v1/chat/completions', 1, 100, 100)
        "#,
    )
    .bind(id)
    .bind(name)
    .execute(db)
    .await
    .unwrap();
}

/// 构造仓储层测试使用的模型样本。
fn sample_new_model(channel_id: &str, model_id: &str) -> NewChannelModel {
    NewChannelModel {
        id: new_uuid_v7(),
        channel_id: channel_id.to_string(),
        model_id: model_id.to_string(),
        display_name: Some(model_id.to_uppercase()),
        context_window: Some(128_000),
        max_output_tokens: Some(16_384),
        temperature: None,
        top_p: None,
    }
}

/// 插入模型后应能按渠道读取持久化结果。
#[tokio::test]
async fn test_insert_model_persists_and_can_be_loaded() {
    let state = helpers::test_state().await;
    let repo = SqlxModelRepo::new(state.db.clone());
    let channel_id = new_uuid_v7();

    insert_channel(&state.db, &channel_id, "OpenAI").await;

    let created = repo
        .insert(&sample_new_model(&channel_id, "gpt-4o"))
        .await
        .unwrap();
    let loaded = repo
        .get_by_channel_and_id(&channel_id, &created.id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(loaded.id, created.id);
    assert_eq!(loaded.model_id, "gpt-4o");
    assert_eq!(loaded.display_name.as_deref(), Some("GPT-4O"));
}

/// 列表应只返回指定渠道模型，并按 model_id 升序稳定排序。
#[tokio::test]
async fn test_list_models_filters_by_channel_and_orders_by_model_id() {
    let state = helpers::test_state().await;
    let repo = SqlxModelRepo::new(state.db.clone());
    let channel_a = new_uuid_v7();
    let channel_b = new_uuid_v7();

    insert_channel(&state.db, &channel_a, "A").await;
    insert_channel(&state.db, &channel_b, "B").await;

    repo.insert(&sample_new_model(&channel_a, "z-model"))
        .await
        .unwrap();
    repo.insert(&sample_new_model(&channel_a, "a-model"))
        .await
        .unwrap();
    repo.insert(&sample_new_model(&channel_b, "other-model"))
        .await
        .unwrap();

    let models = repo.list_by_channel(&channel_a).await.unwrap();

    assert_eq!(models.len(), 2);
    assert_eq!(models[0].model_id, "a-model");
    assert_eq!(models[1].model_id, "z-model");
}

/// 同一渠道下重复 model_id 应触发唯一约束错误。
#[tokio::test]
async fn test_insert_model_rejects_duplicate_model_id_within_same_channel() {
    let state = helpers::test_state().await;
    let repo = SqlxModelRepo::new(state.db.clone());
    let channel_id = new_uuid_v7();

    insert_channel(&state.db, &channel_id, "OpenAI").await;

    repo.insert(&sample_new_model(&channel_id, "gpt-4o"))
        .await
        .unwrap();
    let error = repo
        .insert(&sample_new_model(&channel_id, "gpt-4o"))
        .await
        .unwrap_err();

    assert!(error.contains("UNIQUE"));
}

/// 不同渠道允许创建相同的 model_id。
#[tokio::test]
async fn test_insert_model_allows_duplicate_model_id_across_channels() {
    let state = helpers::test_state().await;
    let repo = SqlxModelRepo::new(state.db.clone());
    let channel_a = new_uuid_v7();
    let channel_b = new_uuid_v7();

    insert_channel(&state.db, &channel_a, "A").await;
    insert_channel(&state.db, &channel_b, "B").await;

    repo.insert(&sample_new_model(&channel_a, "gpt-4o"))
        .await
        .unwrap();
    let created = repo
        .insert(&sample_new_model(&channel_b, "gpt-4o"))
        .await
        .unwrap();

    assert_eq!(created.channel_id, channel_b);
    assert_eq!(created.model_id, "gpt-4o");
}

/// 更新模型时应只改动补丁中给出的字段。
#[tokio::test]
async fn test_update_model_only_changes_supplied_fields() {
    let state = helpers::test_state().await;
    let repo = SqlxModelRepo::new(state.db.clone());
    let channel_id = new_uuid_v7();

    insert_channel(&state.db, &channel_id, "OpenAI").await;

    let created = repo
        .insert(&sample_new_model(&channel_id, "gpt-4o"))
        .await
        .unwrap();
    let updated = repo
        .update(
            &channel_id,
            &created.id,
            &ChannelModelPatch {
                display_name: Some(Some("GPT-4o Latest".to_string())),
                context_window: None,
                max_output_tokens: Some(Some(8_192)),
                temperature: None,
                top_p: None,
            },
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(updated.display_name.as_deref(), Some("GPT-4o Latest"));
    assert_eq!(updated.context_window, Some(128_000));
    assert_eq!(updated.max_output_tokens, Some(8_192));
}

/// 删除模型后，会话上的 channel_model_id 应被外键置空。
#[tokio::test]
async fn test_delete_model_sets_conversation_binding_to_null() {
    let state = helpers::test_state().await;
    let repo = SqlxModelRepo::new(state.db.clone());
    let channel_id = new_uuid_v7();
    let conversation_id = new_uuid_v7();

    insert_channel(&state.db, &channel_id, "OpenAI").await;
    let created = repo
        .insert(&sample_new_model(&channel_id, "gpt-4o"))
        .await
        .unwrap();

    sqlx::query(
        r#"
        INSERT INTO conversations (id, title, channel_id, channel_model_id, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        "#,
    )
    .bind(&conversation_id)
    .bind("测试会话")
    .bind(&channel_id)
    .bind(&created.id)
    .bind(100_i64)
    .bind(100_i64)
    .execute(&state.db)
    .await
    .unwrap();

    repo.delete(&channel_id, &created.id).await.unwrap();

    let row = sqlx::query("SELECT channel_model_id FROM conversations WHERE id = ?1")
        .bind(&conversation_id)
        .fetch_one(&state.db)
        .await
        .unwrap();
    let channel_model_id: Option<String> = row.try_get("channel_model_id").unwrap();

    assert_eq!(channel_model_id, None);
}
