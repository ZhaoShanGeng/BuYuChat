//! 消息与生成控制命令的集成测试。

mod helpers;

use std::time::Duration;

use buyu_lib::{
    commands::{
        agents::create_agent_impl,
        channels::create_channel_impl,
        conversations::create_conversation_impl,
        messages::{
            cancel_generation_impl, delete_version_impl, edit_message_impl,
            get_version_content_impl, list_messages_impl, reroll_impl, send_message_impl,
            set_active_version_impl,
        },
        models::create_model_impl,
    },
    models::{
        CreateAgentInput, CreateChannelInput, CreateConversationInput, CreateModelInput,
        EditMessageInput, RerollInput, SendMessageInput, SendMessageResponse,
    },
    utils::ids::new_uuid_v7,
};
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

/// 创建一条已绑定 Agent / 渠道 / 模型的测试会话。
async fn create_bound_conversation(state: &buyu_lib::state::AppState, base_url: &str) -> String {
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
            api_keys: None,
            auth_type: None,
            models_endpoint: None,
            chat_endpoint: None,
            stream_endpoint: None,
            thinking_tags: None,
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
            temperature: None,
            top_p: None,
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
            enabled_tools: None,
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
async fn wait_for_terminal_status(db: &sqlx::SqlitePool, version_id: &str) -> String {
    for _ in 0..50 {
        let status: Option<String> =
            sqlx::query_scalar("SELECT status FROM message_versions WHERE id = ?1")
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
            images: None,
            files: None,
            tool_results: None,
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
            images: None,
            files: None,
            tool_results: None,
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

    let messages = list_messages_impl(&state, conversation_id, None, None, None)
        .await
        .unwrap();
    assert_eq!(messages.len(), 2);
    assert_eq!(
        messages[1].versions[0].content.as_deref(),
        Some("当然可以，这是结果。")
    );
}

/// 非流式发送在 assistant 返回 content parts 时，应同时持久化正文、图片与文件附件。
#[tokio::test]
async fn test_send_message_non_stream_persists_assistant_attachments() {
    let state = helpers::test_state().await;
    let server = MockServer::start().await;
    let conversation_id = create_bound_conversation(&state, &server.uri()).await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "chatcmpl-attachments-1",
            "object": "chat.completion",
            "created": 1735000002,
            "model": "gpt-4o",
            "choices": [
                {
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": [
                            { "type": "text", "text": "这里有结果和附件。" },
                            {
                                "type": "image_url",
                                "image_url": { "url": "data:image/png;base64,aW1hZ2UtZGF0YQ==" }
                            },
                            {
                                "type": "file",
                                "file": {
                                    "filename": "report.txt",
                                    "mime_type": "text/plain",
                                    "data": "data:text/plain;base64,SGVsbG8sIGZpbGUh"
                                }
                            }
                        ]
                    },
                    "finish_reason": "stop"
                }
            ],
            "usage": {
                "prompt_tokens": 16,
                "completion_tokens": 10,
                "total_tokens": 26
            }
        })))
        .mount(&server)
        .await;

    let response = send_message_impl(
        &state,
        conversation_id.clone(),
        SendMessageInput {
            content: "给我一个带附件的结果".to_string(),
            images: None,
            files: None,
            tool_results: None,
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

    let messages = list_messages_impl(&state, conversation_id, None, None, None)
        .await
        .unwrap();
    let assistant_version = &messages[1].versions[0];

    assert_eq!(
        assistant_version.content.as_deref(),
        Some("这里有结果和附件。")
    );
    assert_eq!(assistant_version.images.len(), 1);
    assert_eq!(assistant_version.images[0].mime_type, "image/png");
    assert_eq!(assistant_version.images[0].base64, "aW1hZ2UtZGF0YQ==");
    assert_eq!(assistant_version.files.len(), 1);
    assert_eq!(assistant_version.files[0].name, "report.txt");
    assert_eq!(assistant_version.files[0].mime_type, "text/plain");
    assert_eq!(assistant_version.files[0].base64, "SGVsbG8sIGZpbGUh");
}

/// 流式发送在只返回 delta、最终 Done 不带完整文本时，也应保留 assistant 节点与正文。
#[tokio::test]
async fn test_send_message_stream_persists_delta_content_without_empty_rollback() {
    let state = helpers::test_state().await;
    let server = MockServer::start().await;
    let conversation_id = create_bound_conversation(&state, &server.uri()).await;

    let sse_body = concat!(
        "data: {\"id\":\"chatcmpl-1\",\"object\":\"chat.completion.chunk\",\"created\":1735000000,\"model\":\"gpt-4o\",\"choices\":[{\"index\":0,\"delta\":{\"role\":\"assistant\",\"content\":\"当然\"},\"finish_reason\":null}]}\n\n",
        "data: {\"id\":\"chatcmpl-1\",\"object\":\"chat.completion.chunk\",\"created\":1735000000,\"model\":\"gpt-4o\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"可以，这是结果。\"},\"finish_reason\":null}]}\n\n",
        "data: {\"id\":\"chatcmpl-1\",\"object\":\"chat.completion.chunk\",\"created\":1735000000,\"model\":\"gpt-4o\",\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"stop\"}],\"usage\":{\"prompt_tokens\":12,\"completion_tokens\":8,\"total_tokens\":20}}\n\n",
        "data: [DONE]\n\n"
    );

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_raw(sse_body, "text/event-stream"),
        )
        .mount(&server)
        .await;

    let response = send_message_impl(
        &state,
        conversation_id.clone(),
        SendMessageInput {
            content: "请给我一个流式结果".to_string(),
            images: None,
            files: None,
            tool_results: None,
            stream: Some(true),
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

    let messages = list_messages_impl(&state, conversation_id, None, None, None)
        .await
        .unwrap();
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[1].role, "assistant");
    assert_eq!(
        messages[1].versions[0].content.as_deref(),
        Some("当然可以，这是结果。")
    );
}

/// 流式发送在只返回图片/文件附件、不返回正文时，也不应触发空消息回滚。
#[tokio::test]
async fn test_send_message_stream_persists_attachment_only_response() {
    let state = helpers::test_state().await;
    let server = MockServer::start().await;
    let conversation_id = create_bound_conversation(&state, &server.uri()).await;

    let sse_body = concat!(
        "data: {\"id\":\"chatcmpl-attachments-2\",\"object\":\"chat.completion.chunk\",\"created\":1735000003,\"model\":\"gpt-4o\",\"choices\":[{\"index\":0,\"delta\":{\"role\":\"assistant\",\"content\":[{\"type\":\"image_url\",\"image_url\":{\"url\":\"data:image/png;base64,c3RyZWFtLWltYWdl\"}},{\"type\":\"file\",\"file\":{\"filename\":\"stream.txt\",\"mime_type\":\"text/plain\",\"data\":\"data:text/plain;base64,c3RyZWFtLWZpbGU=\"}}]},\"finish_reason\":null}]}\n\n",
        "data: {\"id\":\"chatcmpl-attachments-2\",\"object\":\"chat.completion.chunk\",\"created\":1735000003,\"model\":\"gpt-4o\",\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"stop\"}],\"usage\":{\"prompt_tokens\":14,\"completion_tokens\":6,\"total_tokens\":20}}\n\n",
        "data: [DONE]\n\n"
    );

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_raw(sse_body, "text/event-stream"),
        )
        .mount(&server)
        .await;

    let response = send_message_impl(
        &state,
        conversation_id.clone(),
        SendMessageInput {
            content: "给我一个纯附件流式结果".to_string(),
            images: None,
            files: None,
            tool_results: None,
            stream: Some(true),
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

    let messages = list_messages_impl(&state, conversation_id, None, None, None)
        .await
        .unwrap();
    let assistant_version = &messages[1].versions[0];

    assert_eq!(assistant_version.content, None);
    assert_eq!(assistant_version.images.len(), 1);
    assert_eq!(assistant_version.images[0].mime_type, "image/png");
    assert_eq!(assistant_version.images[0].base64, "c3RyZWFtLWltYWdl");
    assert_eq!(assistant_version.files.len(), 1);
    assert_eq!(assistant_version.files[0].name, "stream.txt");
    assert_eq!(assistant_version.files[0].mime_type, "text/plain");
    assert_eq!(assistant_version.files[0].base64, "c3RyZWFtLWZpbGU=");
}

/// 当 user node 后已有 assistant 楼层时，reroll 应复用该 assistant node 创建新版本。
#[tokio::test]
async fn test_reroll_user_node_reuses_following_assistant_node() {
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
            images: None,
            files: None,
            tool_results: None,
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

    let reroll_result = reroll_impl(
        &state,
        conversation_id.clone(),
        started.user_node_id.clone(),
        Some(RerollInput {
            stream: Some(false),
        }),
        None,
    )
    .await
    .unwrap();

    assert_eq!(reroll_result.new_user_version_id, None);
    assert_eq!(reroll_result.assistant_node_id, started.assistant_node_id);

    let status = wait_for_terminal_status(&state.db, &reroll_result.assistant_version_id).await;
    assert_eq!(status, "committed");

    let messages = list_messages_impl(&state, conversation_id, None, None, None)
        .await
        .unwrap();
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[1].versions.len(), 2);
}

/// edit_message(save) 应在原 node 下创建 committed 新版本。
#[tokio::test]
async fn test_edit_message_creates_new_version_on_same_node() {
    let state = helpers::test_state().await;
    let conversation_id = create_bound_conversation(&state, "https://api.openai.com").await;
    seed_history(&state.db, &conversation_id).await;

    let messages = list_messages_impl(&state, conversation_id.clone(), None, None, None)
        .await
        .unwrap();
    let user_node = messages
        .iter()
        .find(|node| node.role == "user")
        .expect("user node should exist");

    let result = edit_message_impl(
        &state,
        conversation_id.clone(),
        user_node.id.clone(),
        EditMessageInput {
            content: "编辑后的用户消息".to_string(),
            resend: Some(false),
            stream: Some(false),
        },
        None,
    )
    .await
    .unwrap();

    let messages = list_messages_impl(&state, conversation_id, None, None, None)
        .await
        .unwrap();
    let updated_user_node = messages
        .iter()
        .find(|node| node.id == user_node.id)
        .expect("edited user node should exist");

    assert_eq!(
        updated_user_node.active_version_id.as_deref(),
        Some(result.edited_version_id.as_str())
    );
    assert_eq!(updated_user_node.versions.len(), 2);
    assert_eq!(
        updated_user_node
            .versions
            .iter()
            .find(|version| version.id == result.edited_version_id)
            .and_then(|version| version.content.as_deref()),
        Some("编辑后的用户消息")
    );
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
    assert_eq!(
        result.new_active_version_id.as_deref(),
        Some(version_b.as_str())
    );
}

/// 上游返回非 2xx 时，应把错误码、错误消息与请求/响应详情落到失败版本。
#[tokio::test]
async fn test_send_message_persists_structured_error_details() {
    let state = helpers::test_state().await;
    let server = MockServer::start().await;
    let conversation_id = create_bound_conversation(&state, &server.uri()).await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "error": {
                "message": "invalid api key",
                "code": "invalid_api_key"
            }
        })))
        .mount(&server)
        .await;

    let response = send_message_impl(
        &state,
        conversation_id.clone(),
        SendMessageInput {
            content: "请触发失败".to_string(),
            images: None,
            files: None,
            tool_results: None,
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

    let status = wait_for_terminal_status(&state.db, &started.assistant_version_id).await;
    assert_eq!(status, "failed");

    let messages = list_messages_impl(&state, conversation_id, None, None, None)
        .await
        .unwrap();
    let assistant_version = &messages[1].versions[0];

    assert_eq!(assistant_version.status, "failed");
    assert_eq!(
        assistant_version.error_code.as_deref(),
        Some("AI_REQUEST_FAILED")
    );
    assert!(assistant_version
        .error_message
        .as_deref()
        .unwrap_or_default()
        .contains("401"));

    let error_details = assistant_version
        .error_details
        .as_ref()
        .expect("error details should be present");
    let expected_request_url = format!("{}/v1/chat/completions", server.uri());
    assert_eq!(
        error_details.request_url.as_deref(),
        Some(expected_request_url.as_str())
    );
    assert_eq!(error_details.request_method.as_deref(), Some("POST"));
    assert_eq!(error_details.response_status, Some(401));
    assert!(error_details
        .request_body
        .as_deref()
        .unwrap_or_default()
        .contains("\"content\": \"请触发失败\""));
    assert!(error_details
        .response_body
        .as_deref()
        .unwrap_or_default()
        .contains("invalid api key"));
}

/// 取消进行中的生成后，版本应最终变成 cancelled，且不会再被后续完成覆盖。
#[tokio::test]
async fn test_cancel_generation_keeps_version_cancelled() {
    let state = helpers::test_state().await;
    let server = MockServer::start().await;
    let conversation_id = create_bound_conversation(&state, &server.uri()).await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(Duration::from_millis(400))
                .set_body_json(serde_json::json!({
                    "id": "chatcmpl-cancel-1",
                    "object": "chat.completion",
                    "created": 1735000004,
                    "model": "gpt-4o",
                    "choices": [
                        {
                            "index": 0,
                            "message": { "role": "assistant", "content": "这条结果不该写回。" },
                            "finish_reason": "stop"
                        }
                    ],
                    "usage": {
                        "prompt_tokens": 12,
                        "completion_tokens": 8,
                        "total_tokens": 20
                    }
                })),
        )
        .mount(&server)
        .await;

    let response = send_message_impl(
        &state,
        conversation_id.clone(),
        SendMessageInput {
            content: "请开始一个可取消的请求".to_string(),
            images: None,
            files: None,
            tool_results: None,
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

    cancel_generation_impl(&state, started.assistant_version_id.clone())
        .await
        .unwrap();

    let status = wait_for_terminal_status(&state.db, &started.assistant_version_id).await;
    assert_eq!(status, "cancelled");

    tokio::time::sleep(Duration::from_millis(700)).await;

    let final_status: String =
        sqlx::query_scalar("SELECT status FROM message_versions WHERE id = ?1")
            .bind(&started.assistant_version_id)
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(final_status, "cancelled");

    let messages = list_messages_impl(&state, conversation_id, None, None, None)
        .await
        .unwrap();
    assert_eq!(messages[1].versions[0].status, "cancelled");
    assert_ne!(
        messages[1].versions[0].content.as_deref(),
        Some("这条结果不该写回。")
    );
}

/// 取消不存在的版本应保持幂等成功。
#[tokio::test]
async fn test_cancel_generation_is_idempotent_for_missing_version() {
    let state = helpers::test_state().await;

    cancel_generation_impl(&state, "missing-version".to_string())
        .await
        .unwrap();
}
