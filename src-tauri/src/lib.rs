// Tauri Uninstaller 后端入口
// 暴露 Tauri commands 供前端调用

mod uninstaller;

use uninstaller::InstalledApp;

/// 扫描系统已安装软件列表
#[tauri::command]
fn scan_installed_apps() -> Vec<InstalledApp> {
    uninstaller::scan_installed_apps()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![scan_installed_apps])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
