use serde::Serialize;

/// 统一的应用条目模型（注册表 / UWP / 便携应用共用）
#[derive(Serialize, Clone, Debug)]
pub struct AppEntry {
    /// 显示名称（DisplayName / Package Name）
    pub name: String,
    /// 发布者
    pub publisher: Option<String>,
    /// 版本
    pub version: Option<String>,
    /// 估算大小（MB，注册表 EstimatedSize / 安装目录扫描）
    pub size_mb: Option<u64>,
    /// 安装日期（ISO YYYY-MM-DD）
    pub install_date: Option<String>,
    /// 安装位置
    pub install_location: Option<String>,
    /// 卸载命令（优先 QuietUninstallString）
    pub uninstall_string: Option<String>,
    /// 强制卸载回退命令（原始 UninstallString）
    pub fallback_uninstall_string: Option<String>,
    /// DisplayIcon 路径（供后续提取图标）
    pub display_icon: Option<String>,
    /// 注册表键完整路径（用于强制卸载定位与残留扫描）
    pub registry_key: Option<String>,
    /// 数据来源：registry / uwp / portable
    pub source: String,
    /// 是否为系统组件（SystemComponent=1）
    pub is_system_component: bool,
    /// 卸载器是否缺失（无任何卸载命令）
    pub uninstaller_missing: bool,
    /// 是否 64 位应用（注册表视图推断）
    pub is_64bit: Option<bool>,
    /// 发布者签名验证状态（后续完善，当前留空）
    pub cert_status: Option<String>,
}

impl AppEntry {
    /// 判定该条目是否具备可用的卸载命令
    pub fn has_uninstaller(&self) -> bool {
        self.uninstall_string.is_some() || self.fallback_uninstall_string.is_some()
    }
}
