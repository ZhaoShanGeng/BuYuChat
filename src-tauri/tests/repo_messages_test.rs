//! 消息仓储层的 SQLite 集成测试。

mod helpers;

use buyu_lib::{
    repo::message_repo,
    utils::ids::new_uuid_v7,
};

/// 向测试数据库插入一条会话记录。
async fn insert_conversation(db: &sqlx::SqlitePool, conversation_id: &str) {
    sqlx::query(
        r#"
        INSERT INTO conversations (id, title, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4)
        "#,
    )
    .bind(conversation_id)
    .bind("测试会话")
    .bind(100_i64)
    .bind(100_i64)
    .execute(db)
    .await
    .unwrap();
}

/// 向测试数据库插入一个包含多版本的 assistant 楼层。
async fn insert_node_with_versions(
    db: &sqlx::SqlitePool,
    conversation_id: &str,
) -> (String, String, String) {
    let node_id = new_uuid_v7();
    let version_a = new_uuid_v7();
    let version_b = new_uuid_v7();

    let mut tx = db.begin().await.unwrap();

    sqlx::query(
        r#"
        INSERT INTO message_nodes (
            id, conversation_id, role, order_key, active_version_id, created_at
        ) VALUES (?1, ?2, 'assistant', '0000000000000100-1-aaaa1111', NULL, 100)
        "#,
    )
    .bind(&node_id)
    .bind(conversation_id)
    .execute(&mut *tx)
    .await
    .unwrap();

    sqlx::query(
        r#"
        INSERT INTO message_versions (id, node_id, status, model_name, created_at)
        VALUES (?1, ?2, 'committed', 'gpt-4o', 101),
               (?3, ?2, 'committed', 'gpt-4o', 102)
        "#,
    )
    .bind(&version_a)
    .bind(&node_id)
    .bind(&version_b)
    .execute(&mut *tx)
    .await
    .unwrap();

    sqlx::query("UPDATE message_nodes SET active_version_id = ?1 WHERE id = ?2")
        .bind(&version_b)
        .bind(&node_id)
        .execute(&mut *tx)
        .await
        .unwrap();

    sqlx::query(
        r#"
        INSERT INTO message_contents (id, version_id, chunk_index, content_type, body, created_at)
        VALUES
            (?1, ?2, 0, 'text/plain', '旧版本内容', 101),
            (?3, ?4, 0, 'text/plain', '新版本', 102),
            (?5, ?4, 1, 'text/plain', '正文', 102)
        "#,
    )
    .bind(new_uuid_v7())
    .bind(&version_a)
    .bind(new_uuid_v7())
    .bind(&version_b)
    .bind(new_uuid_v7())
    .execute(&mut *tx)
    .await
    .unwrap();

    tx.commit().await.unwrap();

    (node_id, version_a, version_b)
}

/// `list_messages` 只应在 active version 中返回 content。
#[tokio::test]
async fn test_list_messages_only_populates_active_version_content() {
    let state = helpers::test_state().await;
    let conversation_id = new_uuid_v7();

    insert_conversation(&state.db, &conversation_id).await;
    let (_, version_a, version_b) = insert_node_with_versions(&state.db, &conversation_id).await;

    let messages = message_repo::list_messages(&state.db, &conversation_id, None, None, false)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(messages.len(), 1);
    let versions = &messages[0].versions;
    let older = versions.iter().find(|item| item.id == version_a).unwrap();
    let active = versions.iter().find(|item| item.id == version_b).unwrap();

    assert_eq!(older.content, None);
    assert_eq!(active.content.as_deref(), Some("新版本正文"));
}

