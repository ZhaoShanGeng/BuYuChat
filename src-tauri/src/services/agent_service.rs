//! Agent 服务编排逻辑。
//!
//! 该文件负责把 `agents` 表对应的仓储操作整理成稳定的业务接口，供命令层直接调用。
//! 这里集中处理三类问题：
//! 1. 输入规范化：名称去首尾空白，空字符串拒绝；`system_prompt` 允许显式清空。
//! 2. 时间戳与资源 ID：统一在 service 层生成，repo 只负责持久化。
//! 3. 业务错误语义：把底层字符串错误转成 `AppError`，让前端拿到稳定的错误码。
//!
//! 当前 Agent 模块只暴露 CRUD，不包含头像上传等扩展能力。

use sqlx::SqlitePool;

use crate::{
    error::AppError,
    models::{Agent, AgentPatch, CreateAgentInput, NewAgent, UpdateAgentInput},
    repo::agent_repo::{AgentRepo, SqlxAgentRepo},
    services::channel_service::{Clock, SystemClock},
    utils::ids::new_uuid_v7,
};

/// 使用连接池创建 Agent。
pub async fn create(pool: &SqlitePool, input: CreateAgentInput) -> Result<Agent, AppError> {
    let repo = SqlxAgentRepo::new(pool.clone());
    create_with(&repo, &SystemClock, input).await
}

/// 使用连接池列出 Agent。
pub async fn list(pool: &SqlitePool, include_disabled: bool) -> Result<Vec<Agent>, AppError> {
    let repo = SqlxAgentRepo::new(pool.clone());
    list_with(&repo, include_disabled).await
}

/// 使用连接池获取单个 Agent。
pub async fn get(pool: &SqlitePool, id: &str) -> Result<Agent, AppError> {
    let repo = SqlxAgentRepo::new(pool.clone());
    get_with(&repo, id).await
}

/// 使用连接池更新 Agent。
pub async fn update(
    pool: &SqlitePool,
    id: &str,
    input: UpdateAgentInput,
) -> Result<Agent, AppError> {
    let repo = SqlxAgentRepo::new(pool.clone());
    update_with(&repo, &SystemClock, id, input).await
}

/// 使用连接池删除 Agent。
pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), AppError> {
    let repo = SqlxAgentRepo::new(pool.clone());
    delete_with(&repo, id).await
}

/// 使用显式依赖创建 Agent，便于后续做稳定测试。
pub async fn create_with<R: AgentRepo, C: Clock>(
    repo: &R,
    clock: &C,
    input: CreateAgentInput,
) -> Result<Agent, AppError> {
    let name = normalize_required_text("name", &input.name)?;
    let timestamp = clock.now_ms().await;

    repo.insert(&NewAgent {
        id: new_uuid_v7(),
        name,
        system_prompt: normalize_optional_text(input.system_prompt),
        avatar_uri: None,
        enabled: true,
        created_at: timestamp,
        updated_at: timestamp,
    })
    .await
    .map_err(|error| AppError::internal(format!("failed to create agent: {error}")))
}

/// 使用显式依赖列出 Agent。
pub async fn list_with<R: AgentRepo>(
    repo: &R,
    include_disabled: bool,
) -> Result<Vec<Agent>, AppError> {
    repo.list(include_disabled)
        .await
        .map_err(|error| AppError::internal(format!("failed to list agents: {error}")))
}

/// 使用显式依赖获取单个 Agent。
pub async fn get_with<R: AgentRepo>(repo: &R, id: &str) -> Result<Agent, AppError> {
    repo.get(id)
        .await
        .map_err(|error| AppError::internal(format!("failed to get agent: {error}")))?
        .ok_or_else(|| AppError::not_found(format!("agent '{id}' not found")))
}

/// 使用显式依赖更新 Agent。
pub async fn update_with<R: AgentRepo, C: Clock>(
    repo: &R,
    clock: &C,
    id: &str,
    input: UpdateAgentInput,
) -> Result<Agent, AppError> {
    let name = input
        .name
        .as_ref()
        .map(|value| normalize_required_text("name", value))
        .transpose()?;

    repo.update(
        id,
        &AgentPatch {
            name,
            system_prompt: input.system_prompt.map(normalize_optional_text),
            enabled: input.enabled,
            updated_at: clock.now_ms().await,
        },
    )
    .await
    .map_err(|error| AppError::internal(format!("failed to update agent: {error}")))?
    .ok_or_else(|| AppError::not_found(format!("agent '{id}' not found")))
}

/// 使用显式依赖删除 Agent。
pub async fn delete_with<R: AgentRepo>(repo: &R, id: &str) -> Result<(), AppError> {
    match repo
        .delete(id)
        .await
        .map_err(|error| AppError::internal(format!("failed to delete agent: {error}")))?
    {
        true => Ok(()),
        false => Err(AppError::not_found(format!("agent '{id}' not found"))),
    }
}

/// 规范化必填文本字段。
fn normalize_required_text(field_name: &str, value: &str) -> Result<String, AppError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::validation(
            "VALIDATION_ERROR",
            format!("{field_name} must not be empty"),
        ));
    }

    Ok(trimmed.to_string())
}

/// 规范化可选文本字段，空字符串会折叠为 `None`。
fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value.and_then(|text| {
        let trimmed = text.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    })
}
