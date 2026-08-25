// Tauri invoke 封装：统一错误处理
const invoke = window.__TAURI__?.core?.invoke;

export async function call(cmd, args = {}) {
  if (!invoke) {
    throw new Error("Tauri bridge not ready");
  }
  try {
    return await invoke(cmd, args);
  } catch (e) {
    console.error(`[command:${cmd}]`, e);
    throw e;
  }
}

// 获取全部已安装应用（注册表 + UWP）
export async function listApps() {
  return call("list_apps");
}

// 执行完美卸载（调用原始卸载器）
export async function uninstallApp(key, opts = {}) {
  return call("uninstall_app", { key, silent: opts.silent ?? true, timeoutSecs: opts.timeoutSecs });
}

// 强制终止卸载进程树
export async function killUninstallProcess(pid) {
  return call("kill_uninstall_process", { pid });
}

// 扫描残留
export async function scanResidue(key) {
  return call("scan_residue", { key });
}

// 删除残留项
export async function removeResidue(items) {
  return call("remove_residue", { items });
}

// 强制卸载（删除注册表项 + 可选安装目录）
export async function forceUninstall(key, removeFiles) {
  return call("force_uninstall", { key, removeFiles });
}

// 是否管理员权限
export async function isElevated() {
  return call("is_elevated");
}
