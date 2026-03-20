use tauri::State;

use crate::app::state::AppState;
use crate::domain::native_capabilities::ToolInvocationDetail;
use crate::support::error::Result;

#[tauri::command]
pub async fn list_tool_invocations_by_run(
    state: State<'_, AppState>,
    generation_run_id: Option<String>,
    workflow_run_id: Option<String>,
) -> Result<Vec<ToolInvocationDetail>> {
    crate::services::tool_invocations::list_tool_invocations_by_run(
        &state.db,
        state.content_store.as_ref(),
        generation_run_id.as_deref(),
        workflow_run_id.as_deref(),
    )
    .await
}
