//! 会话命令层的集成测试。

mod helpers;

use buyu_lib::{
    commands::{
        agents::create_agent_impl,
        channels::create_channel_impl,
        conversations::{
            create_conversation_impl, delete_conversation_impl, get_conversation_impl,
            list_conversations_impl, update_conversation_impl,
        },
        models::create_model_impl,
    },
    error::AppError,
    models::{
        CreateAgentInput, CreateChannelInput, CreateConversationInput, CreateModelInput,
        UpdateConversationInput,
    },
};

/// 创建一组会话绑定所需的依赖资源。
async fn create_dependencies(state: &buyu_lib::state::AppState) -> (String, String, String) {
    let agent = create_agent_impl(
        state,
        CreateAgentInput {
            name: "助手".to_string(),
            system_prompt: Some("你是一个有帮助的助手".to_string()),
        },
    )
    .await
    .unwrap();
    let channel = create_channel_impl(
        state,
        CreateChannelInput {
            name: "My OpenAI".to_string(),
            base_url: "https://api.openai.com".to_string(),
            channel_type: None,
            api_key: Some("sk-xxx".to_string()),
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

    (agent.id, channel.id, model.id)
}

/// 会话命令应支持创建、绑定、查询与删除。
#[tokio::test]
async fn test_conversation_commands_cover_crud_flow() {
    let state = helpers::test_state().await;
    let (agent_id, channel_id, model_id) = create_dependencies(&state).await;

    let created = create_conversation_impl(
        &state,
        Some(CreateConversationInput {
            title: Some("Rust 讨论".to_string()),
            agent_id: Some(agent_id.clone()),
            channel_id: Some(channel_id.clone()),
            channel_model_id: Some(model_id.clone()),
        }),
    )
    .await
    .unwrap();

    let listed = list_conversations_impl(&state, None).await.unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, created.id);

    let fetched = get_conversation_impl(&state, created.id.clone()).await.unwrap();
    assert_eq!(fetched.title, "Rust 讨论");

    let updated = update_conversation_impl(
        &state,
        created.id.clone(),
        UpdateConversationInput {
            title: Some("重命名后会话".to_string()),
            pinned: Some(true),
            ..UpdateConversationInput::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(updated.title, "重命名后会话");
    assert!(updated.pinned);

    delete_conversation_impl(&state, created.id.clone())
        .await
        .unwrap();

    let error = get_conversation_impl(&state, created.id.clone())
        .await
        .unwrap_err();
    assert_eq!(
        error,
        AppError::not_found(format!("conversation '{}' not found", created.id))
    );
}

/// 绑定缺失渠道的模型时，应返回结构化校验错误。
#[tokio::test]
async fn test_update_conversation_requires_channel_when_model_is_set() {
    let state = helpers::test_state().await;
    let (_, _, model_id) = create_dependencies(&state).await;
    let created = create_conversation_impl(&state, None).await.unwrap();

    let error = update_conversation_impl(
        &state,
        created.id,
        UpdateConversationInput {
            channel_model_id: Some(Some(model_id)),
            ..UpdateConversationInput::default()
        },
    )
    .await
    .unwrap_err();

    assert_eq!(
        error,
        AppError::validation("VALIDATION_ERROR", "channel_model_id requires channel_id")
    );
}
