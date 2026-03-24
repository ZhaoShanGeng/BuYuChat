//! 步语桌面应用的后端库入口。

pub mod ai;
pub mod channel_types;
pub mod commands;
pub mod error;
pub mod models;
pub mod repo;
pub mod services;
pub mod state;
pub mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state =
        tauri::async_runtime::block_on(state::AppState::initialize()).expect("初始化应用状态失败");

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::channels::list_channels,
            commands::channels::get_channel,
            commands::channels::create_channel,
            commands::channels::update_channel,
            commands::channels::delete_channel,
            commands::channels::test_channel,
            commands::models::list_models,
            commands::models::create_model,
            commands::models::update_model,
            commands::models::delete_model,
            commands::models::fetch_remote_models
        ])
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用失败");
}
