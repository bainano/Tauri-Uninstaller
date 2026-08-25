// Tauri Uninstaller 后端入口
// 暴露 Tauri commands 供前端调用

mod icon;
mod uninstaller;

use base64::Engine;
use uninstaller::InstalledApp;

/// 扫描系统已安装软件列表
#[tauri::command]
fn scan_installed_apps() -> Vec<InstalledApp> {
    uninstaller::scan_installed_apps()
}

/// 提取文件关联图标，返回 base64 编码的 PNG data URL
#[tauri::command]
fn extract_app_icon(path: String) -> Result<String, String> {
    let png = icon::extract_file_icon(&path)?;
    Ok(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(png)
    ))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            scan_installed_apps,
            extract_app_icon
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
