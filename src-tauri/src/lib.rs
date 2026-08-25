mod commands;
mod models;
mod uninstall;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![commands::list_apps])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
