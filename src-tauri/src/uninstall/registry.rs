use std::collections::HashMap;

use winreg::{enums::*, RegKey};

use crate::models::AppEntry;

const UNINSTALL_SUBKEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall";

/// 枚举全部注册表卸载项，返回去重后的列表
///
/// 视图覆盖：HKLM/HKCU × 64 位视图 / WOW6432Node（32 位存放点）
pub fn enumerate() -> Vec<AppEntry> {
    let mut seen: HashMap<String, ()> = HashMap::new();
    let mut entries: Vec<AppEntry> = Vec::new();

    let views = vec![
        (
            RegKey::predef(HKEY_LOCAL_MACHINE),
            UNINSTALL_SUBKEY,
            true,
            true,
        ),
        (
            RegKey::predef(HKEY_LOCAL_MACHINE),
            "Software\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
            false,
            true,
        ),
        (RegKey::predef(HKEY_CURRENT_USER), UNINSTALL_SUBKEY, true, false),
        (
            RegKey::predef(HKEY_CURRENT_USER),
            "Software\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
            false,
            false,
        ),
    ];

    for (root, path, is_64, is_machine) in views.into_iter() {
        let Ok(key) = root.open_subkey_with_flags(path, KEY_READ | KEY_WOW64_64KEY) else {
            continue;
        };

        let mut subkeys: Vec<String> = Vec::new();
        for name in key.enum_keys().flatten() {
            subkeys.push(name);
        }

        for sub_name in subkeys {
            let Some(mut entry) = read_entry(&key, &sub_name, is_64) else {
                continue;
            };
            entry.registry_key = Some(format!(
                "{}\\{}",
                if is_machine { "HKLM" } else { "HKCU" },
                sub_name
            ));

            // 去重键：名称 + 发布者（小写）；有卸载命令的条目优先保留
            let dedup_key = format!(
                "{}|{}",
                entry.name.to_lowercase(),
                entry.publisher.clone().unwrap_or_default().to_lowercase()
            );
            if seen.contains_key(&dedup_key) {
                continue;
            }
            seen.insert(dedup_key, ());
            entries.push(entry);
        }
    }

    entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    entries
}

/// 读取单个注册表子键并构造成 AppEntry
fn read_entry(parent: &RegKey, sub_name: &str, is_64: bool) -> Option<AppEntry> {
    let key = parent.open_subkey(sub_name).ok()?;

    let get_str = |name: &str| -> Option<String> {
        key.get_value::<String, _>(name)
            .ok()
            .filter(|v| !v.trim().is_empty())
    };
    let get_dword = |name: &str| -> Option<u32> { key.get_value::<u32, _>(name).ok() };

    let name = get_str("DisplayName")?;
    let uninstall_string = get_str("QuietUninstallString")
        .or_else(|| get_str("UninstallString"))
        .filter(|s| !s.to_lowercase().contains("none"));
    let fallback = get_str("UninstallString");

    let system_component = get_dword("SystemComponent").unwrap_or(0) == 1;
    let uninstaller_missing = uninstall_string.is_none() && fallback.is_none();

    // EstimatedSize 单位 KB → MB（向上取整）
    let size_mb = get_dword("EstimatedSize").map(|kb| (kb as u64 + 1023) / 1024);

    // InstallDate 格式 YYYYMMDD → YYYY-MM-DD
    let install_date = get_str("InstallDate").and_then(|d| {
        if d.len() == 8 && d.chars().all(|c| c.is_ascii_digit()) {
            Some(format!("{}-{}-{}", &d[0..4], &d[4..6], &d[6..8]))
        } else {
            None
        }
    });

    Some(AppEntry {
        name,
        publisher: get_str("Publisher"),
        version: get_str("DisplayVersion"),
        size_mb,
        install_date,
        install_location: get_str("InstallLocation"),
        uninstall_string,
        fallback_uninstall_string: fallback,
        display_icon: get_str("DisplayIcon"),
        registry_key: Some(sub_name.to_string()),
        source: "registry".to_string(),
        is_system_component: system_component,
        uninstaller_missing,
        is_64bit: Some(is_64),
        cert_status: None,
    })
}
