use std::process::Command;

use serde_json::Value;

use crate::models::AppEntry;

const PS_SCRIPT: &str = r#"
Get-AppxPackage -AllUsers |
  Where-Object { $_.IsFramework -eq $false } |
  Select-Object Name, PackageFullName, Publisher, Version, InstallLocation, Architecture, IsBundle |
  ConvertTo-Json -Compress -Depth 2
"#;

/// 枚举 UWP / MSIX 应用（跳过框架包）
pub fn enumerate() -> Vec<AppEntry> {
    let output = match Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", PS_SCRIPT])
        .output()
    {
        Ok(o) if o.status.success() => o,
        Ok(_) => return Vec::new(),
        Err(_) => return Vec::new(),
    };

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        return Vec::new();
    }

    let parsed: Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let items: Vec<&Value> = match parsed {
        Value::Array(ref arr) => arr.iter().collect(),
        Value::Object(_) => vec![&parsed],
        _ => return Vec::new(),
    };

    items
        .into_iter()
        .filter_map(|item| {
            let name = item.get("Name")?.as_str()?.to_string();
            if name.is_empty() {
                return None;
            }
            let version = item
                .get("Version")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let install_location = item
                .get("InstallLocation")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            Some(AppEntry {
                name,
                publisher: item
                    .get("Publisher")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                version,
                size_mb: None,
                install_date: None,
                install_location,
                // UWP 卸载统一走 Remove-AppxPackage，无需具体命令
                uninstall_string: None,
                fallback_uninstall_string: None,
                display_icon: None,
                registry_key: None,
                source: "uwp".to_string(),
                is_system_component: false,
                uninstaller_missing: false,
                is_64bit: item
                    .get("Architecture")
                    .and_then(|v| v.as_str())
                    .map(|a| a.contains("64")),
                cert_status: None,
            })
        })
        .collect()
}
