interface TopBarProps {
  keyword: string;
  onKeyword: (v: string) => void;
  onRefresh: () => void;
  loading: boolean;
}

export function TopBar({ keyword, onKeyword, onRefresh, loading }: TopBarProps) {
  return (
    <header className="topbar" data-tauri-drag-region>
      <div className="topbar-left" data-tauri-drag-region>
        <div className="brand" data-tauri-drag-region>
          <span className="brand-logo">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M3 6h18M3 12h18M3 18h18" />
            </svg>
          </span>
          <span className="brand-name" data-tauri-drag-region>Tauri Uninstaller</span>
        </div>
      </div>

      <div className="topbar-center">
        <div className="search-box">
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <circle cx="11" cy="11" r="7" />
            <path d="m21 21-4.35-4.35" />
          </svg>
          <input
            value={keyword}
            onChange={(e) => onKeyword(e.target.value)}
            placeholder="搜索已安装的软件..."
          />
        </div>
      </div>

      <div className="topbar-right">
        <button className="icon-btn" title="重新扫描" onClick={onRefresh} disabled={loading}>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M21 12a9 9 0 1 1-2.64-6.36" />
            <path d="M21 3v6h-6" />
          </svg>
        </button>
      </div>
    </header>
  );
}
