//! 消息与生成控制命令的集成测试。

mod helpers;

use std::time::Duration;

use buyu_lib::{
    commands::{
        agents::create_agent_impl,
        channels::create_channel_impl,
        conversations::create_conversation_impl,
        messages::{
            cancel_generation_impl, delete_version_impl, get_version_content_impl,
            list_messages_impl, reroll_impl, send_message_impl, set_active_version_impl,
        },
        models::create_model_impl,
    },
    error::AppError,
    models::{
        CreateAgentInput, CreateChannelInput, CreateConversationInput, CreateModelInput,
        RerollInput, SendMessageInput, SendMessageResponse,
    },
    utils::ids::new_uuid_v7,
};
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

/// 创建一条已绑定 Agent / 渠道 / 模型的测试会话。
async fn create_bound_conversation(
    state: &buyu_lib::state::AppState,
    base_url: &str,
) -> String {
    let agent = create_agent_impl(
        state,
        CreateAgentInput {
            name: "助手".to_string(),
            system_prompt: Some("你是一个严谨的 Rust 助手".to_string()),
        },
    )
    .await
    .unwrap();
    let channel = create_channel_impl(
        state,
        CreateChannelInput {
            name: "My OpenAI".to_string(),
            base_url: base_url.to_string(),
            channel_type: None,
            api_key: Some("sk-test".to_string()),
            auth_type: None,
            models_endpoint: None,
            chat_endpoint: None,
            stream_endpoint: None,
            enabled: None,
        },
    )
    .await
    .unwrap();
    let model = create_model_impl(
        state,
        channel.id.clone(),
        CreateModelInput {
            model_id: "gpt-4o".to_string(),
            display_name: Some("GPT-4o".to_string()),
            context_window: Some(128_000),
            max_output_tokens: Some(16_384),
        },
    )
    .await
    .unwrap();
    let conversation = create_conversation_impl(
        state,
        Some(CreateConversationInput {
            title: Some("测试会话".to_string()),
            agent_id: Some(agent.id),
            channel_id: Some(channel.id),
            channel_model_id: Some(model.id),
        }),
    )
    .await
    .unwrap();

    conversation.id
}

/// 向测试会话插入一组历史 user / assistant 消息。
async fn seed_history(db: &sqlx::SqlitePool, conversation_id: &str) {
    let user_node_id = new_uuid_v7();
    let user_version_id = new_uuid_v7();
    let assistant_node_id = new_uuid_v7();
    let assistant_version_id = new_uuid_v7();

    let mut tx = db.begin().await.unwrap();

    sqlx::query(
        r#"
        INSERT INTO message_nodes (id, conversation_id, role, order_key, active_version_id, created_at)
        VALUES
            (?1, ?2, 'user', '0000000000000100-0-aaaa1111', NULL, 100),
            (?3, ?2, 'assistant', '0000000000000100-1-bbbb2222', NULL, 101)
        "#,
    )
    .bind(&user_node_id)
    .bind(conversation_id)
    .bind(&assistant_node_id)
    .execute(&mut *tx)
    .await
    .unwrap();

    sqlx::query(
        r#"
        INSERT INTO message_versions (id, node_id, status, model_name, created_at)
        VALUES
            (?1, ?2, 'committed', NULL, 100),
            (?3, ?4, 'committed', 'gpt-4o', 101)
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
            (?3, ?4, 0, 'text/plain', '你好，我在。', 101)
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
}

/// 等待后台生成任务把版本状态从 generating 推进到终态。
async fn wait_for_terminal_status(
    db: &sqlx::SqlitePool,
    version_id: &str,
) -> String {
    for _ in 0..50 {
        let status: Option<String> = sqlx::query_scalar(
            "SELECT status FROM message_versions WHERE id = ?1",
        )
        .bind(version_id)
        .fetch_optional(db)
        .await
        .unwrap();

        if let Some(status) = status {
            if status != "generating" {
                return status;
            }
        }

        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    panic!("version did not reach terminal status in time");
}

/// `send_message` 的 dry_run 应返回拼装好的 prompt 与目标模型。
#[tokio::test]
async fn test_send_message_dry_run_returns_prompt_debug_payload() {
    let state = helpers::test_state().await;
    let conversation_id = create_bound_conversation(&state, "https://api.openai.com").await;
    seed_history(&state.db, &conversation_id).await;

    let response = send_message_impl(
        &state,
        conversation_id,
        SendMessageInput {
            content: "测试问题".to_string(),
            stream: Some(false),
            dry_run: Some(true),
        },
        None,
    )
    .await
    .unwrap();

    let SendMessageResponse::DryRun(result) = response else {
        panic!("expected dry_run response");
    };

    assert_eq!(result.model, "gpt-4o");
    assert_eq!(result.messages.len(), 4);
    assert_eq!(result.messages[0].role, "system");
    assert_eq!(result.messages[3].content, "测试问题");
}

/// 非流式发送消息后，后台任务应写入 committed 终态与完整 assistant 内容。
#[tokio::test]
async fn test_send_message_non_stream_persists_final_assistant_content() {
    let state = helpers::test_state().await;
    let server = MockServer::start().await;
    let conversation_id = create_bound_conversation(&state, &server.uri()).await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "chatcmpl-1",
            "object": "chat.completion",
            "created": 1735000000,
            "model": "gpt-4o",
            "choices": [
                {
                    "index": 0,
                    "message": { "role": "assistant", "content": "当然可以，这是结果。" },
                    "finish_reason": "stop"
                }
            ],
            "usage": {
                "prompt_tokens": 12,
                "completion_tokens": 8,
                "total_tokens": 20
            }
        })))
        .mount(&server)
        .await;

    let response = send_message_impl(
        &state,
        conversation_id.clone(),
        SendMessageInput {
            content: "请给我一个结果".to_string(),
            stream: Some(false),
            dry_run: Some(false),
        },
        None,
    )
    .await
    .unwrap();

    let SendMessageResponse::Started(result) = response else {
        panic!("expected started response");
    };

    let status = wait_for_terminal_status(&state.db, &result.assistant_version_id).await;
    assert_eq!(status, "committed");

    let messages = list_messages_impl(&state, conversation_id, None, None)
        .await
        .unwrap();
    assert_eq!(messages.len(), 2);
    assert_eq!(
        messages[1].versions[0].content.as_deref(),
        Some("当然可以，这是结果。")
    );
}

