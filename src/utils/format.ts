/** 将 MB 转换为可读大小文本 */
export function formatSize(mb: number): string {
  if (mb <= 0) return '—';
  if (mb < 1024) return `${mb} MB`;
  return `${(mb / 1024).toFixed(1)} GB`;
}
