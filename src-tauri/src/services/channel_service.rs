use uuid::Uuid;

use crate::{
    channel_types::{config_for, ChannelTypeConfig},
    db::Db,
    errors::AppError,
    models::{
        Channel, ChannelPatch, ChannelTestResult, CreateChannelInput, NewChannel,
        TestChannelRequest, UpdateChannelInput,
    },
    repo::channel_repo::{ChannelRepo, SqliteChannelRepo},
};

pub trait Clock {
    fn now_ms(&self) -> i64;
}

pub trait IdGenerator {
    fn new_id(&self) -> String;
}

pub trait HttpClient {
    fn get(&self, request: &TestChannelRequest) -> Result<(), String>;
}

pub struct SystemClock;

impl Clock for SystemClock {
    fn now_ms(&self) -> i64 {
        chrono::Utc::now().timestamp_millis()
    }
}

pub struct UuidV7Generator;

impl IdGenerator for UuidV7Generator {
    fn new_id(&self) -> String {
        Uuid::now_v7().to_string()
    }
}

pub struct ReqwestHttpClient(reqwest::blocking::Client);

impl Default for ReqwestHttpClient {
    fn default() -> Self {
        Self(reqwest::blocking::Client::new())
    }
}

impl HttpClient for ReqwestHttpClient {
    fn get(&self, request: &TestChannelRequest) -> Result<(), String> {
        let mut builder = self.0.get(&request.url);
        if let Some((name, value)) = &request.auth_header {
            builder = builder.header(name, value);
        }

        let response = builder.send().map_err(|err| err.to_string())?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("remote endpoint returned {}", response.status()))
        }
    }
}

fn validate_name(name: &str) -> Result<(), AppError> {
    if name.trim().is_empty() {
        return Err(AppError::validation(
            "NAME_EMPTY",
            "channel name cannot be empty",
        ));
    }
    Ok(())
}

fn validate_base_url(base_url: &str) -> Result<(), AppError> {
    if !(base_url.starts_with("http://") || base_url.starts_with("https://")) {
        return Err(AppError::validation(
            "INVALID_URL",
            "base_url must start with http:// or https://",
        ));
    }
    Ok(())
}

fn resolve_config(channel_type: Option<&str>) -> Result<ChannelTypeConfig, AppError> {
    config_for(channel_type.unwrap_or("openai_compatible"))
}

fn build_test_request(channel: &Channel) -> Result<TestChannelRequest, AppError> {
    let config = resolve_config(Some(&channel.channel_type))?;
    let endpoint = channel
        .models_endpoint
        .clone()
        .unwrap_or_else(|| config.models_endpoint.to_string());
    let auth_type = channel
        .auth_type
        .clone()
        .unwrap_or_else(|| config.auth_type.to_string());

    let auth_header = match auth_type.as_str() {
        "bearer" => channel
            .api_key
            .clone()
            .map(|key| ("Authorization".to_string(), format!("Bearer {key}"))),
        "x_api_key" => channel
            .api_key
            .clone()
            .map(|key| ("x-api-key".to_string(), key)),
        "none" => None,
        other => {
            return Err(AppError::validation(
                "VALIDATION_ERROR",
                format!("unsupported auth_type '{other}'"),
            ));
        }
    };

    Ok(TestChannelRequest {
        url: format!("{}{}", channel.base_url.trim_end_matches('/'), endpoint),
        auth_header,
    })
}

pub fn create_with<R: ChannelRepo, I: IdGenerator, C: Clock>(
    repo: &R,
    ids: &I,
    clock: &C,
    input: CreateChannelInput,
) -> Result<Channel, AppError> {
    validate_name(&input.name)?;
    validate_base_url(&input.base_url)?;
    let config = resolve_config(input.channel_type.as_deref())?;
    let ts = clock.now_ms();

    repo.insert(&NewChannel {
        id: ids.new_id(),
        name: input.name.trim().to_string(),
        channel_type: config.channel_type.to_string(),
        base_url: input.base_url.trim().to_string(),
        api_key: input.api_key,
        auth_type: input
            .auth_type
            .or_else(|| Some(config.auth_type.to_string())),
        models_endpoint: input
            .models_endpoint
            .or_else(|| Some(config.models_endpoint.to_string())),
        chat_endpoint: input
            .chat_endpoint
            .or_else(|| Some(config.chat_endpoint.to_string())),
        stream_endpoint: input
            .stream_endpoint
            .or_else(|| Some(config.stream_endpoint.to_string())),
        enabled: input.enabled.unwrap_or(true),
        created_at: ts,
        updated_at: ts,
    })
    .map_err(|err| AppError::internal(format!("failed to create channel: {err}")))
}

