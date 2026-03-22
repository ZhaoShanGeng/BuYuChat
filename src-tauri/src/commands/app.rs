use tauri::{AppHandle, Manager};

use crate::support::error::Result;

#[tauri::command]
pub fn notify_main_window_ready(app: AppHandle) -> Result<()> {
    let Some(window) = app.get_webview_window("main") else {
        return Ok(());
    };

    if !window.is_visible()? {
        window.show()?;
    }

    Ok(())
}
