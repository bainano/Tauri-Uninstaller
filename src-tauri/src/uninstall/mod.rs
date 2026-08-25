pub mod registry;
pub mod uwp;

use crate::models::AppEntry;

/// 汇总全部应用（注册表 + UWP）
pub fn enumerate_all() -> Vec<AppEntry> {
    let mut apps = registry::enumerate();
    apps.extend(uwp::enumerate());
    apps
}