pub fn list_with<R: ChannelRepo>(
    repo: &R,
    include_disabled: bool,
) -> Result<Vec<Channel>, AppError> {
    repo.list(include_disabled)
        .map_err(|err| AppError::internal(format!("failed to list channels: {err}")))
}

pub fn get_with<R: ChannelRepo>(repo: &R, id: &str) -> Result<Channel, AppError> {
    repo.get(id)
        .map_err(|err| AppError::internal(format!("failed to get channel: {err}")))?
        .ok_or_else(|| AppError::not_found(format!("channel '{id}' not found")))
}

pub fn update_with<R: ChannelRepo, C: Clock>(
    repo: &R,
    clock: &C,
    id: &str,
    input: UpdateChannelInput,
) -> Result<Channel, AppError> {
    if let Some(name) = &input.name {
        validate_name(name)?;
    }
    if let Some(base_url) = &input.base_url {
        validate_base_url(base_url)?;
    }
    if let Some(channel_type) = input.channel_type.as_deref() {
        resolve_config(Some(channel_type))?;
    }

    repo.update(
        id,
        &ChannelPatch {
            name: input.name.map(|value| value.trim().to_string()),
            base_url: input.base_url.map(|value| value.trim().to_string()),
            channel_type: input.channel_type,
            api_key: input.api_key,
            auth_type: input.auth_type,
            models_endpoint: input.models_endpoint,
            chat_endpoint: input.chat_endpoint,
            stream_endpoint: input.stream_endpoint,
            enabled: input.enabled,
            updated_at: clock.now_ms(),
        },
    )
    .map_err(|err| AppError::internal(format!("failed to update channel: {err}")))?
    .ok_or_else(|| AppError::not_found(format!("channel '{id}' not found")))
}

pub fn delete_with<R: ChannelRepo>(repo: &R, id: &str) -> Result<(), AppError> {
    match repo
        .delete(id)
        .map_err(|err| AppError::internal(format!("failed to delete channel: {err}")))?
    {
        true => Ok(()),
        false => Err(AppError::not_found(format!("channel '{id}' not found"))),
    }
}

pub fn test_with<R: ChannelRepo, H: HttpClient>(
    repo: &R,
    http_client: &H,
    id: &str,
) -> Result<ChannelTestResult, AppError> {
    let channel = get_with(repo, id)?;
    let request = build_test_request(&channel)?;
    http_client
        .get(&request)
        .map_err(|err| AppError::channel_unreachable(format!("failed to reach channel: {err}")))?;

    Ok(ChannelTestResult {
        success: true,
        message: Some("channel is reachable".to_string()),
    })
}

pub fn create(db: &Db, input: CreateChannelInput) -> Result<Channel, AppError> {
    let conn = db
        .lock()
        .map_err(|_| AppError::internal("db mutex poisoned"))?;
    let repo = SqliteChannelRepo::new(&conn);
    create_with(&repo, &UuidV7Generator, &SystemClock, input)
}

pub fn list(db: &Db, include_disabled: bool) -> Result<Vec<Channel>, AppError> {
    let conn = db
        .lock()
        .map_err(|_| AppError::internal("db mutex poisoned"))?;
    let repo = SqliteChannelRepo::new(&conn);
    list_with(&repo, include_disabled)
}

pub fn get(db: &Db, id: &str) -> Result<Channel, AppError> {
    let conn = db
        .lock()
        .map_err(|_| AppError::internal("db mutex poisoned"))?;
    let repo = SqliteChannelRepo::new(&conn);
    get_with(&repo, id)
}

pub fn update(db: &Db, id: &str, input: UpdateChannelInput) -> Result<Channel, AppError> {
    let conn = db
        .lock()
        .map_err(|_| AppError::internal("db mutex poisoned"))?;
    let repo = SqliteChannelRepo::new(&conn);
    update_with(&repo, &SystemClock, id, input)
}

pub fn delete(db: &Db, id: &str) -> Result<(), AppError> {
    let conn = db
        .lock()
        .map_err(|_| AppError::internal("db mutex poisoned"))?;
    let repo = SqliteChannelRepo::new(&conn);
    delete_with(&repo, id)
}

