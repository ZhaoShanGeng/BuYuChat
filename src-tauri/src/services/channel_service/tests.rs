//! 使用假仓储与假探测器的渠道服务单元测试。

use std::{
    collections::VecDeque,
    sync::{Mutex, MutexGuard},
};

use async_trait::async_trait;

use crate::{
    ai::adapter::{AiChannelConfig, AiMetadataClient},
    error::AppError,
    models::{
        Channel, ChannelPatch, ChannelTestResult, CreateChannelInput, NewChannel, RemoteModelInfo,
        UpdateChannelInput,
    },
    repo::channel_repo::ChannelRepo,
};

use super::{create_with, delete_with, get_with, list_with, test_with, update_with, Clock};

#[derive(Default)]
struct FakeRepo {
    channels: Mutex<Vec<Channel>>,
}

impl FakeRepo {
    /// 创建一个预置单条渠道数据的假仓储。
    fn with_channel(channel: Channel) -> Self {
        Self {
            channels: Mutex::new(vec![channel]),
        }
    }
}

/// 获取假仓储中的渠道集合锁。
fn lock_channels(repo: &FakeRepo) -> MutexGuard<'_, Vec<Channel>> {
    repo.channels.lock().expect("假仓储互斥锁不应中毒")
}

#[async_trait]
impl ChannelRepo for FakeRepo {
    /// 向内存集合插入渠道。
    async fn insert(&self, new_channel: &NewChannel) -> Result<Channel, String> {
        let channel = Channel {
            id: new_channel.id.clone(),
            name: new_channel.name.clone(),
            channel_type: new_channel.channel_type.clone(),
            base_url: new_channel.base_url.clone(),
            api_key: new_channel.api_key.clone(),
            api_keys: new_channel.api_keys.clone(),
            auth_type: new_channel.auth_type.clone(),
            models_endpoint: new_channel.models_endpoint.clone(),
            chat_endpoint: new_channel.chat_endpoint.clone(),
            stream_endpoint: new_channel.stream_endpoint.clone(),
            thinking_tags: new_channel.thinking_tags.clone(),
            enabled: new_channel.enabled,
            created_at: new_channel.created_at,
            updated_at: new_channel.updated_at,
        };
        lock_channels(self).push(channel.clone());
        Ok(channel)
    }

    /// 从内存集合列出渠道。
    async fn list(&self, include_disabled: bool) -> Result<Vec<Channel>, String> {
        Ok(self
            .channels
            .lock()
            .expect("假仓储互斥锁不应中毒")
            .iter()
            .filter(|channel| include_disabled || channel.enabled)
            .cloned()
            .collect())
    }

    /// 按 ID 从内存集合读取渠道。
    async fn get(&self, id: &str) -> Result<Option<Channel>, String> {
        Ok(self
            .channels
            .lock()
            .expect("假仓储互斥锁不应中毒")
            .iter()
            .find(|channel| channel.id == id)
            .cloned())
    }

    /// 在内存集合中更新单个渠道。
    async fn update(&self, id: &str, patch: &ChannelPatch) -> Result<Option<Channel>, String> {
        let mut channels = lock_channels(self);
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
        if let Some(api_keys) = &patch.api_keys {
            channel.api_keys = Some(api_keys.clone());
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
        if let Some(thinking_tags) = &patch.thinking_tags {
            channel.thinking_tags = Some(thinking_tags.clone());
        }
        if let Some(enabled) = patch.enabled {
            channel.enabled = enabled;
        }
        channel.updated_at = patch.updated_at;
        Ok(Some(channel.clone()))
    }

    /// 从内存集合删除渠道。
    async fn delete(&self, id: &str) -> Result<bool, String> {
        let mut channels = lock_channels(self);
        let before = channels.len();
        channels.retain(|channel| channel.id != id);
        Ok(channels.len() != before)
    }
}

/// 固定时间戳时钟。
struct FixedClock(i64);

#[async_trait]
impl Clock for FixedClock {
    /// 返回测试预设的固定时间戳。
    async fn now_ms(&self) -> i64 {
        self.0
    }
}

/// 可按顺序返回结果的假探测器。
#[derive(Default)]
struct FakeMetadataClient {
    responses: Mutex<VecDeque<Result<(), String>>>,
}

#[async_trait]
impl AiMetadataClient for FakeMetadataClient {
    /// 返回队列中的下一条探测结果。
    async fn probe_models_endpoint(
        &self,
        _http_client: &reqwest::Client,
        _config: &AiChannelConfig,
    ) -> Result<(), AppError> {
        self.responses
            .lock()
            .expect("假探测器互斥锁不应中毒")
            .pop_front()
            .unwrap_or(Ok(()))
            .map_err(|error| {
                AppError::channel_unreachable(format!("failed to reach channel: {error}"))
            })
    }

