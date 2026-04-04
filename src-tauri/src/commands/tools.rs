//! 内置工具相关命令。

use tauri::State;

use crate::{error::AppError, mcp::tools::BuiltinToolInfo, state::AppState};

#[tauri::command]
pub async fn list_builtin_tools(
    state: State<'_, AppState>,
) -> Result<Vec<BuiltinToolInfo>, AppError> {
    Ok(state.tool_registry.list_tools())
}
