//! 会话服务编排逻辑。
//!
//! 这个文件负责处理 `conversations` 资源的完整业务约束，并把会话与 Agent/渠道/模型
//! 三种绑定资源的关系校验集中在一起，避免命令层反复拼装规则。
//!
//! 当前实现遵循 MVP 文档约束：
//! - 会话本身只保存一组绑定：`agent_id`、`channel_id`、`channel_model_id`。
//! - service 层显式校验引用资源存在，并要求 Agent/渠道处于启用状态。
//! - `channel_model_id` 必须隶属于当前生效的 `channel_id`，不允许跨渠道悬挂绑定。

use sqlx::SqlitePool;

use crate::{
    error::AppError,
    models::{
        Conversation, ConversationPatch, ConversationSummary, CreateConversationInput,
        UpdateConversationInput,
    },
    repo::{
        agent_repo::{AgentRepo, SqlxAgentRepo},
        channel_repo::{ChannelRepo, SqlxChannelRepo},
        conversation_repo::{ConversationRepo, SqlxConversationRepo},
        model_repo::{ModelRepo, SqlxModelRepo},
    },
    services::channel_service::{Clock, SystemClock},
    utils::ids::new_uuid_v7,
};

use crate::models::NewConversation;

/// 使用连接池创建会话。
pub async fn create(
    pool: &SqlitePool,
    input: CreateConversationInput,
) -> Result<Conversation, AppError> {
    let conversation_repo = SqlxConversationRepo::new(pool.clone());
    let agent_repo = SqlxAgentRepo::new(pool.clone());
    let channel_repo = SqlxChannelRepo::new(pool.clone());
    let model_repo = SqlxModelRepo::new(pool.clone());

    create_with(
        &conversation_repo,
        &agent_repo,
        &channel_repo,
        &model_repo,
        &SystemClock,
        input,
    )
    .await
}

/// 使用连接池列出会话。
pub async fn list(pool: &SqlitePool, archived: bool) -> Result<Vec<ConversationSummary>, AppError> {
    let repo = SqlxConversationRepo::new(pool.clone());
    list_with(&repo, archived).await
}

/// 使用连接池获取单个会话。
pub async fn get(pool: &SqlitePool, id: &str) -> Result<Conversation, AppError> {
    let repo = SqlxConversationRepo::new(pool.clone());
    get_with(&repo, id).await
}

/// 使用连接池更新会话。
pub async fn update(
    pool: &SqlitePool,
    id: &str,
    input: UpdateConversationInput,
) -> Result<Conversation, AppError> {
    let conversation_repo = SqlxConversationRepo::new(pool.clone());
    let agent_repo = SqlxAgentRepo::new(pool.clone());
    let channel_repo = SqlxChannelRepo::new(pool.clone());
    let model_repo = SqlxModelRepo::new(pool.clone());

    update_with(
        &conversation_repo,
        &agent_repo,
        &channel_repo,
        &model_repo,
        &SystemClock,
        id,
        input,
    )
    .await
}

/// 使用连接池删除会话。
pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), AppError> {
    let repo = SqlxConversationRepo::new(pool.clone());
    delete_with(&repo, id).await
}

/// 使用显式依赖创建会话。
pub async fn create_with<CR, AR, CHR, MR, C>(
    conversation_repo: &CR,
    agent_repo: &AR,
    channel_repo: &CHR,
    model_repo: &MR,
    clock: &C,
    input: CreateConversationInput,
) -> Result<Conversation, AppError>
where
    CR: ConversationRepo,
    AR: AgentRepo,
    CHR: ChannelRepo,
    MR: ModelRepo,
    C: Clock,
{
    validate_bindings(
        agent_repo,
        channel_repo,
        model_repo,
        input.agent_id.as_deref(),
        input.channel_id.as_deref(),
        input.channel_model_id.as_deref(),
    )
    .await?;

    let timestamp = clock.now_ms().await;
    conversation_repo
        .insert(&NewConversation {
            id: new_uuid_v7(),
            title: normalize_title(input.title)?,
            agent_id: input.agent_id,
            channel_id: input.channel_id,
            channel_model_id: input.channel_model_id,
            archived: false,
            pinned: false,
            created_at: timestamp,
            updated_at: timestamp,
        })
        .await
        .map_err(|error| AppError::internal(format!("failed to create conversation: {error}")))
}

/// 使用显式依赖列出会话。
pub async fn list_with<CR: ConversationRepo>(
    repo: &CR,
    archived: bool,
) -> Result<Vec<ConversationSummary>, AppError> {
    repo.list(archived)
        .await
        .map_err(|error| AppError::internal(format!("failed to list conversations: {error}")))
}

/// 使用显式依赖获取单个会话。
pub async fn get_with<CR: ConversationRepo>(
    repo: &CR,
    id: &str,
) -> Result<Conversation, AppError> {
    repo.get(id)
        .await
        .map_err(|error| AppError::internal(format!("failed to get conversation: {error}")))?
        .ok_or_else(|| AppError::not_found(format!("conversation '{id}' not found")))
}