    /// 渠道服务测试不会调用远程模型拉取。
    async fn fetch_remote_models(
        &self,
        _http_client: &reqwest::Client,
        _config: &AiChannelConfig,
    ) -> Result<Vec<RemoteModelInfo>, AppError> {
        unreachable!("渠道服务测试不会调用远程模型拉取")
    }
}

/// 构造一个默认渠道样本。
fn sample_channel() -> Channel {
    Channel {
        id: "0195d4f5-3af3-7c13-8d69-f4f4bb73f0aa".to_string(),
        name: "My OpenAI".to_string(),
        channel_type: "openai_compatible".to_string(),
        base_url: "https://api.openai.com/".to_string(),
        api_key: Some("sk-xxx".to_string()),
        api_keys: None,
        auth_type: None,
        models_endpoint: None,
        chat_endpoint: None,
        stream_endpoint: None,
        thinking_tags: None,
        enabled: true,
        created_at: 10,
        updated_at: 10,
    }
}

#[tokio::test]
/// 创建渠道时应自动补齐默认值并生成 UUID v7。
async fn create_channel_sets_defaults_and_uuid_v7() {
    let repo = FakeRepo::default();
    let channel = create_with(
        &repo,
        &FixedClock(123),
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

    assert_eq!(channel.channel_type, "openai_compatible");
    assert_eq!(channel.auth_type.as_deref(), Some("bearer"));
    assert_eq!(channel.models_endpoint.as_deref(), Some("/v1/models"));
    assert!(channel.enabled);
    assert_eq!(channel.id.len(), 36);
}

#[tokio::test]
/// 非法 URL 应返回 INVALID_URL。
async fn create_channel_with_invalid_url_returns_invalid_url_error() {
    let repo = FakeRepo::default();
    let err = create_with(
        &repo,
        &FixedClock(123),
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
        err,
        AppError::validation(
            "INVALID_URL",
            "base_url must start with http:// or https://"
        )
    );
}

#[tokio::test]
/// include_disabled 标志应影响列表结果。
async fn list_channels_uses_include_disabled_flag() {
    let mut disabled = sample_channel();
    disabled.enabled = false;
    let repo = FakeRepo {
        channels: Mutex::new(vec![sample_channel(), disabled]),
    };

    assert_eq!(list_with(&repo, false).await.unwrap().len(), 1);
    assert_eq!(list_with(&repo, true).await.unwrap().len(), 2);
}

#[tokio::test]
/// 更新渠道时只改动提交字段。
async fn update_channel_modifies_only_supplied_fields() {
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
    .await
    .unwrap();

    assert_eq!(updated.name, "OpenAI Pro");
    assert_eq!(updated.base_url, "https://api.openai.com/");
    assert_eq!(updated.updated_at, 200);
}

#[tokio::test]
/// 缺失渠道读取应返回 NOT_FOUND。
async fn get_missing_channel_returns_not_found() {
    let err = get_with(&FakeRepo::default(), "missing").await.unwrap_err();
    assert_eq!(err, AppError::not_found("channel 'missing' not found"));
}

#[tokio::test]
/// 缺失渠道删除应返回 NOT_FOUND。
async fn delete_missing_channel_returns_not_found() {
    let err = delete_with(&FakeRepo::default(), "missing")
        .await
        .unwrap_err();
    assert_eq!(err, AppError::not_found("channel 'missing' not found"));
}

#[tokio::test]
/// 探测成功时应返回成功结果。
async fn test_channel_returns_success_when_probe_succeeds() {
    let repo = FakeRepo::with_channel(sample_channel());
    let probe = FakeMetadataClient::default();

    let result = test_with(
        &repo,
        &probe,
        &reqwest::Client::new(),
        "0195d4f5-3af3-7c13-8d69-f4f4bb73f0aa",
    )
    .await
    .unwrap();

    assert_eq!(
        result,
        ChannelTestResult {
            success: true,
            message: Some("channel is reachable".to_string())
        }
    );
}

#[tokio::test]
/// 探测失败应映射为 CHANNEL_UNREACHABLE。
async fn test_channel_maps_probe_failures_to_channel_unreachable() {
    let repo = FakeRepo::with_channel(sample_channel());
    let probe = FakeMetadataClient {
        responses: Mutex::new(VecDeque::from([Err("network down".to_string())])),
    };

    let err = test_with(
        &repo,
        &probe,
        &reqwest::Client::new(),
        "0195d4f5-3af3-7c13-8d69-f4f4bb73f0aa",
    )
    .await
    .unwrap_err();

    assert_eq!(
        err,
        AppError::channel_unreachable("failed to reach channel: network down")
    );
}
