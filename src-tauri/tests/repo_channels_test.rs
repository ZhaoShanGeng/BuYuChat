//! 渠道仓储层的 SQLite 集成测试。

mod helpers;

use buyu_lib::{
    models::{ChannelPatch, NewChannel},
    repo::channel_repo::{ChannelRepo, SqlxChannelRepo},
    utils::ids::new_uuid_v7,
};
use sqlx::Row;
use uuid::{Uuid, Version};

/// 构造仓储层测试使用的默认渠道数据。
fn sample_new_channel(id: String, name: &str, created_at: i64) -> NewChannel {
    NewChannel {
        id,
        name: name.to_string(),
        channel_type: "openai_compatible".to_string(),
        base_url: "https://api.openai.com".to_string(),
        api_key: Some("sk-test".to_string()),
        api_keys: None,
        auth_type: Some("bearer".to_string()),
        models_endpoint: Some("/v1/models".to_string()),
        chat_endpoint: Some("/v1/chat/completions".to_string()),
        stream_endpoint: Some("/v1/chat/completions".to_string()),
        thinking_tags: None,
        enabled: true,
        created_at,
        updated_at: created_at,
    }
}

/// 插入渠道后应能持久化 UUID v7。
#[tokio::test]
async fn test_insert_channel_persists_uuid_v7() {
    let state = helpers::test_state().await;
    let repo = SqlxChannelRepo::new(state.db.clone());
    let channel = repo
        .insert(&sample_new_channel(new_uuid_v7(), "OpenAI", 100))
        .await
        .unwrap();

    let parsed = Uuid::parse_str(&channel.id).unwrap();
    assert_eq!(parsed.get_version(), Some(Version::SortRand));
    assert_eq!(channel.name, "OpenAI");
}

/// 渠道列表应按 created_at 倒序返回。
#[tokio::test]
async fn test_list_channels_orders_by_created_at_desc() {
    let state = helpers::test_state().await;
    let repo = SqlxChannelRepo::new(state.db.clone());

    repo.insert(&sample_new_channel(new_uuid_v7(), "旧渠道", 100))
        .await
        .unwrap();
    repo.insert(&sample_new_channel(new_uuid_v7(), "新渠道", 200))
        .await
        .unwrap();

    let channels = repo.list(true).await.unwrap();
    assert_eq!(channels[0].name, "新渠道");
    assert_eq!(channels[1].name, "旧渠道");
}

/// 渠道更新应只影响提交字段。
#[tokio::test]
async fn test_update_channel_only_changes_supplied_fields() {
    let state = helpers::test_state().await;
    let repo = SqlxChannelRepo::new(state.db.clone());
    let channel = repo
        .insert(&sample_new_channel(new_uuid_v7(), "OpenAI", 100))
        .await
        .unwrap();

    let updated = repo
        .update(
            &channel.id,
            &ChannelPatch {
                name: Some("OpenAI Pro".to_string()),
                base_url: None,
                channel_type: None,
                api_key: None,
                api_keys: None,
                auth_type: None,
                models_endpoint: None,
                chat_endpoint: None,
                stream_endpoint: None,
                thinking_tags: None,
                enabled: Some(false),
                updated_at: 999,
            },
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(updated.name, "OpenAI Pro");
    assert_eq!(updated.base_url, "https://api.openai.com");
    assert!(!updated.enabled);
    assert_eq!(updated.updated_at, 999);
}

/// 删除渠道后应级联删除关联模型。
#[tokio::test]
async fn test_delete_channel_cascades_models() {
    let state = helpers::test_state().await;
    let repo = SqlxChannelRepo::new(state.db.clone());
    let channel = repo
        .insert(&sample_new_channel(new_uuid_v7(), "OpenAI", 100))
        .await
        .unwrap();

    sqlx::query(
        r#"
        INSERT INTO api_channel_models (id, channel_id, model_id, display_name)
        VALUES (?1, ?2, ?3, ?4)
        "#,
    )
    .bind(new_uuid_v7())
    .bind(&channel.id)
    .bind("gpt-4o-mini")
    .bind("GPT-4o Mini")
    .execute(&state.db)
    .await
    .unwrap();

    repo.delete(&channel.id).await.unwrap();

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM api_channel_models")
        .fetch_one(&state.db)
        .await
        .unwrap();
    assert_eq!(count, 0);
}

/// 删除渠道后会话绑定应被置空。
#[tokio::test]
async fn test_delete_channel_sets_conversation_bindings_to_null() {
    let state = helpers::test_state().await;
    let repo = SqlxChannelRepo::new(state.db.clone());
    let channel = repo
        .insert(&sample_new_channel(new_uuid_v7(), "OpenAI", 100))
        .await
        .unwrap();
    let model_id = new_uuid_v7();
    let conversation_id = new_uuid_v7();

    sqlx::query(
        r#"
        INSERT INTO api_channel_models (id, channel_id, model_id, display_name)
        VALUES (?1, ?2, ?3, ?4)
        "#,
    )
    .bind(&model_id)
    .bind(&channel.id)
    .bind("gpt-4o")
    .bind("GPT-4o")
    .execute(&state.db)
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
    .bind(&channel.id)
    .bind(&model_id)
    .bind(100_i64)
    .bind(100_i64)
    .execute(&state.db)
    .await
    .unwrap();

    repo.delete(&channel.id).await.unwrap();

    let row = sqlx::query("SELECT channel_id, channel_model_id FROM conversations WHERE id = ?1")
        .bind(&conversation_id)
        .fetch_one(&state.db)
        .await
        .unwrap();

    let channel_id: Option<String> = row.try_get("channel_id").unwrap();
    let channel_model_id: Option<String> = row.try_get("channel_model_id").unwrap();
    assert_eq!(channel_id, None);
    assert_eq!(channel_model_id, None);
}
