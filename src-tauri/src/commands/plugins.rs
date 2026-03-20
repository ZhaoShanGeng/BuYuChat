use tauri::{AppHandle, State};

use crate::app::state::AppState;
use crate::commands::incremental;
use crate::domain::plugins::{CreatePluginInput, PluginDef, UpdatePluginInput};
use crate::support::error::Result;

#[tauri::command]
pub async fn list_plugins(state: State<'_, AppState>) -> Result<Vec<PluginDef>> {
    crate::services::plugins::list_plugins(&state.db).await
}

#[tauri::command]
pub async fn get_plugin(state: State<'_, AppState>, id: String) -> Result<PluginDef> {
    crate::services::plugins::get_plugin(&state.db, &id).await
}

#[tauri::command]
pub async fn create_plugin(
    app: AppHandle,
    state: State<'_, AppState>,
    input: CreatePluginInput,
) -> Result<PluginDef> {
    let plugin =
        crate::services::plugins::create_plugin(&state.db, state.plugin_runtime.as_ref(), &input)
            .await?;
    incremental::emit_upsert(&app, "global", None, "plugin", Some(&plugin.id), &plugin)?;
    Ok(plugin)
}

#[tauri::command]
pub async fn update_plugin(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    input: UpdatePluginInput,
) -> Result<PluginDef> {
    let plugin = crate::services::plugins::update_plugin(
        &state.db,
        state.plugin_runtime.as_ref(),
        &id,
        &input,
    )
    .await?;
    incremental::emit_upsert(&app, "global", None, "plugin", Some(&plugin.id), &plugin)?;
    Ok(plugin)
}

#[tauri::command]
pub async fn delete_plugin(app: AppHandle, state: State<'_, AppState>, id: String) -> Result<()> {
    crate::services::plugins::delete_plugin(&state.db, state.plugin_runtime.as_ref(), &id).await?;
    incremental::emit_delete(&app, "global", None, "plugin", &id)?;
    Ok(())
}
