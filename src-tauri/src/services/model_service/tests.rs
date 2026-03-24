//! 使用假仓储与假 AI 适配器的模型服务单元测试。

use std::sync::{Mutex, MutexGuard};

use async_trait::async_trait;

use crate::{
    ai::adapter::{AiChannelConfig, AiMetadataClient},
    error::AppError,
    models::{
        Channel, ChannelModel, ChannelModelPatch, CreateModelInput, NewChannelModel,
        RemoteModelInfo, UpdateModelInput,
    },
    repo::{channel_repo::ChannelRepo, model_repo::ModelRepo},
    services::model_service::{
        create_with, delete_with, fetch_remote_models_with, list_with, update_with,
    },
};

/// 基于内存集合的假模型仓储。
#[derive(Default)]
struct FakeModelRepo {
    channels: Mutex<Vec<String>>,
    models: Mutex<Vec<ChannelModel>>,
}

impl FakeModelRepo {
    /// 创建带有单个渠道和单个模型的假仓储。
    fn with_model(channel_id: &str, model: ChannelModel) -> Self {
        Self {
            channels: Mutex::new(vec![channel_id.to_string()]),
            models: Mutex::new(vec![model]),
        }
    }
}

/// 获取假模型仓储中的模型集合锁。
fn lock_models(repo: &FakeModelRepo) -> MutexGuard<'_, Vec<ChannelModel>> {
    repo.models.lock().expect("假模型仓储互斥锁不应中毒")
}

#[async_trait]
impl ModelRepo for FakeModelRepo {
    /// 判断渠道是否存在。
    async fn channel_exists(&self, channel_id: &str) -> Result<bool, String> {
        Ok(self
            .channels
            .lock()
            .expect("假模型仓储互斥锁不应中毒")
            .iter()
            .any(|candidate| candidate == channel_id))
    }

    /// 判断同渠道下是否已存在同名模型。
    async fn model_id_exists(&self, channel_id: &str, model_id: &str) -> Result<bool, String> {
        Ok(lock_models(self)
            .iter()
            .any(|model| model.channel_id == channel_id && model.model_id == model_id))
    }

    /// 向内存集合插入模型。
    async fn insert(&self, new_model: &NewChannelModel) -> Result<ChannelModel, String> {
        let model = ChannelModel {
            id: new_model.id.clone(),
            channel_id: new_model.channel_id.clone(),
            model_id: new_model.model_id.clone(),
            display_name: new_model.display_name.clone(),
            context_window: new_model.context_window,
            max_output_tokens: new_model.max_output_tokens,
        };
        lock_models(self).push(model.clone());
        Ok(model)
    }

    /// 按渠道列出模型。
    async fn list_by_channel(&self, channel_id: &str) -> Result<Vec<ChannelModel>, String> {
        Ok(lock_models(self)
            .iter()
            .filter(|model| model.channel_id == channel_id)
            .cloned()
            .collect())
    }

    /// 按渠道与模型 ID 获取模型。
    async fn get_by_channel_and_id(
        &self,
        channel_id: &str,
        id: &str,
    ) -> Result<Option<ChannelModel>, String> {
        Ok(lock_models(self)
            .iter()
            .find(|model| model.channel_id == channel_id && model.id == id)
            .cloned())
    }

    /// 在内存集合中更新模型。
    async fn update(
        &self,
        channel_id: &str,
        id: &str,
        patch: &ChannelModelPatch,
    ) -> Result<Option<ChannelModel>, String> {
        let mut models = lock_models(self);
        let Some(model) = models
            .iter_mut()
            .find(|model| model.channel_id == channel_id && model.id == id)
        else {
            return Ok(None);
        };

        if let Some(display_name) = &patch.display_name {
            model.display_name = display_name.clone();
        }
        if let Some(context_window) = patch.context_window {
            model.context_window = context_window;
        }
        if let Some(max_output_tokens) = patch.max_output_tokens {
            model.max_output_tokens = max_output_tokens;
        }

        Ok(Some(model.clone()))
    }

    /// 从内存集合删除模型。
    async fn delete(&self, channel_id: &str, id: &str) -> Result<bool, String> {
        let mut models = lock_models(self);
        let before = models.len();
        models.retain(|model| !(model.channel_id == channel_id && model.id == id));
        Ok(models.len() != before)
    }
}

