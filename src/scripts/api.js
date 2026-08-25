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

// 占位：M0 阶段验证桥接可用
export async function greet(name) {
  return call("greet", { name });
}