/// 当 user node 不是最后一个楼层时，reroll 应返回 NOT_LAST_USER_NODE。
#[tokio::test]
async fn test_reroll_user_node_returns_not_last_user_node() {
    let state = helpers::test_state().await;
    let server = MockServer::start().await;
    let conversation_id = create_bound_conversation(&state, &server.uri()).await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "chatcmpl-1",
            "object": "chat.completion",
            "created": 1735000001,
            "model": "gpt-4o",
            "choices": [
                {
                    "index": 0,
                    "message": { "role": "assistant", "content": "好的" },
                    "finish_reason": "stop"
                }
            ],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 2,
                "total_tokens": 12
            }
        })))
        .mount(&server)
        .await;

    let response = send_message_impl(
        &state,
        conversation_id.clone(),
        SendMessageInput {
            content: "第一条".to_string(),
            stream: Some(false),
            dry_run: Some(false),
        },
        None,
    )
    .await
    .unwrap();

    let SendMessageResponse::Started(started) = response else {
        panic!("expected started response");
    };
    let _ = wait_for_terminal_status(&state.db, &started.assistant_version_id).await;

    let error = reroll_impl(
        &state,
        conversation_id,
        started.user_node_id,
        Some(RerollInput { stream: Some(false) }),
        None,
    )
    .await
    .unwrap_err();

    assert_eq!(error, AppError::not_last_user_node());
}

/// 版本切换与删除命令应能围绕同一楼层完成基本管理。
#[tokio::test]
async fn test_set_active_version_and_delete_version_commands() {
    let state = helpers::test_state().await;
    let conversation_id = create_bound_conversation(&state, "https://api.openai.com").await;
    let node_id = new_uuid_v7();
    let version_a = new_uuid_v7();
    let version_b = new_uuid_v7();

    let mut tx = state.db.begin().await.unwrap();

    sqlx::query(
        r#"
        INSERT INTO message_nodes (id, conversation_id, role, order_key, active_version_id, created_at)
        VALUES (?1, ?2, 'assistant', '0000000000000200-1-cccc3333', NULL, 200)
        "#,
    )
    .bind(&node_id)
    .bind(&conversation_id)
    .execute(&mut *tx)
    .await
    .unwrap();

    sqlx::query(
        r#"
        INSERT INTO message_versions (id, node_id, status, model_name, created_at)
        VALUES
            (?1, ?2, 'committed', 'gpt-4o', 201),
            (?3, ?2, 'committed', 'gpt-4o', 202)
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
            (?1, ?2, 0, 'text/plain', '版本A', 201),
            (?3, ?4, 0, 'text/plain', '版本B', 202)
        "#,
    )
    .bind(new_uuid_v7())
    .bind(&version_a)
    .bind(new_uuid_v7())
    .bind(&version_b)
    .execute(&mut *tx)
    .await
    .unwrap();

    tx.commit().await.unwrap();

    set_active_version_impl(
        &state,
        conversation_id.clone(),
        node_id.clone(),
        version_a.clone(),
    )
    .await
    .unwrap();

    let content = get_version_content_impl(&state, version_a.clone())
        .await
        .unwrap();
    assert_eq!(content.content, "版本A");

    let result = delete_version_impl(&state, conversation_id, node_id, version_a.clone())
        .await
        .unwrap();
    assert!(!result.node_deleted);
    assert_eq!(result.new_active_version_id.as_deref(), Some(version_b.as_str()));
}

/// 取消不存在的版本应保持幂等成功。
#[tokio::test]
async fn test_cancel_generation_is_idempotent_for_missing_version() {
    let state = helpers::test_state().await;

    cancel_generation_impl(&state, "missing-version".to_string())
        .await
        .unwrap();
}
