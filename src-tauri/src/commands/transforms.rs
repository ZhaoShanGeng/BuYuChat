use tauri::{AppHandle, State};

use crate::app::state::AppState;
use crate::commands::incremental;
use crate::domain::transforms::{
    ApplyTransformsInput, ApplyTransformsResult, CreateTransformPipelineInput, TransformBinding,
    TransformBindingInput, TransformPipeline, TransformStep, TransformStepInput,
    UpdateTransformPipelineInput,
};
use crate::support::error::Result;

#[tauri::command]
pub async fn list_transform_pipelines(
    state: State<'_, AppState>,
) -> Result<Vec<TransformPipeline>> {
    crate::services::transforms::list_transform_pipelines(&state.db, &state.content_store).await
}

#[tauri::command]
pub async fn get_transform_pipeline(
    state: State<'_, AppState>,
    id: String,
) -> Result<TransformPipeline> {
    crate::services::transforms::get_transform_pipeline(&state.db, &state.content_store, &id).await
}

#[tauri::command]
pub async fn create_transform_pipeline(
    app: AppHandle,
    state: State<'_, AppState>,
    input: CreateTransformPipelineInput,
) -> Result<TransformPipeline> {
    let pipeline = crate::services::transforms::create_transform_pipeline(
        &state.db,
        &state.content_store,
        &input,
    )
    .await?;
    incremental::emit_upsert(
        &app,
        "global",
        None,
        "transform_pipeline",
        Some(&pipeline.id),
        &pipeline,
    )?;
    Ok(pipeline)
}

#[tauri::command]
pub async fn update_transform_pipeline(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    input: UpdateTransformPipelineInput,
) -> Result<TransformPipeline> {
    let pipeline = crate::services::transforms::update_transform_pipeline(
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
        "transform_pipeline",
        Some(&pipeline.id),
        &pipeline,
    )?;
    Ok(pipeline)
}

#[tauri::command]
pub async fn delete_transform_pipeline(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<()> {
    crate::services::transforms::delete_transform_pipeline(&state.db, &id).await?;
    incremental::emit_delete(&app, "global", None, "transform_pipeline", &id)?;
    Ok(())
}

#[tauri::command]
pub async fn list_transform_steps(
    state: State<'_, AppState>,
    pipeline_id: String,
) -> Result<Vec<TransformStep>> {
    crate::services::transforms::list_transform_steps(&state.db, &pipeline_id).await
}

#[tauri::command]
pub async fn replace_transform_steps(
    app: AppHandle,
    state: State<'_, AppState>,
    pipeline_id: String,
    items: Vec<TransformStepInput>,
) -> Result<Vec<TransformStep>> {
    let steps =
        crate::services::transforms::replace_transform_steps(&state.db, &pipeline_id, &items)
            .await?;
    incremental::emit_replace(
        &app,
        "transform_pipeline",
        Some(&pipeline_id),
        "transform_steps",
        &steps,
    )?;
    Ok(steps)
}

#[tauri::command]
pub async fn list_transform_bindings(state: State<'_, AppState>) -> Result<Vec<TransformBinding>> {
    crate::services::transforms::list_transform_bindings(&state.db).await
}

#[tauri::command]
pub async fn create_transform_binding(
    app: AppHandle,
    state: State<'_, AppState>,
    input: TransformBindingInput,
) -> Result<TransformBinding> {
    let binding = crate::services::transforms::create_transform_binding(&state.db, &input).await?;
    incremental::emit_upsert(
        &app,
        "global",
        None,
        "transform_binding",
        Some(&binding.id),
        &binding,
    )?;
    Ok(binding)
}

#[tauri::command]
pub async fn update_transform_binding(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    input: TransformBindingInput,
) -> Result<TransformBinding> {
    let binding =
        crate::services::transforms::update_transform_binding(&state.db, &id, &input).await?;
    incremental::emit_upsert(
        &app,
        "global",
        None,
        "transform_binding",
        Some(&binding.id),
        &binding,
    )?;
    Ok(binding)
}

#[tauri::command]
pub async fn delete_transform_binding(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<()> {
    crate::services::transforms::delete_transform_binding(&state.db, &id).await?;
    incremental::emit_delete(&app, "global", None, "transform_binding", &id)?;
    Ok(())
}

#[tauri::command]
pub async fn apply_transforms(
    state: State<'_, AppState>,
    input: ApplyTransformsInput,
) -> Result<ApplyTransformsResult> {
    crate::services::transforms::apply_transforms(&state.db, &state.content_store, &input).await
}
