import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { InstalledApp } from '../types';
import { formatSize } from '../utils/format';

interface AppListProps {
  apps: InstalledApp[];
  loading: boolean;
  error: string | null;
  totalSize: number;
  onRefresh: () => void;
}

export function AppList({ apps, loading, error, totalSize, onRefresh }: AppListProps) {
  return (
    <section className="panel">
      <div className="panel-header">
        <div>
          <h2 className="panel-title">已安装软件</h2>
          <p className="panel-subtitle">
            {apps.length} 个应用 · 合计约 {formatSize(totalSize)}
          </p>
        </div>
        <button className="ghost-btn" onClick={onRefresh} disabled={loading}>
          重新扫描
        </button>
      </div>

      {loading && (
        <div className="state-box">
          <div className="spinner" />
          <p>正在扫描系统已安装软件...</p>
        </div>
      )}

      {!loading && error && (
        <div className="state-box error">
          <p>扫描失败：{error}</p>
        </div>
      )}

      {!loading && !error && apps.length === 0 && (
        <div className="state-box">
          <p>没有找到匹配的软件</p>
        </div>
      )}

      {!loading && !error && apps.length > 0 && (
        <div className="app-table">
          <div className="app-table-head">
            <span>名称</span>
            <span>版本</span>
            <span>大小</span>
            <span>发布者</span>
            <span className="col-action">操作</span>
          </div>
          <div className="app-table-body">
            {apps.map((app) => (
              <AppRow key={app.registry_path} app={app} />
            ))}
          </div>
        </div>
      )}
    </section>
  );
}

function AppRow({ app }: { app: InstalledApp }) {
  return (
    <div className="app-row">
      <div className="app-name-cell">
        <AppIcon app={app} />
        <div className="app-name-text">
          <span className="app-name">{app.name}</span>
          <span className="app-path">{app.install_location ?? app.publisher ?? '—'}</span>
        </div>
      </div>
      <span className="app-version">{app.version ?? '—'}</span>
      <span className="app-size">{formatSize(app.estimated_size_mb ?? 0)}</span>
      <span className="app-publisher">{app.publisher ?? '—'}</span>
      <div className="col-action">
        <button className="uninstall-btn" title="卸载此软件">
          卸载
        </button>
      </div>
    </div>
  );
}

// 图标缓存：避免重复调用后端提取
const iconCache = new Map<string, string>();

function AppIcon({ app }: { app: InstalledApp }) {
  const [src, setSrc] = useState<string | null>(null);
  const [failed, setFailed] = useState(false);

  useEffect(() => {
    const path = app.icon_path;
    if (!path) {
      setFailed(true);
      return;
    }

    const cached = iconCache.get(path);
    if (cached) {
      setSrc(cached);
      return;
    }

    let cancelled = false;
    invoke<string>('extract_app_icon', { path })
      .then((dataUrl) => {
        if (cancelled) return;
        iconCache.set(path, dataUrl);
        setSrc(dataUrl);
      })
      .catch(() => {
        if (!cancelled) setFailed(true);
      });

    return () => {
      cancelled = true;
    };
  }, [app.icon_path]);

  if (src) {
    return (
      <span className="app-icon">
        <img src={src} alt="" />
      </span>
    );
  }

  if (failed) {
    return (
      <span className="app-icon">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
          <path d="M14 2v6h6" />
        </svg>
      </span>
    );
  }

  // 加载中：占位
  return (
    <span className="app-icon">
      <span className="icon-loading" />
    </span>
  );
}
