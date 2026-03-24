//! Agent 管理命令。
//!
//! 该文件只负责把 Tauri IPC 参数解包后转交给 `agent_service`，保持命令层足够薄，
//! 这样单元测试既可以直接测 service，也可以通过 `*_impl` 测命令编排。

use tauri::State;

use crate::{
    error::AppError,
    models::{Agent, CreateAgentInput, UpdateAgentInput},
    services::agent_service,
    state::AppState,
};

/// 列出 Agent 列表。
pub async fn list_agents_impl(
    state: &AppState,
    include_disabled: Option<bool>,
) -> Result<Vec<Agent>, AppError> {
    agent_service::list(&state.db, include_disabled.unwrap_or(true)).await
}

/// Tauri 命令：列出 Agent 列表。
#[tauri::command]
pub async fn list_agents(
    state: State<'_, AppState>,
    include_disabled: Option<bool>,
) -> Result<Vec<Agent>, AppError> {
    list_agents_impl(state.inner(), include_disabled).await
}

/// 获取单个 Agent。
pub async fn get_agent_impl(state: &AppState, id: String) -> Result<Agent, AppError> {
    agent_service::get(&state.db, &id).await
}

/// Tauri 命令：获取单个 Agent。
#[tauri::command]
pub async fn get_agent(state: State<'_, AppState>, id: String) -> Result<Agent, AppError> {
    get_agent_impl(state.inner(), id).await
}

/// 创建 Agent。
pub async fn create_agent_impl(
    state: &AppState,
    input: CreateAgentInput,
) -> Result<Agent, AppError> {
    agent_service::create(&state.db, input).await
}

/// Tauri 命令：创建 Agent。
#[tauri::command]
pub async fn create_agent(
    state: State<'_, AppState>,
    input: CreateAgentInput,
) -> Result<Agent, AppError> {
    create_agent_impl(state.inner(), input).await
}

/// 更新 Agent。
pub async fn update_agent_impl(
    state: &AppState,
    id: String,
    input: UpdateAgentInput,
) -> Result<Agent, AppError> {
    agent_service::update(&state.db, &id, input).await
}

/// Tauri 命令：更新 Agent。
#[tauri::command]
pub async fn update_agent(
    state: State<'_, AppState>,
    id: String,
    input: UpdateAgentInput,
) -> Result<Agent, AppError> {
    update_agent_impl(state.inner(), id, input).await
}

/// 删除 Agent。
pub async fn delete_agent_impl(state: &AppState, id: String) -> Result<(), AppError> {
    agent_service::delete(&state.db, &id).await
}

/// Tauri 命令：删除 Agent。
#[tauri::command]
pub async fn delete_agent(state: State<'_, AppState>, id: String) -> Result<(), AppError> {
    delete_agent_impl(state.inner(), id).await
}
