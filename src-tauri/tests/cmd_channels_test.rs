use buyu_lib::{
    commands::channels::{
        create_channel_impl, delete_channel_impl, get_channel_impl, list_channels_impl,
        test_channel_impl, update_channel_impl,
    },
    db::init_db,
    errors::AppError,
    models::{CreateChannelInput, UpdateChannelInput},
};

#[test]
fn create_get_update_list_and_delete_channel_via_command_impls() {
    let db = init_db().unwrap();

    let created = create_channel_impl(
        &db,
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
    .unwrap();
    let channel_id = created.id.clone();

    let listed = list_channels_impl(&db, None).unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, created.id);

    let fetched = get_channel_impl(&db, created.id.clone()).unwrap();
    assert_eq!(fetched.name, "My OpenAI");

    let updated = update_channel_impl(
        &db,
        created.id.clone(),
        UpdateChannelInput {
            name: Some("OpenAI Pro".to_string()),
            ..UpdateChannelInput::default()
        },
    )
    .unwrap();
    assert_eq!(updated.name, "OpenAI Pro");

    delete_channel_impl(&db, channel_id.clone()).unwrap();

    let err = get_channel_impl(&db, channel_id.clone()).unwrap_err();
    assert_eq!(
        err,
        AppError::not_found(format!("channel '{channel_id}' not found"))
    );
}

#[test]
fn list_channels_defaults_to_include_disabled_true() {
    let db = init_db().unwrap();
    let created = create_channel_impl(
        &db,
        CreateChannelInput {
            name: "Disabled".to_string(),
            base_url: "https://disabled.example.com".to_string(),
            channel_type: None,
            api_key: None,
            auth_type: None,
            models_endpoint: None,
            chat_endpoint: None,
            stream_endpoint: None,
            enabled: Some(false),
        },
    )
    .unwrap();

    let listed = list_channels_impl(&db, None).unwrap();

    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, created.id);
}

#[test]
fn create_channel_invalid_url_returns_structured_error() {
    let db = init_db().unwrap();

    let err = create_channel_impl(
        &db,
        CreateChannelInput {
            name: "Bad".to_string(),
            base_url: "api.openai.com".to_string(),
            channel_type: None,
            api_key: None,
            auth_type: None,
            models_endpoint: None,
            chat_endpoint: None,
            stream_endpoint: None,
            enabled: None,
        },
    )
    .unwrap_err();

    assert_eq!(
        err,
        AppError::validation(
            "INVALID_URL",
            "base_url must start with http:// or https://"
        )
    );
}

#[test]
fn test_channel_missing_resource_returns_not_found() {
    let db = init_db().unwrap();

    let err = test_channel_impl(&db, "missing".to_string()).unwrap_err();

    assert_eq!(err, AppError::not_found("channel 'missing' not found"));
}