/// 基于固定渠道样本的假渠道仓储。
struct FakeChannelRepo(Channel);

#[async_trait]
impl ChannelRepo for FakeChannelRepo {
    /// 测试中不会调用该方法。
    async fn insert(&self, _new_channel: &crate::models::NewChannel) -> Result<Channel, String> {
        unreachable!("模型服务测试不会调用渠道插入")
    }

    /// 测试中不会调用该方法。
    async fn list(&self, _include_disabled: bool) -> Result<Vec<Channel>, String> {
        unreachable!("模型服务测试不会调用渠道列表")
    }

    /// 返回预置的渠道样本。
    async fn get(&self, id: &str) -> Result<Option<Channel>, String> {
        Ok((self.0.id == id).then(|| self.0.clone()))
    }

    /// 测试中不会调用该方法。
    async fn update(
        &self,
        _id: &str,
        _patch: &crate::models::ChannelPatch,
    ) -> Result<Option<Channel>, String> {
        unreachable!("模型服务测试不会调用渠道更新")
    }

    /// 测试中不会调用该方法。
    async fn delete(&self, _id: &str) -> Result<bool, String> {
        unreachable!("模型服务测试不会调用渠道删除")
    }
}

/// 可返回固定模型列表的假 AI 适配器。
#[derive(Default)]
struct FakeMetadataClient {
    fetched: Mutex<Vec<RemoteModelInfo>>,
}

#[async_trait]
impl AiMetadataClient for FakeMetadataClient {
    /// 测试中不会调用该方法。
    async fn probe_models_endpoint(
        &self,
        _http_client: &reqwest::Client,
        _config: &AiChannelConfig,
    ) -> Result<(), AppError> {
        unreachable!("模型服务测试不会调用连通性探测")
    }

    /// 返回预置的远程模型列表。
    async fn fetch_remote_models(
        &self,
        _http_client: &reqwest::Client,
        _config: &AiChannelConfig,
    ) -> Result<Vec<RemoteModelInfo>, AppError> {
        Ok(self
            .fetched
            .lock()
            .expect("假 AI 适配器互斥锁不应中毒")
            .clone())
    }
}

/// 构造一个默认模型样本。
fn sample_model(channel_id: &str) -> ChannelModel {
    ChannelModel {
        id: "0195d4f5-3af3-7c13-8d69-f4f4bb73f0aa".to_string(),
        channel_id: channel_id.to_string(),
        model_id: "gpt-4o".to_string(),
        display_name: Some("GPT-4o".to_string()),
        context_window: Some(128_000),
        max_output_tokens: Some(16_384),
    }
}

/// 构造一个默认渠道样本。
fn sample_channel() -> Channel {
    Channel {
        id: "0195d4f5-3af3-7c13-8d69-f4f4bb73f0ab".to_string(),
        name: "My OpenAI".to_string(),
        channel_type: "openai_compatible".to_string(),
        base_url: "https://api.openai.com".to_string(),
        api_key: Some("sk-xxx".to_string()),
        auth_type: Some("bearer".to_string()),
        models_endpoint: Some("/v1/models".to_string()),
        chat_endpoint: Some("/v1/chat/completions".to_string()),
        stream_endpoint: Some("/v1/chat/completions".to_string()),
        enabled: true,
        created_at: 100,
        updated_at: 100,
    }
}

/// 创建模型时应自动裁剪字符串并生成 UUID。
#[tokio::test]
async fn create_model_trims_fields_and_generates_uuid() {
    let channel_id = sample_channel().id;
    let repo = FakeModelRepo {
        channels: Mutex::new(vec![channel_id.clone()]),
        ..FakeModelRepo::default()
    };

    let created = create_with(
        &repo,
        &channel_id,
        CreateModelInput {
            model_id: "  gpt-4o  ".to_string(),
            display_name: Some("  GPT-4o  ".to_string()),
            context_window: Some(128_000),
            max_output_tokens: Some(16_384),
        },
    )
    .await
    .unwrap();

    assert_eq!(created.model_id, "gpt-4o");
    assert_eq!(created.display_name.as_deref(), Some("GPT-4o"));
    assert_eq!(created.id.len(), 36);
}

