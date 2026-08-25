use crate::models::AppEntry;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// 残留条目
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResidueItem {
    /// 类别：file / registry / shortcut
    pub kind: String,
    /// 绝对路径或注册表键路径
    pub path: String,
    /// 显示名称（文件名/键名）
    pub name: String,
    /// 大小（字节，文件类）
    pub size_bytes: Option<u64>,
    /// 是否安全可删（启发式）
    pub safe: bool,
}

/// 残留扫描报告
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResidueReport {
    pub files: Vec<ResidueItem>,
    pub shortcuts: Vec<ResidueItem>,
    pub registry: Vec<ResidueItem>,
    pub total_size_bytes: u64,
}

/// 残留删除结果
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RemoveResult {
    pub ok: Vec<String>,
    pub failed: Vec<(String, String)>,
}

/// 获取用于扫描的基准名称（软件名去壳）
fn base_name(app: &AppEntry) -> String {
    let name = app.name.trim().to_string();
    // 去掉常见后缀（如 " (x64)"、" 1.0" 等），保留主体用于路径匹配
    let mut n = name.clone();
    for suffix in [" (x64)", " (32-bit)", " 64-bit", " - 中文", " (zh-CN)"] {
        if n.to_lowercase().ends_with(&suffix.to_lowercase()) {
            n = n[..n.len() - suffix.len()].trim().to_string();
            break;
        }
    }
    if n.len() > 40 {
        n = n[..40].to_string();
    }
    n
}

/// 判断名称是否包含发布者或软件名特征（同名匹配）
fn name_matches(app: &AppEntry, candidate: &str) -> bool {
    let cand = candidate.to_lowercase();
    let base = base_name(app).to_lowercase();
    let pub_name = app
        .publisher
        .as_deref()
        .unwrap_or("")
        .trim()
        .to_lowercase();

    if base.len() >= 3 && cand.contains(&base) {
        return true;
    }
    // 发布者匹配：要求候选含完整发布者名
    if pub_name.len() >= 3 && cand.contains(&pub_name) {
        return true;
    }
    false
}

/// 获取用户/系统关键目录
fn known_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(ud) = std::env::var("USERPROFILE") {
        dirs.push(PathBuf::from(&ud).join("AppData").join("Local"));
        dirs.push(PathBuf::from(&ud).join("AppData").join("Roaming"));
        dirs.push(PathBuf::from(&ud).join("AppData").join("LocalLow"));
    }
    if let Ok(pd) = std::env::var("ProgramData") {
        dirs.push(PathBuf::from(pd));
    }
    dirs
}

/// 递归枚举目录下直接子项（一层），用于残留检测；太深不扫
fn scan_dir_level1(dir: &Path, app: &AppEntry, out: &mut Vec<ResidueItem>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let path = e.path();
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        let nm = name_matches(app, name);
        // 目录级命中：进入子层再列一层（最多两层）
        if nm {
            if path.is_dir() {
                if let Ok(sd) = std::fs::read_dir(&path) {
                    for sub in sd.flatten() {
                        let sp = sub.path();
                        let Some(sn) = sp.file_name().and_then(|s| s.to_str()) else {
                            continue;
                        };
                        if name_matches(app, sn) {
                            add_file_item(&sp, out);
                        }
                    }
                }
                // 目录本身也作为候选
                add_dir_item(&path, out);
            } else {
                add_file_item(&path, out);
            }
        }
    }
}

fn add_file_item(path: &Path, out: &mut Vec<ResidueItem>) {
    let size = std::fs::metadata(path).ok().map(|m| m.len());
    out.push(ResidueItem {
        kind: "file".into(),
        path: path.to_string_lossy().to_string(),
        name: path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default(),
        size_bytes: size,
        safe: true,
    });
}

fn add_dir_item(path: &Path, out: &mut Vec<ResidueItem>) {
    out.push(ResidueItem {
        kind: "file".into(),
        path: path.to_string_lossy().to_string(),
        name: path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default(),
        size_bytes: None,
        safe: true,
    });
}

/// 扫描软件残留（文件 + 快捷方式 + 注册表）
pub fn scan_residue(app: &AppEntry) -> ResidueReport {
    let mut files: Vec<ResidueItem> = Vec::new();
    let mut shortcuts: Vec<ResidueItem> = Vec::new();

    // 1. 安装目录本身（若存在）
    if let Some(il) = &app.install_location {
        let p = PathBuf::from(il);
        if p.exists() {
            add_dir_item(&p, &mut files);
        }
    }

    // 2. 常见目录扫描（AppData / ProgramData）
    for d in known_dirs() {
        scan_dir_level1(&d, app, &mut files);
    }

    // 3. 用户目录下同名文件夹（Downloads/Documents 等，一层）
    if let Ok(ud) = std::env::var("USERPROFILE") {
        let home = PathBuf::from(&ud);
        for sub in ["Documents", "Downloads", "Desktop"] {
            let dir = home.join(sub);
            if dir.exists() {
                scan_dir_level1(&dir, app, &mut files);
            }
        }
    }

    // 4. 快捷方式：开始菜单 + 桌面
    let mut lnk_roots = Vec::new();
    if let Ok(ud) = std::env::var("USERPROFILE") {
        lnk_roots.push(PathBuf::from(&ud).join("Desktop"));
        lnk_roots.push(
            PathBuf::from(&ud).join("AppData").join("Roaming").join("Microsoft").join("Windows").join("Start Menu"),
        );
    }
    if let Ok(pd) = std::env::var("ProgramData") {
        lnk_roots.push(
            PathBuf::from(&pd).join("Microsoft").join("Windows").join("Start Menu"),
        );
    }
    for root in lnk_roots {
        let Ok(entries) = std::fs::read_dir(&root) else {
            continue;
        };
        for e in entries.flatten() {
            let path = e.path();
            let is_lnk = path
                .extension()
                .map(|x| x.to_string_lossy().to_lowercase() == "lnk")
                .unwrap_or(false);
            if !is_lnk {
                continue;
            }
            let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
                continue;
            };
            if name_matches(app, name) {
                shortcuts.push(ResidueItem {
                    kind: "shortcut".into(),
                    path: path.to_string_lossy().to_string(),
                    name: name.to_string(),
                    size_bytes: None,
                    safe: true,
                });
            }
        }
    }

    // 5. 注册表残留
    let mut registry: Vec<ResidueItem> = Vec::new();
    // 5.1 当前卸载项自身
    if let Some(rk) = &app.registry_key {
        if !rk.is_empty() {
            registry.push(ResidueItem {
                kind: "registry".into(),
                path: rk.clone(),
                name: rk
                    .rsplit('\\')
                    .next()
                    .unwrap_or("")
                    .to_string(),
                size_bytes: None,
                safe: false, // 注册表删除需谨慎，默认不安全
            });
        }
    }
    // 5.2 HKLM\Software 下同名键（一层）
    scan_software_keys(app, &mut registry);

    let total_size_bytes = files
        .iter()
        .filter_map(|f| f.size_bytes)
        .sum::<u64>();

    ResidueReport {
        files,
        shortcuts,
        registry,
        total_size_bytes,
    }
}

