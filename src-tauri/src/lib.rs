pub mod channel_types;
pub mod commands;
pub mod db;
pub mod errors;
pub mod models;
pub mod repo;
pub mod services;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db = db::init_db().expect("failed to initialize sqlite database");

    tauri::Builder::default()
        .manage(db)
        .invoke_handler(tauri::generate_handler![
            commands::channels::list_channels,
            commands::channels::get_channel,
            commands::channels::create_channel,
            commands::channels::update_channel,
            commands::channels::delete_channel,
            commands::channels::test_channel
        ])
        .run(tauri::generate_context!())
        .expect("failed to run tauri application");
}
