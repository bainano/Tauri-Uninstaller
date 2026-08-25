use crate::models::AppEntry;

/// 返回系统中全部已安装应用（注册表 + UWP）
#[tauri::command]
pub fn list_apps() -> Vec<AppEntry> {
    crate::uninstall::enumerate_all()
}