/// 空 model_id 应返回结构化校验错误。
#[tokio::test]
async fn create_model_with_empty_model_id_returns_validation_error() {
    let repo = FakeModelRepo {
        channels: Mutex::new(vec!["channel-1".to_string()]),
        ..FakeModelRepo::default()
    };

    let error = create_with(
        &repo,
        "channel-1",
        CreateModelInput {
            model_id: "   ".to_string(),
            display_name: None,
            context_window: None,
            max_output_tokens: None,
        },
    )
    .await
    .unwrap_err();

    assert_eq!(
        error,
        AppError::validation("VALIDATION_ERROR", "model_id cannot be empty")
    );
}

/// 重复 model_id 应返回 MODEL_ID_CONFLICT。
#[tokio::test]
async fn create_model_with_duplicate_model_id_returns_conflict() {
    let channel = sample_channel();
    let repo = FakeModelRepo::with_model(&channel.id, sample_model(&channel.id));

    let error = create_with(
        &repo,
        &channel.id,
        CreateModelInput {
            model_id: "gpt-4o".to_string(),
            display_name: None,
            context_window: None,
            max_output_tokens: None,
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

/// 缺失渠道时，模型列表应返回 NOT_FOUND。
#[tokio::test]
async fn list_models_returns_not_found_when_channel_missing() {
    let error = list_with(&FakeModelRepo::default(), "missing")
        .await
        .unwrap_err();

    assert_eq!(error, AppError::not_found("channel 'missing' not found"));
}

/// 更新模型时应只修改允许的字段。
#[tokio::test]
async fn update_model_changes_only_allowed_fields() {
    let channel = sample_channel();
    let original = sample_model(&channel.id);
    let repo = FakeModelRepo::with_model(&channel.id, original.clone());

    let updated = update_with(
        &repo,
        &channel.id,
        &original.id,
        UpdateModelInput {
            display_name: Some(Some("GPT-4o Latest".to_string())),
            context_window: Some(Some(200_000)),
            max_output_tokens: None,
        },
    )
    .await
    .unwrap();

    assert_eq!(updated.model_id, "gpt-4o");
    assert_eq!(updated.display_name.as_deref(), Some("GPT-4o Latest"));
    assert_eq!(updated.context_window, Some(200_000));
    assert_eq!(updated.max_output_tokens, Some(16_384));
}

/// 删除缺失模型时应返回 NOT_FOUND。
#[tokio::test]
async fn delete_missing_model_returns_not_found() {
    let channel = sample_channel();
    let repo = FakeModelRepo {
        channels: Mutex::new(vec![channel.id.clone()]),
        ..FakeModelRepo::default()
    };

    let error = delete_with(&repo, &channel.id, "missing")
        .await
        .unwrap_err();

    assert_eq!(error, AppError::not_found("model 'missing' not found"));
}

/// 缺失渠道时，远程拉取应返回 NOT_FOUND。
#[tokio::test]
async fn fetch_remote_models_returns_not_found_when_channel_missing() {
    let adapter = FakeMetadataClient::default();
    let error = fetch_remote_models_with(
        &FakeChannelRepo(sample_channel()),
        &adapter,
        &reqwest::Client::new(),
        "missing",
    )
    .await
    .unwrap_err();

    assert_eq!(error, AppError::not_found("channel 'missing' not found"));
}

/// 远程拉取应委托 AI 适配层并透传解析结果。
#[tokio::test]
async fn fetch_remote_models_delegates_to_ai_adapter() {
    let channel = sample_channel();
    let adapter = FakeMetadataClient {
        fetched: Mutex::new(vec![RemoteModelInfo {
            model_id: "gpt-4o".to_string(),
            display_name: Some("GPT-4o".to_string()),
            context_window: Some(128_000),
        }]),
    };

    let models = fetch_remote_models_with(
        &FakeChannelRepo(channel.clone()),
        &adapter,
        &reqwest::Client::new(),
        &channel.id,
    )
    .await
    .unwrap();

    assert_eq!(models.len(), 1);
    assert_eq!(models[0].model_id, "gpt-4o");
    assert_eq!(models[0].display_name.as_deref(), Some("GPT-4o"));
}
