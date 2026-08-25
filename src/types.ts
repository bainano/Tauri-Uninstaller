// 已安装软件数据结构（与后端 Rust InstalledApp 对齐）
export interface InstalledApp {
  name: string;
  version: string | null;
  publisher: string | null;
  install_location: string | null;
  uninstall_string: string | null;
  quiet_uninstall_string: string | null;
  estimated_size_mb: number | null;
  icon_path: string | null;
  registry_path: string;
  system_wide: boolean;
}

export type NavKey = 'apps' | 'residue' | 'everything' | 'shortcut' | 'settings';
