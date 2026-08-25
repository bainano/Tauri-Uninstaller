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
