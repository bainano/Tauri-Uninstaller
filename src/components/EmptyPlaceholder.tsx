import type { NavKey } from '../types';

const CONTENT: Record<NavKey, { title: string; desc: string }> = {
  apps: { title: '已安装软件', desc: '' },
  residue: {
    title: '残留清理',
    desc: '扫描卸载后遗留的注册表项与文件（规划中，将在阶段二实现）',
  },
  everything: {
    title: 'Everything 高级搜索',
    desc: '集成 Everything 实现毫秒级文件搜索（规划中，将在阶段四实现）',
  },
  shortcut: {
    title: '快捷方式卸载',
    desc: '拖放快捷方式到窗口即可定位并卸载对应应用（规划中，将在阶段四实现）',
  },
  settings: {
    title: '设置',
    desc: '应用偏好与卸载行为配置（规划中）',
  },
};

export function EmptyPlaceholder({ nav }: { nav: NavKey }) {
  const c = CONTENT[nav];
  return (
    <section className="panel placeholder-panel">
      <div className="placeholder-icon">
        <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
          <circle cx="12" cy="12" r="10" />
          <path d="M12 16v-4M12 8h.01" />
        </svg>
      </div>
      <h2 className="panel-title">{c.title}</h2>
      <p className="panel-subtitle">{c.desc}</p>
    </section>
  );
}
