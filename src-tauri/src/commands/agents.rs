use tauri::{AppHandle, State};

use crate::app::state::AppState;
use crate::commands::incremental;
use crate::domain::agents::{
    AddAgentMediaInput, AgentChannelBindingInput, AgentDetail, AgentGreetingDetail,
    AgentMediaDetail, AgentResourceBindingInput, AgentSummary, CreateAgentGreetingInput,
    CreateAgentInput, UpdateAgentGreetingInput, UpdateAgentInput,
};
use crate::domain::common::{ChannelBindingDetail, ResourceBindingDetail};
use crate::support::error::Result;

#[tauri::command]
pub async fn list_agents(state: State<'_, AppState>) -> Result<Vec<AgentSummary>> {
    crate::services::agents::list_agents(&state.db).await
}

#[tauri::command]
pub async fn get_agent_detail(state: State<'_, AppState>, id: String) -> Result<AgentDetail> {
    crate::services::agents::get_agent_detail(&state.db, &state.content_store, &id).await
}

#[tauri::command]
pub async fn create_agent(
    app: AppHandle,
    state: State<'_, AppState>,
    input: CreateAgentInput,
) -> Result<AgentDetail> {
    let agent =
        crate::services::agents::create_agent(&state.db, &state.content_store, &input).await?;
    incremental::emit_upsert(
        &app,
        "global",
        None,
        "agent",
        Some(&agent.summary.id),
        &agent,
    )?;
    Ok(agent)
}

#[tauri::command]
pub async fn update_agent(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    input: UpdateAgentInput,
) -> Result<AgentDetail> {
    let agent =
        crate::services::agents::update_agent(&state.db, &state.content_store, &id, &input).await?;
    incremental::emit_upsert(
        &app,
        "global",
        None,
        "agent",
        Some(&agent.summary.id),
        &agent,
    )?;
    Ok(agent)
}

#[tauri::command]
pub async fn delete_agent(app: AppHandle, state: State<'_, AppState>, id: String) -> Result<()> {
    crate::services::agents::delete_agent(&state.db, &id).await?;
    incremental::emit_delete(&app, "global", None, "agent", &id)?;
    Ok(())
}

#[tauri::command]
pub async fn create_agent_greeting(
    app: AppHandle,
    state: State<'_, AppState>,
    agent_id: String,
    input: CreateAgentGreetingInput,
) -> Result<AgentGreetingDetail> {
    let greeting = crate::services::agents::create_greeting(
        &state.db,
        &state.content_store,
        &agent_id,
        &input,
    )
    .await?;
    incremental::emit_upsert(
        &app,
        "agent",
        Some(&agent_id),
        "agent_greeting",
        Some(&greeting.id),
        &greeting,
    )?;
    Ok(greeting)
}

#[tauri::command]
pub async fn update_agent_greeting(
    app: AppHandle,
    state: State<'_, AppState>,
    greeting_id: String,
    input: UpdateAgentGreetingInput,
) -> Result<AgentGreetingDetail> {
    let greeting = crate::services::agents::update_greeting(
        &state.db,
        &state.content_store,
        &greeting_id,
        &input,
    )
    .await?;
    incremental::emit_upsert(
        &app,
        "agent",
        Some(&greeting.agent_id),
        "agent_greeting",
        Some(&greeting.id),
        &greeting,
    )?;
    Ok(greeting)
}

#[tauri::command]
pub async fn delete_agent_greeting(
    app: AppHandle,
    state: State<'_, AppState>,
    greeting_id: String,
) -> Result<()> {
    crate::services::agents::delete_greeting(&state.db, &greeting_id).await?;
    incremental::emit_delete(&app, "agent_greeting", None, "agent_greeting", &greeting_id)?;
    Ok(())
}

#[tauri::command]
pub async fn add_agent_media(
    app: AppHandle,
    state: State<'_, AppState>,
    agent_id: String,
    input: AddAgentMediaInput,
) -> Result<AgentMediaDetail> {
    let media =
        crate::services::agents::add_media(&state.db, &state.content_store, &agent_id, &input)
            .await?;
    incremental::emit_upsert(
        &app,
        "agent",
        Some(&agent_id),
        "agent_media",
        Some(&media.id),
        &media,
    )?;
    Ok(media)
}

#[tauri::command]
pub async fn remove_agent_media(
    app: AppHandle,
    state: State<'_, AppState>,
    media_id: String,
) -> Result<()> {
    crate::services::agents::remove_media(&state.db, &media_id).await?;
    incremental::emit_delete(&app, "agent_media", None, "agent_media", &media_id)?;
    Ok(())
}

#[tauri::command]
pub async fn replace_agent_presets(
    app: AppHandle,
    state: State<'_, AppState>,
    agent_id: String,
    items: Vec<AgentResourceBindingInput>,
) -> Result<Vec<ResourceBindingDetail>> {
    crate::services::agents::replace_default_presets(&state.db, &agent_id, &items).await?;
    let detail =
        crate::services::agents::get_agent_detail(&state.db, &state.content_store, &agent_id)
            .await?;
    incremental::emit_replace(
        &app,
        "agent",
        Some(&agent_id),
        "agent_preset_bindings",
        &detail.preset_bindings,
    )?;
    Ok(detail.preset_bindings)
}

#[tauri::command]
pub async fn replace_agent_lorebooks(
    app: AppHandle,
    state: State<'_, AppState>,
    agent_id: String,
    items: Vec<AgentResourceBindingInput>,
) -> Result<Vec<ResourceBindingDetail>> {
    crate::services::agents::replace_default_lorebooks(&state.db, &agent_id, &items).await?;
    let detail =
        crate::services::agents::get_agent_detail(&state.db, &state.content_store, &agent_id)
            .await?;
    incremental::emit_replace(
        &app,
        "agent",
        Some(&agent_id),
        "agent_lorebook_bindings",
        &detail.lorebook_bindings,
    )?;
    Ok(detail.lorebook_bindings)
}

#[tauri::command]
pub async fn replace_agent_user_profiles(
    app: AppHandle,
    state: State<'_, AppState>,
    agent_id: String,
    items: Vec<AgentResourceBindingInput>,
) -> Result<Vec<ResourceBindingDetail>> {
    crate::services::agents::replace_default_user_profiles(&state.db, &agent_id, &items).await?;
    let detail =
        crate::services::agents::get_agent_detail(&state.db, &state.content_store, &agent_id)
            .await?;
    incremental::emit_replace(
        &app,
        "agent",
        Some(&agent_id),
        "agent_user_profile_bindings",
        &detail.user_profile_bindings,
    )?;
    Ok(detail.user_profile_bindings)
}

#[tauri::command]
pub async fn replace_agent_channels(
    app: AppHandle,
    state: State<'_, AppState>,
    agent_id: String,
    items: Vec<AgentChannelBindingInput>,
) -> Result<Vec<ChannelBindingDetail>> {
    crate::services::agents::replace_default_channels(&state.db, &agent_id, &items).await?;
    let detail =
        crate::services::agents::get_agent_detail(&state.db, &state.content_store, &agent_id)
            .await?;
    incremental::emit_replace(
        &app,
        "agent",
        Some(&agent_id),
        "agent_channel_bindings",
        &detail.channel_bindings,
    )?;
    Ok(detail.channel_bindings)
}
