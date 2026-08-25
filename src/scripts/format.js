// 通用格式化工具（独立模块，避免循环依赖）
export function fmtSize(mb) {
  if (mb == null) return "—";
  if (mb >= 1024) return `${(mb / 1024).toFixed(1)} GB`;
  return `${mb} MB`;
}

export function fmtDate(d) {
  return d || "—";
}
