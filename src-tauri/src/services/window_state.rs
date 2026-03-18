use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{Manager, PhysicalPosition, PhysicalSize, Position, Size, WebviewWindow, WindowEvent};

use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SavedWindowState {
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    maximized: bool,
}

pub fn setup_window_state(app: &tauri::AppHandle) -> Result<()> {
    let Some(window) = app.get_webview_window("main") else {
        return Ok(());
    };

    let state_path = app
        .path()
        .app_data_dir()
        .map_err(|err| crate::error::AppError::Other(format!("failed to get app data dir: {err}")))?
        .join("window-state.json");

    apply_window_state(&window, &state_path)?;

    let state_path_for_events = state_path.clone();
    let window_for_events = window.clone();
    window.on_window_event(move |event| {
        if matches!(
            event,
            WindowEvent::Resized(_) | WindowEvent::Moved(_) | WindowEvent::CloseRequested { .. }
        ) {
            let _ = save_window_state(&window_for_events, &state_path_for_events);
        }
    });

    Ok(())
}

fn apply_window_state(window: &WebviewWindow, state_path: &Path) -> Result<()> {
    if let Ok(saved) = load_window_state(state_path) {
        window.set_size(Size::Physical(PhysicalSize::new(saved.width, saved.height)))?;
        window.set_position(Position::Physical(PhysicalPosition::new(saved.x, saved.y)))?;
        if saved.maximized {
            let _ = window.maximize();
        }
        return Ok(());
    }

    if let Some(monitor) = window.current_monitor()? {
        let monitor_size = monitor.size();
        let width = ((monitor_size.width as f64) * 0.4).round() as u32;
        let height = ((monitor_size.height as f64) * 0.8).round() as u32;
        let width = width.max(1100);
        let height = height.max(720);
        window.set_size(Size::Physical(PhysicalSize::new(width, height)))?;
        window.center()?;
    }

    Ok(())
}

fn load_window_state(path: &Path) -> Result<SavedWindowState> {
    let content = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}

fn save_window_state(window: &WebviewWindow, state_path: &PathBuf) -> Result<()> {
    let size = window.outer_size()?;
    let position = window.outer_position()?;
    let maximized = window.is_maximized()?;

    let state = SavedWindowState {
        width: size.width,
        height: size.height,
        x: position.x,
        y: position.y,
        maximized,
    };

    if let Some(parent) = state_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(state_path, serde_json::to_vec_pretty(&state)?)?;
    Ok(())
}
