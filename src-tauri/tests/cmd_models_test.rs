//! 模型命令层的集成测试。

mod helpers;

use buyu_lib::{
    commands::{
        channels::create_channel_impl,
        models::{
            create_model_impl, delete_model_impl, fetch_remote_models_impl, list_models_impl,
            update_model_impl,
        },
    },
    error::AppError,
    models::{CreateChannelInput, CreateModelInput, UpdateModelInput},
};
use wiremock::{
    matchers::{header, method, path},
    Mock, MockServer, ResponseTemplate,
};

/// 创建一条可供模型命令复用的测试渠道。
async fn create_test_channel(state: &buyu_lib::state::AppState, base_url: &str) -> String {
    create_channel_impl(
        state,
        CreateChannelInput {
            name: "My OpenAI".to_string(),
            base_url: base_url.to_string(),
            channel_type: None,
            api_key: Some("sk-xxx".to_string()),
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
    .unwrap()
    .id
}

/// 模型命令应支持完整的 CRUD 流程。
#[tokio::test]
async fn test_model_commands_cover_crud_flow() {
    let state = helpers::test_state().await;
    let channel_id = create_test_channel(&state, "https://api.openai.com").await;

    let created = create_model_impl(
        &state,
        channel_id.clone(),
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

    let listed = list_models_impl(&state, channel_id.clone()).await.unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, created.id);

    let updated = update_model_impl(
        &state,
        channel_id.clone(),
        created.id.clone(),
        UpdateModelInput {
            display_name: Some(Some("GPT-4o Latest".to_string())),
            context_window: None,
            max_output_tokens: Some(Some(8_192)),
            temperature: None,
            top_p: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(updated.display_name.as_deref(), Some("GPT-4o Latest"));

    delete_model_impl(&state, channel_id.clone(), created.id.clone())
        .await
        .unwrap();

    let listed_after_delete = list_models_impl(&state, channel_id).await.unwrap();
    assert!(listed_after_delete.is_empty());
}

/// 重复创建模型时应返回结构化冲突错误。
#[tokio::test]
async fn test_create_model_returns_conflict_for_duplicate_model_id() {
    let state = helpers::test_state().await;
    let channel_id = create_test_channel(&state, "https://api.openai.com").await;

    create_model_impl(
        &state,
        channel_id.clone(),
        CreateModelInput {
            model_id: "gpt-4o".to_string(),
            display_name: None,
            context_window: None,
            max_output_tokens: None,
            temperature: None,
            top_p: None,
        },
    )
    .await
    .unwrap();

    let error = create_model_impl(
        &state,
        channel_id,
        CreateModelInput {
            model_id: "gpt-4o".to_string(),
            display_name: None,
            context_window: None,
            max_output_tokens: None,
            temperature: None,
            top_p: None,
        },
    )
    .await
    .unwrap_err();

    assert_eq!(
        error,
        AppError::conflict(
            "MODEL_ID_CONFLICT",
            "model_id 'gpt-4o' already exists in this channel"
        )
    );
}

/// 缺失渠道时，模型列表命令应返回 NOT_FOUND。
#[tokio::test]
async fn test_list_models_returns_not_found_for_missing_channel() {
    let state = helpers::test_state().await;

    let error = list_models_impl(&state, "missing".to_string())
        .await
        .unwrap_err();

    assert_eq!(error, AppError::not_found("channel 'missing' not found"));
}

/// 远程拉取模型命令应返回解析后的 OpenAI-compatible 模型列表。
#[tokio::test]
async fn test_fetch_remote_models_returns_remote_model_list() {
    let state = helpers::test_state().await;
    let server = MockServer::start().await;
    let channel_id = create_test_channel(&state, &server.uri()).await;

    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .and(header("authorization", "Bearer sk-xxx"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                { "id": "gpt-4o", "display_name": "GPT-4o", "context_window": 128000 },
                { "id": "gpt-4o-mini" }
            ]
        })))
        .mount(&server)
        .await;

    let models = fetch_remote_models_impl(&state, channel_id).await.unwrap();

    assert_eq!(models.len(), 2);
    assert_eq!(models[0].model_id, "gpt-4o");
    assert_eq!(models[0].display_name.as_deref(), Some("GPT-4o"));
    assert_eq!(models[0].context_window, Some(128_000));
    assert_eq!(models[1].model_id, "gpt-4o-mini");
}

/// 缺失渠道时，远程拉取模型命令应返回 NOT_FOUND。
#[tokio::test]
async fn test_fetch_remote_models_returns_not_found_for_missing_channel() {
    let state = helpers::test_state().await;

    let error = fetch_remote_models_impl(&state, "missing".to_string())
        .await
        .unwrap_err();

    assert_eq!(error, AppError::not_found("channel 'missing' not found"));
}
