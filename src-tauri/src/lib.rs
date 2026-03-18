#[cfg(test)]
mod branch_tests;
mod commands;
pub mod db;
pub mod error;
pub mod providers;
pub mod services;
#[cfg(test)]
mod smoke_tests;
mod state;
pub mod types;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    services::logging::init_logging();
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();
            let state = tauri::async_runtime::block_on(async move {
                let app_data_dir = app_handle.path().app_data_dir().map_err(|err| {
                    crate::error::AppError::Other(format!(
                        "failed to resolve app data directory: {err}"
                    ))
                })?;

                std::fs::create_dir_all(&app_data_dir)?;

                let db_path = app_data_dir.join("omnichat.db");
                let db = db::init_pool(&db_path).await?;
                let providers = providers::ProviderRegistry::new_shared();
                let keyring = services::keyring::KeyringService::new("omnichat");

                providers::init_enabled_providers(&providers, &db, &keyring).await?;

                Ok::<state::AppState, crate::error::AppError>(state::AppState::new(
                    db, providers, keyring,
                ))
            })
            .map_err(|err| -> Box<dyn std::error::Error> { Box::new(err) })?;

            app.manage(state);
            services::window_state::setup_window_state(&app.handle().clone())
                .map_err(|err| -> Box<dyn std::error::Error> { Box::new(err) })?;
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::conversation::list_conversations,
            commands::conversation::get_conversation,
            commands::conversation::create_conversation,
            commands::conversation::update_conversation_title,
            commands::conversation::update_conversation_model,
            commands::conversation::update_conversation_system_prompt,
            commands::conversation::delete_conversation,
            commands::conversation::clear_conversation_messages,
            commands::conversation::fork_conversation_from_message,
            commands::message::list_messages,
            commands::message::list_message_bundle,
            commands::message::get_message_versions,
            commands::message::switch_message_version,
            commands::message::delete_message,
            commands::provider::list_provider_configs,
            commands::provider::list_custom_channels,
            commands::provider::get_provider_api_key,
            commands::provider::get_custom_channel_api_key,
            commands::provider::save_provider_config,
            commands::provider::create_custom_channel,
            commands::provider::update_custom_channel,
            commands::provider::delete_custom_channel,
            commands::provider::test_provider_connection,
            commands::provider::list_models,
            commands::provider::refresh_custom_channel_models,
            commands::provider::save_custom_channel_models,
            commands::provider::save_api_key,
            commands::provider::delete_api_key,
            commands::chat::send_message,
            commands::chat::regenerate_message,
            commands::chat::save_message_edit,
            commands::chat::edit_user_message,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run tauri application");
}
