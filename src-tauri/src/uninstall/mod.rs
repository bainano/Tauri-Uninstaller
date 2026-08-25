pub mod registry;
pub mod residue;
pub mod runner;
pub mod uwp;

use crate::models::AppEntry;
use residue::{ResidueItem, ResidueReport, RemoveResult};
use runner::{parse_uninstall_string, spawn_uninstaller, wait_with_timeout, UninstallOutcome};
use std::time::Duration;

/// 枚举全部应用（注册表 + UWP）
pub fn enumerate_all() -> Vec<AppEntry> {
    let mut apps = registry::enumerate();
    apps.extend(uwp::enumerate());
    apps
}

/// 执行完美卸载：调用原始卸载器（静默优先），等待完成或超时
#[tauri::command]
pub fn uninstall_app(
    key: String,
    silent: Option<bool>,
    timeout_secs: Option<u64>,
) -> Result<UninstallOutcome, String> {
    let app = find_app_by_key(&key).ok_or_else(|| format!("未找到应用: {}", key))?;
    let uninst = app
        .uninstall_string
        .clone()
        .or(app.fallback_uninstall_string.clone())
        .ok_or_else(|| "该应用没有可用的卸载命令".to_string())?;

    let parsed = parse_uninstall_string(&uninst);
    if parsed.program.is_empty() {
        return Err(format!("无法解析卸载命令: {}", uninst));
    }

    let silent = silent.unwrap_or(true);
    let timeout = Duration::from_secs(timeout_secs.unwrap_or(if silent { 60 } else { 180 }));

    let mut child = spawn_uninstaller(&parsed, silent)?;
    let pid = child.id();

    // 静默模式：部分卸载器立即退出（子进程接管），给 5s 观察窗口
    let mut actual_timeout = timeout;
    if silent {
        std::thread::sleep(Duration::from_secs(5));
        if let Ok(Some(_)) = child.try_wait() {
            actual_timeout = Duration::from_secs(5);
        }
    }

    let (timed_out, exit_code) = wait_with_timeout(&mut child, actual_timeout)?;
    let waited = if timed_out {
        actual_timeout.as_secs()
    } else {
        actual_timeout.as_secs()
    };

    if timed_out {
        Ok(UninstallOutcome {
            status: "timed_out".into(),
            pid: Some(pid),
            exit_code: None,
            message: format!("卸载器运行超过 {} 秒仍未结束，可能弹出了交互窗口或已挂起", actual_timeout.as_secs()),
            waited_secs: waited,
        })
    } else {
        match exit_code {
            Some(0) => Ok(UninstallOutcome {
                status: "finished".into(),
                pid: Some(pid),
                exit_code: Some(0),
                message: "卸载命令已执行完成".into(),
                waited_secs: waited,
            }),
            Some(code) => Ok(UninstallOutcome {
                status: "failed".into(),
                pid: Some(pid),
                exit_code: Some(code),
                message: format!("卸载器退出，退出码 {}", code),
                waited_secs: waited,
            }),
            None => Ok(UninstallOutcome {
                status: "finished".into(),
                pid: Some(pid),
                exit_code: None,
                message: "卸载命令已执行完成（无退出码）".into(),
                waited_secs: waited,
            }),
        }
    }
}

/// 强制终止卸载进程树
#[tauri::command]
pub fn kill_uninstall_process(pid: u32) -> Result<(), String> {
    runner::kill_process_tree(pid)
}

/// 扫描残留（文件 / 快捷方式 / 注册表）
#[tauri::command]
pub fn scan_residue(key: String) -> Result<ResidueReport, String> {
    let app = find_app_by_key(&key).ok_or_else(|| format!("未找到应用: {}", key))?;
    Ok(residue::scan_residue(&app))
}

/// 删除残留项（文件进回收站，注册表直接删除）
#[tauri::command]
pub fn remove_residue(items: Vec<ResidueItem>) -> Result<RemoveResult, String> {
    Ok(residue::remove_residue(&items))
}

/// 强制卸载：删除注册表卸载项 + 安装目录（进回收站）
#[tauri::command]
pub fn force_uninstall(key: String, remove_files: bool) -> Result<RemoveResult, String> {
    let app = find_app_by_key(&key).ok_or_else(|| format!("未找到应用: {}", key))?;
    let mut items: Vec<ResidueItem> = Vec::new();
    if let Some(rk) = &app.registry_key {
        if !rk.is_empty() {
            items.push(ResidueItem {
                kind: "registry".into(),
                path: rk.clone(),
                name: rk.rsplit('\\').next().unwrap_or("").to_string(),
                size_bytes: None,
                safe: false,
            });
        }
    }
    if remove_files {
        if let Some(il) = &app.install_location {
            if !il.is_empty() {
                items.push(ResidueItem {
                    kind: "file".into(),
                    path: il.clone(),
                    name: il.rsplit('\\').next().unwrap_or("").to_string(),
                    size_bytes: None,
                    safe: true,
                });
            }
        }
    }
    Ok(residue::remove_residue(&items))
}

/// 当前进程是否以管理员权限运行
#[tauri::command]
pub fn is_elevated() -> bool {
    use windows_sys::Win32::Foundation::*;
    use windows_sys::Win32::Security::*;
    use windows_sys::Win32::System::Threading::*;

    unsafe {
        let mut token: HANDLE = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return false;
        }
        let mut elevation: TOKEN_ELEVATION = std::mem::zeroed();
        let mut size: u32 = 0;
        let r = GetTokenInformation(
            token,
            TokenElevation,
            &mut elevation as *mut _ as *mut _,
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut size,
        );
        CloseHandle(token);
        r != 0 && elevation.TokenIsElevated != 0
    }
}

/// 按注册表键路径查找应用（从全部枚举结果中）
fn find_app_by_key(key: &str) -> Option<AppEntry> {
    enumerate_all()
        .into_iter()
        .find(|a| a.registry_key.as_deref() == Some(key))
}