/// 扫描 HKLM/HKCU Software 下与软件同名的键
fn scan_software_keys(app: &AppEntry, out: &mut Vec<ResidueItem>) {
    use winreg::enums::*;
    use winreg::RegKey;

    let base = base_name(app);
    if base.len() < 3 {
        return;
    }
    let hives: [(RegKey, &str); 2] = [
        (RegKey::predef(HKEY_LOCAL_MACHINE), "HKLM"),
        (RegKey::predef(HKEY_CURRENT_USER), "HKCU"),
    ];
    for (hive, hive_name) in hives {
        let ok = hive.open_subkey("Software");
        let Ok(sw) = ok else {
            continue;
        };
        let keys = sw.enum_keys();
        for key in keys.flatten() {
            let k = key.to_lowercase();
            let b = base.to_lowercase();
            if k == b || k.contains(&b) {
                out.push(ResidueItem {
                    kind: "registry".into(),
                    path: format!("{}\\Software\\{}", hive_name, key),
                    name: key,
                    size_bytes: None,
                    safe: false,
                });
            }
        }
    }
}

/// 删除残留项。
/// kind=file/shortcut 时移入回收站；kind=registry 时删除注册表键。
/// 返回逐项结果。
pub fn remove_residue(items: &[ResidueItem]) -> RemoveResult {
    let mut ok = Vec::new();
    let mut failed = Vec::new();
    for item in items {
        let r = match item.kind.as_str() {
            "registry" => delete_registry_key(&item.path),
            _ => recycle_file(&item.path),
        };
        match r {
            Ok(()) => ok.push(item.path.clone()),
            Err(e) => failed.push((item.path.clone(), e)),
        }
    }
    RemoveResult { ok, failed }
}

/// 删除注册表键（含子键）
fn delete_registry_key(path: &str) -> Result<(), String> {
    use winreg::enums::*;
    use winreg::RegKey;

    // 解析 "HKLM\..." / "HKCU\..."
    let lower = path.to_lowercase();
    let (hive, rest) = if lower.starts_with("hklm") {
        (RegKey::predef(HKEY_LOCAL_MACHINE), &path[4..])
    } else if lower.starts_with("hkcu") {
        (RegKey::predef(HKEY_CURRENT_USER), &path[4..])
    } else {
        return Err(format!("不支持的注册表路径: {}", path));
    };
    let rest = rest.trim_start_matches('\\');
    if rest.is_empty() {
        return Err("注册表路径缺少键名".into());
    }
    // 定位父键
    let (parent_path, key_name) = match rest.rfind('\\') {
        Some(idx) => (&rest[..idx], &rest[idx + 1..]),
        None => ("", rest),
    };
    let parent = if parent_path.is_empty() {
        hive
    } else {
        hive.open_subkey(parent_path)
            .map_err(|e| format!("打开注册表父键失败: {}", e))?
    };
    parent
        .delete_subkey_all(key_name)
        .map_err(|e| format!("删除注册表键失败: {}", e))
}

/// 移入回收站（SHFileOperationW）
fn recycle_file(path: &str) -> Result<(), String> {
    use std::ffi::OsStr;
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::UI::Shell::*;

    let wide: Vec<u16> = OsStr::new(path).encode_wide().chain(once(0)).collect();
    // SHFILEOPSTRUCTW 需要双 null 结尾
    let mut wide_double: Vec<u16> = OsStr::new(path).encode_wide().collect();
    wide_double.push(0);
    wide_double.push(0);

    let mut op = SHFILEOPSTRUCTW {
        hwnd: std::ptr::null_mut(),
        wFunc: FO_DELETE,
        pFrom: wide_double.as_ptr(),
        pTo: std::ptr::null(),
        fFlags: (FOF_ALLOWUNDO | FOF_NOCONFIRMATION | FOF_NOERRORUI | FOF_SILENT) as u16,
        fAnyOperationsAborted: 0,
        hNameMappings: std::ptr::null_mut(),
        lpszProgressTitle: std::ptr::null(),
    };
    unsafe {
        let r = SHFileOperationW(&mut op);
        if r == 0 {
            Ok(())
        } else {
            let _ = wide;
            Err(format!("移入回收站失败（代码 {}）", r))
        }
    }
}
