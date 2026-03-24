//! 渠道资源的 CRUD 编排逻辑。

use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::{
    error::AppError,
    models::{Channel, ChannelPatch, CreateChannelInput, NewChannel, UpdateChannelInput},
    repo::channel_repo::{ChannelRepo, SqlxChannelRepo},
    utils::ids::new_uuid_v7,
};

use super::validation::{resolve_config, validate_base_url, validate_name};

/// 对当前时间的抽象，便于保持服务层测试稳定。
#[async_trait]
pub trait Clock: Send + Sync {
    /// 返回当前毫秒时间戳。
    async fn now_ms(&self) -> i64;
}

/// 基于 `chrono` 的生产时钟实现。
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

#[async_trait]
impl Clock for SystemClock {
    /// 返回当前毫秒时间戳。
    async fn now_ms(&self) -> i64 {
        chrono::Utc::now().timestamp_millis()
    }
}

/// 使用连接池创建渠道。
pub async fn create(pool: &SqlitePool, input: CreateChannelInput) -> Result<Channel, AppError> {
    let repo = SqlxChannelRepo::new(pool.clone());
    create_with(&repo, &SystemClock, input).await
}

/// 使用连接池列出渠道。
pub async fn list(pool: &SqlitePool, include_disabled: bool) -> Result<Vec<Channel>, AppError> {
    let repo = SqlxChannelRepo::new(pool.clone());
    list_with(&repo, include_disabled).await
}

/// 使用连接池按 ID 获取渠道。
pub async fn get(pool: &SqlitePool, id: &str) -> Result<Channel, AppError> {
    let repo = SqlxChannelRepo::new(pool.clone());
    get_with(&repo, id).await
}

/// 使用连接池更新渠道。
pub async fn update(
    pool: &SqlitePool,
    id: &str,
    input: UpdateChannelInput,
) -> Result<Channel, AppError> {
    let repo = SqlxChannelRepo::new(pool.clone());
    update_with(&repo, &SystemClock, id, input).await
}

/// 使用连接池删除渠道。
pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), AppError> {
    let repo = SqlxChannelRepo::new(pool.clone());
    delete_with(&repo, id).await
}

/// 使用显式依赖创建渠道，便于测试。
pub async fn create_with<R: ChannelRepo, C: Clock>(
    repo: &R,
    clock: &C,
    input: CreateChannelInput,
) -> Result<Channel, AppError> {
    validate_name(&input.name)?;
    validate_base_url(&input.base_url)?;
    let config = resolve_config(input.channel_type.as_deref())?;
    let timestamp = clock.now_ms().await;

    repo.insert(&NewChannel {
        id: new_uuid_v7(),
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
        created_at: timestamp,
        updated_at: timestamp,
    })
    .await
    .map_err(|error| AppError::internal(format!("failed to create channel: {error}")))
}

/// 使用注入的仓储列出渠道。
pub async fn list_with<R: ChannelRepo>(
    repo: &R,
    include_disabled: bool,
) -> Result<Vec<Channel>, AppError> {
    repo.list(include_disabled)
        .await
        .map_err(|error| AppError::internal(format!("failed to list channels: {error}")))
}

/// 使用注入的仓储按 ID 获取渠道。
pub async fn get_with<R: ChannelRepo>(repo: &R, id: &str) -> Result<Channel, AppError> {
    repo.get(id)
        .await
        .map_err(|error| AppError::internal(format!("failed to get channel: {error}")))?
        .ok_or_else(|| AppError::not_found(format!("channel '{id}' not found")))
}

/// 使用注入的仓储与时钟更新渠道。
pub async fn update_with<R: ChannelRepo, C: Clock>(
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
            updated_at: clock.now_ms().await,
        },
    )
    .await
    .map_err(|error| AppError::internal(format!("failed to update channel: {error}")))?
    .ok_or_else(|| AppError::not_found(format!("channel '{id}' not found")))
}

/// 使用注入的仓储删除渠道。
pub async fn delete_with<R: ChannelRepo>(repo: &R, id: &str) -> Result<(), AppError> {
    match repo
        .delete(id)
        .await
        .map_err(|error| AppError::internal(format!("failed to delete channel: {error}")))?
    {
        true => Ok(()),
        false => Err(AppError::not_found(format!("channel '{id}' not found"))),
    }
}