/// 使用显式依赖更新会话。
pub async fn update_with<CR, AR, CHR, MR, C>(
    conversation_repo: &CR,
    agent_repo: &AR,
    channel_repo: &CHR,
    model_repo: &MR,
    clock: &C,
    id: &str,
    input: UpdateConversationInput,
) -> Result<Conversation, AppError>
where
    CR: ConversationRepo,
    AR: AgentRepo,
    CHR: ChannelRepo,
    MR: ModelRepo,
    C: Clock,
{
    let current = conversation_repo
        .get(id)
        .await
        .map_err(|error| AppError::internal(format!("failed to load conversation: {error}")))?
        .ok_or_else(|| AppError::not_found(format!("conversation '{id}' not found")))?;

    let next_agent_id = merge_optional_patch(current.agent_id.as_deref(), input.agent_id.as_ref());
    let next_channel_id =
        merge_optional_patch(current.channel_id.as_deref(), input.channel_id.as_ref());
    let next_channel_model_id = merge_optional_patch(
        current.channel_model_id.as_deref(),
        input.channel_model_id.as_ref(),
    );

    validate_bindings(
        agent_repo,
        channel_repo,
        model_repo,
        next_agent_id.as_deref(),
        next_channel_id.as_deref(),
        next_channel_model_id.as_deref(),
    )
    .await?;

    let title = input
        .title
        .as_ref()
        .map(|value| normalize_title(Some(value.clone())))
        .transpose()?;

    conversation_repo
        .update(
            id,
            &ConversationPatch {
                title,
                agent_id: input.agent_id,
                channel_id: input.channel_id,
                channel_model_id: input.channel_model_id,
                archived: input.archived,
                pinned: input.pinned,
                updated_at: clock.now_ms().await,
            },
        )
        .await
        .map_err(|error| AppError::internal(format!("failed to update conversation: {error}")))?
        .ok_or_else(|| AppError::not_found(format!("conversation '{id}' not found")))
}

/// 使用显式依赖删除会话。
pub async fn delete_with<CR: ConversationRepo>(repo: &CR, id: &str) -> Result<(), AppError> {
    match repo
        .delete(id)
        .await
        .map_err(|error| AppError::internal(format!("failed to delete conversation: {error}")))? {
        true => Ok(()),
        false => Err(AppError::not_found(format!("conversation '{id}' not found"))),
    }
}

/// 校验会话绑定资源的存在性与启用状态。
async fn validate_bindings<AR, CHR, MR>(
    agent_repo: &AR,
    channel_repo: &CHR,
    model_repo: &MR,
    agent_id: Option<&str>,
    channel_id: Option<&str>,
    channel_model_id: Option<&str>,
) -> Result<(), AppError>
where
    AR: AgentRepo,
    CHR: ChannelRepo,
    MR: ModelRepo,
{
    if let Some(agent_id) = agent_id {
        let agent = agent_repo
            .get(agent_id)
            .await
            .map_err(|error| AppError::internal(format!("failed to load agent: {error}")))?
            .ok_or_else(|| AppError::not_found(format!("agent '{agent_id}' not found")))?;
        if !agent.enabled {
            return Err(AppError::agent_disabled());
        }
    }

    if let Some(channel_id) = channel_id {
        let channel = channel_repo
            .get(channel_id)
            .await
            .map_err(|error| AppError::internal(format!("failed to load channel: {error}")))?
            .ok_or_else(|| AppError::not_found(format!("channel '{channel_id}' not found")))?;
        if !channel.enabled {
            return Err(AppError::channel_disabled());
        }
    }

    if let Some(channel_model_id) = channel_model_id {
        let channel_id = channel_id.ok_or_else(|| {
            AppError::validation(
                "VALIDATION_ERROR",
                "channel_model_id requires channel_id",
            )
        })?;

        model_repo
            .get_by_channel_and_id(channel_id, channel_model_id)
            .await
            .map_err(|error| AppError::internal(format!("failed to load model: {error}")))?
            .ok_or_else(|| AppError::not_found(format!("model '{channel_model_id}' not found")))?;
    }

    Ok(())
}

/// 规范化会话标题，未提供时回落到默认值。
fn normalize_title(value: Option<String>) -> Result<String, AppError> {
    let raw = value.unwrap_or_else(|| "新会话".to_string());
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(AppError::validation(
            "VALIDATION_ERROR",
            "title must not be empty",
        ));
    }

    Ok(trimmed.to_string())
}

/// 将补丁字段与当前值合并成“更新后的最终值”。
fn merge_optional_patch(current: Option<&str>, patch: Option<&Option<String>>) -> Option<String> {
    match patch {
        Some(Some(value)) => Some(value.clone()),
        Some(None) => None,
        None => current.map(ToOwned::to_owned),
    }
}
