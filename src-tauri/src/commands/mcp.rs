use tauri::State;

use crate::app::state::AppState;
use crate::domain::native_capabilities::McpEventDetail;
use crate::support::error::Result;

#[tauri::command]
pub async fn list_mcp_events_by_run(
    state: State<'_, AppState>,
    generation_run_id: Option<String>,
    workflow_run_id: Option<String>,
) -> Result<Vec<McpEventDetail>> {
    crate::services::mcp::list_mcp_events_by_run(
        &state.db,
        state.content_store.as_ref(),
        generation_run_id.as_deref(),
        workflow_run_id.as_deref(),
    )
    .await
}
