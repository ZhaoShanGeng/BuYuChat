//! 渠道命令层的集成测试。

mod helpers;

use buyu_lib::{
    commands::channels::{
        create_channel_impl, delete_channel_impl, get_channel_impl, list_channels_impl,
        test_channel_impl, update_channel_impl,
    },
    error::AppError,
    models::{CreateChannelInput, UpdateChannelInput},
};

/// 创建命令应支持完整的 CRUD 流程。
#[tokio::test]
async fn test_channel_commands_cover_crud_flow() {
    let state = helpers::test_state().await;

    let created = create_channel_impl(
        &state,
        CreateChannelInput {
            name: "My OpenAI".to_string(),
            base_url: "https://api.openai.com".to_string(),
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
    .unwrap();

    let listed = list_channels_impl(&state, None).await.unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, created.id);

    let fetched = get_channel_impl(&state, created.id.clone()).await.unwrap();
    assert_eq!(fetched.name, "My OpenAI");

    let updated = update_channel_impl(
        &state,
        created.id.clone(),
        UpdateChannelInput {
            name: Some("OpenAI Pro".to_string()),
            ..UpdateChannelInput::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(updated.name, "OpenAI Pro");

    delete_channel_impl(&state, created.id.clone())
        .await
        .unwrap();

    let missing = get_channel_impl(&state, created.id.clone())
        .await
        .unwrap_err();
    assert_eq!(
        missing,
        AppError::not_found(format!("channel '{}' not found", created.id))
    );
}

/// list_channels 未传 include_disabled 时应默认包含已禁用渠道。
#[tokio::test]
async fn test_list_channels_defaults_to_include_disabled_true() {
    let state = helpers::test_state().await;

    create_channel_impl(
        &state,
        CreateChannelInput {
            name: "Disabled".to_string(),
            base_url: "https://disabled.example.com".to_string(),
            channel_type: None,
            api_key: None,
            api_keys: None,
            auth_type: None,
            models_endpoint: None,
            chat_endpoint: None,
            stream_endpoint: None,
            thinking_tags: None,
            enabled: Some(false),
        },
    )
    .await
    .unwrap();

    let listed = list_channels_impl(&state, None).await.unwrap();
    assert_eq!(listed.len(), 1);
    assert!(!listed[0].enabled);
}

/// 参数非法时应返回结构化错误。
#[tokio::test]
async fn test_create_channel_returns_structured_validation_error() {
    let state = helpers::test_state().await;

    let error = create_channel_impl(
        &state,
        CreateChannelInput {
            name: "Bad".to_string(),
            base_url: "api.openai.com".to_string(),
            channel_type: None,
            api_key: None,
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
    .unwrap_err();

    assert_eq!(
        error,
        AppError::validation(
            "INVALID_URL",
            "base_url must start with http:// or https://"
        )
    );
}

/// test_channel 读取缺失资源时应返回 NOT_FOUND。
#[tokio::test]
async fn test_test_channel_returns_not_found_for_missing_channel() {
    let state = helpers::test_state().await;

    let error = test_channel_impl(&state, "missing".to_string())
        .await
        .unwrap_err();
    assert_eq!(error, AppError::not_found("channel 'missing' not found"));
}
