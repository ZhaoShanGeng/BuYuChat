//! 步语桌面应用的后端库入口。

use tauri::Manager;

pub mod ai;
pub mod channel_types;
pub mod commands;
pub mod error;
pub mod mcp;
pub mod models;
pub mod repo;
pub mod services;
pub mod state;
pub mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default();

    #[cfg(desktop)]
    let builder = builder
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|app| {
            let state = tauri::async_runtime::block_on(state::AppState::initialize_for_app(app))
                .expect("初始化应用状态失败");
            app.manage(state);
            setup_initial_window(app)?;
            Ok(())
        });

    #[cfg(not(desktop))]
    let builder = builder
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let state = tauri::async_runtime::block_on(state::AppState::initialize_for_app(app))
                .expect("初始化应用状态失败");
            app.manage(state);
            Ok(())
        });

    builder
        .invoke_handler(tauri::generate_handler![
            commands::agents::list_agents,
            commands::agents::get_agent,
            commands::agents::create_agent,
            commands::agents::update_agent,
            commands::agents::delete_agent,
            commands::channels::list_channels,
            commands::channels::get_channel,
            commands::channels::create_channel,
            commands::channels::update_channel,
            commands::channels::delete_channel,
            commands::channels::test_channel,
            commands::conversations::list_conversations,
            commands::conversations::get_conversation,
            commands::conversations::create_conversation,
            commands::conversations::update_conversation,
            commands::conversations::delete_conversation,
            commands::messages::list_messages,
            commands::messages::get_version_content,
            commands::messages::set_active_version,
            commands::messages::delete_version,
            commands::messages::send_message,
            commands::messages::reroll,
            commands::messages::edit_message,
            commands::messages::cancel_generation,
            commands::models::list_models,
            commands::models::create_model,
            commands::models::update_model,
            commands::models::delete_model,
            commands::models::fetch_remote_models,
            commands::tools::list_builtin_tools
        ])
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用失败");
}

/// 首次启动时计算窗口的初始尺寸和居中位置。
///
/// 规则：上下共保留 10%（高 = 屏幕 90%），左右共保留 25%（宽 = 屏幕 75%）。
/// `tauri-plugin-window-state` 有记忆数据时会覆盖此设置。
#[cfg(desktop)]
fn setup_initial_window(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::Manager;

    let window = match app.get_webview_window("main") {
        Some(w) => w,
        None => return Ok(()),
    };

    // 获取主显示器尺寸
    let monitor = match window.current_monitor() {
        Ok(Some(m)) => m,
        _ => {
            window.show()?;
            return Ok(());
        }
    };

    let screen = monitor.size();
    let scale = monitor.scale_factor();
    let sw = screen.width as f64 / scale;
    let sh = screen.height as f64 / scale;

    let win_w = (sw * 0.75).round().max(960.0);
    let win_h = (sh * 0.90).round().max(640.0);
    let pos_x = ((sw - win_w) / 2.0).round().max(0.0);
    let pos_y = ((sh - win_h) / 2.0).round().max(0.0);

    use tauri::{LogicalPosition, LogicalSize};
    window.set_size(LogicalSize::new(win_w, win_h))?;
    window.set_position(LogicalPosition::new(pos_x, pos_y))?;
    window.show()?;

    Ok(())
}
