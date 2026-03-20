use tauri::{AppHandle, State};

use crate::app::state::AppState;
use crate::commands::incremental;
use crate::domain::user_profiles::{
    CreateUserProfileInput, UpdateUserProfileInput, UserProfileDetail, UserProfileSummary,
};
use crate::support::error::Result;

#[tauri::command]
pub async fn list_user_profiles(state: State<'_, AppState>) -> Result<Vec<UserProfileSummary>> {
    crate::services::user_profiles::list_user_profiles(&state.db).await
}

#[tauri::command]
pub async fn get_user_profile(state: State<'_, AppState>, id: String) -> Result<UserProfileDetail> {
    crate::services::user_profiles::get_user_profile(&state.db, &state.content_store, &id).await
}

#[tauri::command]
pub async fn create_user_profile(
    app: AppHandle,
    state: State<'_, AppState>,
    input: CreateUserProfileInput,
) -> Result<UserProfileDetail> {
    let profile = crate::services::user_profiles::create_user_profile(
        &state.db,
        &state.content_store,
        &input,
    )
    .await?;
    incremental::emit_upsert(
        &app,
        "global",
        None,
        "user_profile",
        Some(&profile.summary.id),
        &profile,
    )?;
    Ok(profile)
}

#[tauri::command]
pub async fn update_user_profile(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    input: UpdateUserProfileInput,
) -> Result<UserProfileDetail> {
    let profile = crate::services::user_profiles::update_user_profile(
        &state.db,
        &state.content_store,
        &id,
        &input,
    )
    .await?;
    incremental::emit_upsert(
        &app,
        "global",
        None,
        "user_profile",
        Some(&profile.summary.id),
        &profile,
    )?;
    Ok(profile)
}

#[tauri::command]
pub async fn delete_user_profile(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<()> {
    crate::services::user_profiles::delete_user_profile(&state.db, &id).await?;
    incremental::emit_delete(&app, "global", None, "user_profile", &id)?;
    Ok(())
}
