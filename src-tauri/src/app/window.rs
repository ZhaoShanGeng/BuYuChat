use tauri::{AppHandle, LogicalSize, Manager};

use crate::support::error::Result;

const MIN_WINDOW_WIDTH: f64 = 920.0;
const MIN_WINDOW_HEIGHT: f64 = 640.0;
const MAX_WINDOW_WIDTH: f64 = 1720.0;
const MAX_WINDOW_HEIGHT: f64 = 1200.0;

pub fn apply_responsive_window_defaults(app: &AppHandle) -> Result<()> {
    let Some(window) = app.get_webview_window("main") else {
        return Ok(());
    };

    let monitor = window
        .current_monitor()?
        .or_else(|| window.available_monitors().ok().and_then(|mut monitors| monitors.drain(..).next()));

    let Some(monitor) = monitor else {
        return Ok(());
    };

    let work_area = monitor.work_area();
    let scale_factor = monitor.scale_factor().max(1.0);
    let available_width = work_area.size.width as f64 / scale_factor;
    let available_height = work_area.size.height as f64 / scale_factor;

    let width_ratio = if available_width <= 1366.0 {
        0.94
    } else if available_width <= 1600.0 {
        0.9
    } else {
        0.84
    };

    let height_ratio = if available_height <= 820.0 {
        0.94
    } else if available_height <= 980.0 {
        0.9
    } else {
        0.86
    };

    let target_width = (available_width * width_ratio)
        .clamp(MIN_WINDOW_WIDTH, MAX_WINDOW_WIDTH)
        .min(available_width);
    let target_height = (available_height * height_ratio)
        .clamp(MIN_WINDOW_HEIGHT, MAX_WINDOW_HEIGHT)
        .min(available_height);

    window.set_size(LogicalSize::new(target_width, target_height))?;
    window.center()?;

    Ok(())
}
