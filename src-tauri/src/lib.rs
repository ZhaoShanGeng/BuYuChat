#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {name}! Rust + Tauri is ready.")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("failed to run tauri application");
}
