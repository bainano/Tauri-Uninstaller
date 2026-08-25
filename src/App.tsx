import { useEffect, useMemo, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { InstalledApp, NavKey } from './types';
import { TopBar } from './components/TopBar';
import { Sidebar } from './components/Sidebar';
import { AppList } from './components/AppList';
import { EmptyPlaceholder } from './components/EmptyPlaceholder';
import './App.css';

export default function App() {
  const [apps, setApps] = useState<InstalledApp[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [keyword, setKeyword] = useState('');
  const [nav, setNav] = useState<NavKey>('apps');

  const loadApps = async () => {
    setLoading(true);
    setError(null);
    try {
      const list = await invoke<InstalledApp[]>('scan_installed_apps');
      setApps(list);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadApps();
  }, []);

  const filtered = useMemo(() => {
    const kw = keyword.trim().toLowerCase();
    if (!kw) return apps;
    return apps.filter(
      (a) =>
        a.name.toLowerCase().includes(kw) ||
        (a.publisher ?? '').toLowerCase().includes(kw),
    );
  }, [apps, keyword]);

  const totalSize = useMemo(
    () => apps.reduce((s, a) => s + (a.estimated_size_mb ?? 0), 0),
    [apps],
  );

  return (
    <div className="app-shell">
      <TopBar keyword={keyword} onKeyword={setKeyword} onRefresh={loadApps} loading={loading} />
      <div className="app-body">
        <Sidebar active={nav} onSelect={setNav} appCount={apps.length} />
        <main className="app-main">
          {nav === 'apps' && (
            <AppList
              apps={filtered}
              loading={loading}
              error={error}
              totalSize={totalSize}
              onRefresh={loadApps}
            />
          )}
          {nav !== 'apps' && (
            <EmptyPlaceholder nav={nav} />
          )}
        </main>
      </div>
    </div>
  );
}
