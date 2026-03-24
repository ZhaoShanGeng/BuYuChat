//! 模型资源的 CRUD 编排逻辑。

use sqlx::SqlitePool;

use crate::{
    error::AppError,
    models::{ChannelModel, ChannelModelPatch, CreateModelInput, NewChannelModel, UpdateModelInput},
    repo::model_repo::{ModelRepo, SqlxModelRepo},
    utils::ids::new_uuid_v7,
};

use super::validation::{normalize_optional_text, validate_model_id};

/// 使用连接池创建模型。
pub async fn create(
    pool: &SqlitePool,
    channel_id: &str,
    input: CreateModelInput,
) -> Result<ChannelModel, AppError> {
    let repo = SqlxModelRepo::new(pool.clone());
    create_with(&repo, channel_id, input).await
}

/// 使用连接池列出模型。
pub async fn list(pool: &SqlitePool, channel_id: &str) -> Result<Vec<ChannelModel>, AppError> {
    let repo = SqlxModelRepo::new(pool.clone());
    list_with(&repo, channel_id).await
}

/// 使用连接池更新模型。
pub async fn update(
    pool: &SqlitePool,
    channel_id: &str,
    id: &str,
    input: UpdateModelInput,
) -> Result<ChannelModel, AppError> {
    let repo = SqlxModelRepo::new(pool.clone());
    update_with(&repo, channel_id, id, input).await
}

/// 使用连接池删除模型。
pub async fn delete(pool: &SqlitePool, channel_id: &str, id: &str) -> Result<(), AppError> {
    let repo = SqlxModelRepo::new(pool.clone());
    delete_with(&repo, channel_id, id).await
}

/// 使用注入的仓储创建模型，便于测试。
pub async fn create_with<R: ModelRepo>(
    repo: &R,
    channel_id: &str,
    input: CreateModelInput,
) -> Result<ChannelModel, AppError> {
    ensure_channel_exists(repo, channel_id).await?;
    validate_model_id(&input.model_id)?;

    let model_id = input.model_id.trim().to_string();
    if repo
        .model_id_exists(channel_id, &model_id)
        .await
        .map_err(|error| AppError::internal(format!("failed to check model conflict: {error}")))?
    {
        return Err(AppError::conflict(
            "MODEL_ID_CONFLICT",
            format!("model_id '{model_id}' already exists in this channel"),
        ));
    }

    repo.insert(&NewChannelModel {
        id: new_uuid_v7(),
        channel_id: channel_id.to_string(),
        model_id,
        display_name: normalize_optional_text(input.display_name),
        context_window: input.context_window,
        max_output_tokens: input.max_output_tokens,
    })
    .await
    .map_err(|error| AppError::internal(format!("failed to create model: {error}")))
}

/// 使用注入的仓储列出模型，便于测试。
pub async fn list_with<R: ModelRepo>(
    repo: &R,
    channel_id: &str,
) -> Result<Vec<ChannelModel>, AppError> {
    ensure_channel_exists(repo, channel_id).await?;

    repo.list_by_channel(channel_id)
        .await
        .map_err(|error| AppError::internal(format!("failed to list models: {error}")))
}

/// 使用注入的仓储更新模型，便于测试。
pub async fn update_with<R: ModelRepo>(
    repo: &R,
    channel_id: &str,
    id: &str,
    input: UpdateModelInput,
) -> Result<ChannelModel, AppError> {
    ensure_channel_exists(repo, channel_id).await?;

    repo.update(
        channel_id,
        id,
        &ChannelModelPatch {
            display_name: input.display_name.map(normalize_optional_text),
            context_window: input.context_window,
            max_output_tokens: input.max_output_tokens,
        },
    )
    .await
    .map_err(|error| AppError::internal(format!("failed to update model: {error}")))?
    .ok_or_else(|| AppError::not_found(format!("model '{id}' not found")))
}

/// 使用注入的仓储删除模型，便于测试。
pub async fn delete_with<R: ModelRepo>(
    repo: &R,
    channel_id: &str,
    id: &str,
) -> Result<(), AppError> {
    ensure_channel_exists(repo, channel_id).await?;

    match repo
        .delete(channel_id, id)
        .await
        .map_err(|error| AppError::internal(format!("failed to delete model: {error}")))? {
        true => Ok(()),
        false => Err(AppError::not_found(format!("model '{id}' not found"))),
    }
}

/// 统一校验模型所属渠道是否存在。
async fn ensure_channel_exists<R: ModelRepo>(repo: &R, channel_id: &str) -> Result<(), AppError> {
    if repo
        .channel_exists(channel_id)
        .await
        .map_err(|error| AppError::internal(format!("failed to load channel: {error}")))?
    {
        return Ok(());
    }

    Err(AppError::not_found(format!("channel '{channel_id}' not found")))
}
