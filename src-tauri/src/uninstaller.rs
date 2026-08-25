//! 已安装软件扫描模块
//!
//! 数据来源：Windows 注册表的标准卸载信息键
//! - HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall
//! - HKLM\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall（32 位应用）
//! - HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall（当前用户应用）

use serde::Serialize;
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ};
use winreg::RegKey;

/// 一条已安装软件记录
#[derive(Debug, Clone, Serialize)]
pub struct InstalledApp {
    /// 软件显示名称
    pub name: String,
    /// 版本号
    pub version: Option<String>,
    /// 发布者
    pub publisher: Option<String>,
    /// 安装位置
    pub install_location: Option<String>,
    /// 卸载命令（标准）
    pub uninstall_string: Option<String>,
    /// 静默卸载命令（优先使用）
    pub quiet_uninstall_string: Option<String>,
    /// 估算占用空间（MB）
    pub estimated_size_mb: Option<u64>,
    /// 显示图标路径
    pub icon_path: Option<String>,
    /// 注册表键完整路径（用于定位卸载命令）
    pub registry_path: String,
    /// 是否系统级安装（HKLM）
    pub system_wide: bool,
}

/// 扫描的注册表卸载信息键
const UNINSTALL_SUBKEYS: [&str; 3] = [
    r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
    r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
    r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\WOW6432Node", // 兼容性兜底
];

/// 展开字符串中的环境变量（如 %ProgramFiles% → C:\Program Files）
fn expand_env(s: &str) -> String {
    let mut result = s.to_string();
    for (key, value) in std::env::vars() {
        let pattern = format!("%{}%", key);
        result = result.replace(&pattern, &value);
    }
    result
}

/// 扫描全部已安装软件，按名称排序返回
pub fn scan_installed_apps() -> Vec<InstalledApp> {
    let mut apps: Vec<InstalledApp> = Vec::new();

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    // HKLM：系统级安装（含 32/64 位）
    for sub in &UNINSTALL_SUBKEYS[..2] {
        if let Ok(key) = hklm.open_subkey_with_flags(sub, KEY_READ) {
            collect_from_key(&key, &mut apps, true, &format!(r"HKLM\{}", sub));
        }
    }

    // HKCU：当前用户级安装
    let hkcu_sub = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall";
    if let Ok(key) = hkcu.open_subkey_with_flags(hkcu_sub, KEY_READ) {
        collect_from_key(&key, &mut apps, false, &format!(r"HKCU\{}", hkcu_sub));
    }

    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    apps
}

/// 枚举单个卸载信息键下的所有子键，收集有效软件记录
fn collect_from_key(key: &RegKey, apps: &mut Vec<InstalledApp>, system_wide: bool, parent_path: &str) {
    for subkey_name in key.enum_keys().flatten() {
        let Ok(subkey) = key.open_subkey_with_flags(&subkey_name, KEY_READ) else {
            continue;
        };

        // 跳过系统组件（SystemComponent=1 表示系统自带组件，不可卸载）
        if let Ok(v) = subkey.get_value::<u32, _>("SystemComponent") {
            if v == 1 {
                continue;
            }
        }
        // 跳过发布者更新（ReleaseType 为 Security Update / Update Rollup 等）
        if let Ok(rt) = subkey.get_value::<String, _>("ReleaseType") {
            if rt != "Hotfix" && !rt.is_empty() {
                continue;
            }
        }

        // 必须有显示名称才视为有效软件
        let Ok(name) = subkey.get_value::<String, _>("DisplayName") else {
            continue;
        };
        if name.trim().is_empty() {
            continue;
        }

        let version = subkey.get_value::<String, _>("DisplayVersion").ok();
        let publisher = subkey.get_value::<String, _>("Publisher").ok();
        let install_location = subkey
            .get_value::<String, _>("InstallLocation")
            .ok()
            .map(|s| expand_env(&s));
        let uninstall_string = subkey
            .get_value::<String, _>("UninstallString")
            .ok()
            .map(|s| expand_env(&s));
        let quiet_uninstall_string = subkey
            .get_value::<String, _>("QuietUninstallString")
            .ok()
            .map(|s| expand_env(&s));
        let estimated_size_kb = subkey.get_value::<u32, _>("EstimatedSize").ok();
        let icon_path = subkey
            .get_value::<String, _>("DisplayIcon")
            .ok()
            .map(|s| expand_env(&s));

        apps.push(InstalledApp {
            name,
            version,
            publisher,
            install_location,
            uninstall_string,
            quiet_uninstall_string,
            estimated_size_mb: estimated_size_kb.map(|kb| (kb / 1024) as u64),
            icon_path,
            registry_path: format!(r"{}\{}", parent_path, subkey_name),
            system_wide,
        });
    }
}