/// `list_messages` 应支持从最新消息向前分页读取。
#[tokio::test]
async fn test_list_messages_supports_latest_page_and_before_cursor() {
    let state = helpers::test_state().await;
    let conversation_id = new_uuid_v7();

    insert_conversation(&state.db, &conversation_id).await;

    let mut tx = state.db.begin().await.unwrap();
    for index in 0..5 {
        let node_id = new_uuid_v7();
        let version_id = new_uuid_v7();
        let order_key = format!("000000000000010{}-0-node{:04}", index, index);
        let body = format!("消息{}", index);

        sqlx::query(
            r#"
            INSERT INTO message_nodes (
                id, conversation_id, role, order_key, active_version_id, created_at
            ) VALUES (?1, ?2, 'user', ?3, ?4, ?5)
            "#,
        )
        .bind(&node_id)
        .bind(&conversation_id)
        .bind(&order_key)
        .bind(&version_id)
        .bind(100_i64 + index as i64)
        .execute(&mut *tx)
        .await
        .unwrap();

        sqlx::query(
            r#"
            INSERT INTO message_versions (id, node_id, status, created_at)
            VALUES (?1, ?2, 'committed', ?3)
            "#,
        )
        .bind(&version_id)
        .bind(&node_id)
        .bind(100_i64 + index as i64)
        .execute(&mut *tx)
        .await
        .unwrap();

        sqlx::query(
            r#"
            INSERT INTO message_contents (id, version_id, chunk_index, content_type, body, created_at)
            VALUES (?1, ?2, 0, 'text/plain', ?3, ?4)
            "#,
        )
        .bind(new_uuid_v7())
        .bind(&version_id)
        .bind(&body)
        .bind(100_i64 + index as i64)
        .execute(&mut *tx)
        .await
        .unwrap();
    }
    tx.commit().await.unwrap();

    let latest_page = message_repo::list_messages(&state.db, &conversation_id, None, Some(2), true)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(latest_page.len(), 2);
    assert_eq!(latest_page[0].versions[0].content.as_deref(), Some("消息3"));
    assert_eq!(latest_page[1].versions[0].content.as_deref(), Some("消息4"));

    let older_page = message_repo::list_messages(
        &state.db,
        &conversation_id,
        latest_page.first().map(|node| node.order_key.as_str()),
        Some(2),
        false,
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(older_page.len(), 2);
    assert_eq!(older_page[0].versions[0].content.as_deref(), Some("消息1"));
    assert_eq!(older_page[1].versions[0].content.as_deref(), Some("消息2"));
}

/// `get_version_content` 应按 chunk_index 拼接完整正文。
#[tokio::test]
async fn test_get_version_content_assembles_all_chunks() {
    let state = helpers::test_state().await;
    let conversation_id = new_uuid_v7();

    insert_conversation(&state.db, &conversation_id).await;
    let (_, _, active_version_id) = insert_node_with_versions(&state.db, &conversation_id).await;

    let content = message_repo::get_version_content(&state.db, &active_version_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(content.content_type, "text/plain");
    assert_eq!(content.content, "新版本正文");
}

/// 删除版本后，其内容块应由外键级联删除。
#[tokio::test]
async fn test_delete_version_cascades_message_contents() {
    let state = helpers::test_state().await;
    let conversation_id = new_uuid_v7();

    insert_conversation(&state.db, &conversation_id).await;
    let (_, version_a, _) = insert_node_with_versions(&state.db, &conversation_id).await;

    let mut tx = state.db.begin().await.unwrap();
    message_repo::delete_version_tx(&mut tx, &version_a)
        .await
        .unwrap();
    tx.commit().await.unwrap();

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM message_contents WHERE version_id = ?1",
    )
    .bind(&version_a)
    .fetch_one(&state.db)
    .await
    .unwrap();

    assert_eq!(count, 0);
}

/// `build_prompt_messages` 应只读取 active version，并支持按 order_key 截断。
#[tokio::test]
async fn test_build_prompt_messages_uses_active_versions_and_cutoff() {
    let state = helpers::test_state().await;
    let conversation_id = new_uuid_v7();
    let user_node_id = new_uuid_v7();
    let user_version_id = new_uuid_v7();
    let assistant_node_id = new_uuid_v7();
    let assistant_version_id = new_uuid_v7();

    insert_conversation(&state.db, &conversation_id).await;

    let mut tx = state.db.begin().await.unwrap();

    sqlx::query(
        r#"
        INSERT INTO message_nodes (id, conversation_id, role, order_key, active_version_id, created_at)
        VALUES
            (?1, ?2, 'user', '0000000000000100-0-aaaa1111', NULL, 100),
            (?3, ?2, 'assistant', '0000000000000101-1-bbbb2222', NULL, 101)
        "#,
    )
    .bind(&user_node_id)
    .bind(&conversation_id)
    .bind(&assistant_node_id)
    .execute(&mut *tx)
    .await
    .unwrap();

    sqlx::query(
        r#"
        INSERT INTO message_versions (id, node_id, status, created_at)
        VALUES (?1, ?2, 'committed', 100), (?3, ?4, 'committed', 101)
        "#,
    )
    .bind(&user_version_id)
    .bind(&user_node_id)
    .bind(&assistant_version_id)
    .bind(&assistant_node_id)
    .execute(&mut *tx)
    .await
    .unwrap();

    sqlx::query(
        r#"
        UPDATE message_nodes
        SET active_version_id = CASE id
            WHEN ?1 THEN ?2
            WHEN ?3 THEN ?4
        END
        WHERE id IN (?1, ?3)
        "#,
    )
    .bind(&user_node_id)
    .bind(&user_version_id)
    .bind(&assistant_node_id)
    .bind(&assistant_version_id)
    .execute(&mut *tx)
    .await
    .unwrap();

    sqlx::query(
        r#"
        INSERT INTO message_contents (id, version_id, chunk_index, content_type, body, created_at)
        VALUES
            (?1, ?2, 0, 'text/plain', '你好', 100),
            (?3, ?4, 0, 'text/plain', '你好，我是助手', 101)
        "#,
    )
    .bind(new_uuid_v7())
    .bind(&user_version_id)
    .bind(new_uuid_v7())
    .bind(&assistant_version_id)
    .execute(&mut *tx)
    .await
    .unwrap();

    tx.commit().await.unwrap();

    let prompt_messages = message_repo::build_prompt_messages(
        &state.db,
        &conversation_id,
        Some("0000000000000101-1-bbbb2222"),
        None,
    )
    .await
    .unwrap();

    assert_eq!(prompt_messages.len(), 1);
    assert_eq!(prompt_messages[0].role, "user");
    assert_eq!(prompt_messages[0].content, "你好");
}