pub fn test_channel(db: &Db, id: &str) -> Result<ChannelTestResult, AppError> {
    let conn = db
        .lock()
        .map_err(|_| AppError::internal("db mutex poisoned"))?;
    let repo = SqliteChannelRepo::new(&conn);
    test_with(&repo, &ReqwestHttpClient::default(), id)
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, collections::VecDeque};

    use crate::{
        errors::AppError,
        models::{CreateChannelInput, UpdateChannelInput},
        repo::channel_repo::ChannelRepo,
    };

    use super::{
        build_test_request, create_with, delete_with, get_with, list_with, test_with, update_with,
        Channel, ChannelPatch, ChannelTestResult, Clock, HttpClient, IdGenerator, NewChannel,
    };

    #[derive(Default)]
    struct FakeRepo {
        channels: RefCell<Vec<Channel>>,
    }

    impl FakeRepo {
        fn with_channel(channel: Channel) -> Self {
            Self {
                channels: RefCell::new(vec![channel]),
            }
        }
    }

    impl ChannelRepo for FakeRepo {
        fn insert(&self, new_channel: &NewChannel) -> Result<Channel, String> {
            let channel = Channel {
                id: new_channel.id.clone(),
                name: new_channel.name.clone(),
                channel_type: new_channel.channel_type.clone(),
                base_url: new_channel.base_url.clone(),
                api_key: new_channel.api_key.clone(),
                auth_type: new_channel.auth_type.clone(),
                models_endpoint: new_channel.models_endpoint.clone(),
                chat_endpoint: new_channel.chat_endpoint.clone(),
                stream_endpoint: new_channel.stream_endpoint.clone(),
                enabled: new_channel.enabled,
                created_at: new_channel.created_at,
                updated_at: new_channel.updated_at,
            };
            self.channels.borrow_mut().push(channel.clone());
            Ok(channel)
        }

        fn list(&self, include_disabled: bool) -> Result<Vec<Channel>, String> {
            Ok(self
                .channels
                .borrow()
                .iter()
                .filter(|channel| include_disabled || channel.enabled)
                .cloned()
                .collect())
        }

        fn get(&self, id: &str) -> Result<Option<Channel>, String> {
            Ok(self
                .channels
                .borrow()
                .iter()
                .find(|channel| channel.id == id)
                .cloned())
        }

        fn update(&self, id: &str, patch: &ChannelPatch) -> Result<Option<Channel>, String> {
            let mut channels = self.channels.borrow_mut();
            let Some(channel) = channels.iter_mut().find(|channel| channel.id == id) else {
                return Ok(None);
            };

            if let Some(name) = &patch.name {
                channel.name = name.clone();
            }
            if let Some(base_url) = &patch.base_url {
                channel.base_url = base_url.clone();
            }
            if let Some(channel_type) = &patch.channel_type {
                channel.channel_type = channel_type.clone();
            }
            if let Some(api_key) = &patch.api_key {
                channel.api_key = Some(api_key.clone());
            }
            if let Some(auth_type) = &patch.auth_type {
                channel.auth_type = Some(auth_type.clone());
            }
            if let Some(models_endpoint) = &patch.models_endpoint {
                channel.models_endpoint = Some(models_endpoint.clone());
            }
            if let Some(chat_endpoint) = &patch.chat_endpoint {
                channel.chat_endpoint = Some(chat_endpoint.clone());
            }
            if let Some(stream_endpoint) = &patch.stream_endpoint {
                channel.stream_endpoint = Some(stream_endpoint.clone());
            }
            if let Some(enabled) = patch.enabled {
                channel.enabled = enabled;
            }
            channel.updated_at = patch.updated_at;
            Ok(Some(channel.clone()))
        }

        fn delete(&self, id: &str) -> Result<bool, String> {
            let mut channels = self.channels.borrow_mut();
            let before = channels.len();
            channels.retain(|channel| channel.id != id);
            Ok(channels.len() != before)
        }
    }

    struct FixedClock(i64);

    impl Clock for FixedClock {
        fn now_ms(&self) -> i64 {
            self.0
        }
    }

    struct FixedIds;

    impl IdGenerator for FixedIds {
        fn new_id(&self) -> String {
            "0195d4f5-3af3-7c13-8d69-f4f4bb73f001".to_string()
        }
    }

    #[derive(Default)]
    struct FakeHttpClient {
        responses: RefCell<VecDeque<Result<(), String>>>,
    }

    impl HttpClient for FakeHttpClient {
        fn get(&self, _request: &crate::models::TestChannelRequest) -> Result<(), String> {
            self.responses.borrow_mut().pop_front().unwrap_or(Ok(()))
        }
    }

    fn sample_channel() -> Channel {
        Channel {
            id: "0195d4f5-3af3-7c13-8d69-f4f4bb73f0aa".to_string(),
            name: "My OpenAI".to_string(),
            channel_type: "openai_compatible".to_string(),
            base_url: "https://api.openai.com/".to_string(),
            api_key: Some("sk-xxx".to_string()),
            auth_type: None,
            models_endpoint: None,
            chat_endpoint: None,
            stream_endpoint: None,
            enabled: true,
            created_at: 10,
            updated_at: 10,
        }
    }

    #[test]
    fn create_channel_sets_defaults_and_uuid_v7() {
        let repo = FakeRepo::default();
        let channel = create_with(
            &repo,
            &FixedIds,
            &FixedClock(123),
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

        assert_eq!(channel.id, "0195d4f5-3af3-7c13-8d69-f4f4bb73f001");
        assert_eq!(channel.channel_type, "openai_compatible");
        assert_eq!(channel.auth_type.as_deref(), Some("bearer"));
        assert_eq!(channel.models_endpoint.as_deref(), Some("/v1/models"));
        assert!(channel.enabled);
    }

    #[test]
    fn create_channel_with_invalid_url_returns_invalid_url_error() {
        let repo = FakeRepo::default();
        let err = create_with(
            &repo,
            &FixedIds,
            &FixedClock(123),
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
    fn list_channels_uses_include_disabled_flag() {
        let mut disabled = sample_channel();
        disabled.enabled = false;
        let repo = FakeRepo {
            channels: RefCell::new(vec![sample_channel(), disabled]),
        };

        assert_eq!(list_with(&repo, false).unwrap().len(), 1);
        assert_eq!(list_with(&repo, true).unwrap().len(), 2);
    }

    #[test]
    fn update_channel_modifies_only_supplied_fields() {
        let repo = FakeRepo::with_channel(sample_channel());
        let updated = update_with(
            &repo,
            &FixedClock(200),
            "0195d4f5-3af3-7c13-8d69-f4f4bb73f0aa",
            UpdateChannelInput {
                name: Some("OpenAI Pro".to_string()),
                ..UpdateChannelInput::default()
            },
        )
        .unwrap();

        assert_eq!(updated.name, "OpenAI Pro");
        assert_eq!(updated.base_url, "https://api.openai.com/");
        assert_eq!(updated.updated_at, 200);
    }

    #[test]
    fn get_missing_channel_returns_not_found() {
        let err = get_with(&FakeRepo::default(), "missing").unwrap_err();
        assert_eq!(err, AppError::not_found("channel 'missing' not found"));
    }

    #[test]
    fn delete_missing_channel_returns_not_found() {
        let err = delete_with(&FakeRepo::default(), "missing").unwrap_err();
        assert_eq!(err, AppError::not_found("channel 'missing' not found"));
    }

    #[test]
    fn build_test_request_uses_default_models_endpoint_and_auth_header() {
        let request = build_test_request(&sample_channel()).unwrap();
        assert_eq!(request.url, "https://api.openai.com/v1/models");
        assert_eq!(
            request.auth_header,
            Some(("Authorization".to_string(), "Bearer sk-xxx".to_string()))
        );
    }

    #[test]
    fn test_channel_returns_success_when_http_client_succeeds() {
        let repo = FakeRepo::with_channel(sample_channel());
        let http_client = FakeHttpClient::default();

        let result =
            test_with(&repo, &http_client, "0195d4f5-3af3-7c13-8d69-f4f4bb73f0aa").unwrap();

        assert_eq!(
            result,
            ChannelTestResult {
                success: true,
                message: Some("channel is reachable".to_string())
            }
        );
    }

    #[test]
    fn test_channel_maps_network_failures_to_channel_unreachable() {
        let repo = FakeRepo::with_channel(sample_channel());
        let http_client = FakeHttpClient {
            responses: RefCell::new(VecDeque::from([Err("network down".to_string())])),
        };

        let err =
            test_with(&repo, &http_client, "0195d4f5-3af3-7c13-8d69-f4f4bb73f0aa").unwrap_err();

        assert_eq!(
            err,
            AppError::channel_unreachable("failed to reach channel: network down")
        );
    }
}
