# Tauri Uninstaller

现代化跨平台卸载工具，使用 Tauri v2 构建。以 [Bulk-Crap-Uninstaller](https://github.com/Klocman/Bulk-Crap-Uninstaller) 为功能蓝本，计划集成 Everything 高级文件搜索与"拖放快捷方式卸载"特色功能。

## 功能规划

- [x] 已安装软件扫描（Windows 注册表卸载信息键）
- [ ] 批量卸载（含静默卸载）
- [ ] 残留清理（注册表项 + 文件）
- [ ] Everything 集成（毫秒级文件搜索）
- [ ] 拖放快捷方式启动卸载
- [ ] 浅色模式现代化 UI（参考 Trae Work 界面风格）

## 技术栈

| 层 | 技术 |
| --- | --- |
| 桌面框架 | Tauri v2（Rust） |
| 前端 | React 19 + TypeScript + Vite 7 |
| 后端 | Rust（winreg 注册表读取） |
| 图标 | 内联 SVG（后续按需接入 Reicon） |

## 开发环境要求

- Node.js ≥ 20
- Rust 工具链（rustc / cargo）
- Windows 10/11（当前仅验证 Windows）

## 常用命令

```bash
# 安装前端依赖
npm install

# 启动开发模式（自动打开应用窗口）
npm run tauri dev

# 前端构建
npm run build

# Rust 后端编译检查
cd src-tauri && cargo check

# 打包安装程序
npm run tauri build
```

## 目录结构

```
Tauri-Uninstaller/
├── src/                  # 前端源码（React）
│   ├── components/       # 界面组件
│   │   ├── TopBar.tsx    # 顶栏（横跨页面，含搜索）
│   │   ├── Sidebar.tsx   # 侧边栏（与顶栏融合）
│   │   ├── AppList.tsx   # 软件列表
│   │   └── EmptyPlaceholder.tsx
│   ├── utils/format.ts   # 大小格式化等工具
│   ├── App.tsx           # 主应用
│   ├── App.css           # 全局样式（浅色模式）
│   └── types.ts          # 数据类型定义
├── src-tauri/            # 后端源码（Rust）
│   └── src/
│       ├── main.rs       # 入口
│       ├── lib.rs        # Tauri 命令注册
│       └── uninstaller.rs # 已安装软件扫描（注册表）
├── ui-reference/         # UI 参考素材（本地，不入库）
└── TraeWork Copy.zip     # 界面风格参考包（本地，不入库）
```

## 开发进度

### 阶段一：项目基础搭建（进行中）

- [x] Tauri v2 + React + TS 项目脚手架
- [x] 无边框窗口 + 自定义标题栏（含 WebView2 合成修复参数）
- [x] Git 仓库初始化 + 开发分支
- [ ] 远程仓库关联

### 阶段二：核心功能迁移（进行中）

- [x] 注册表扫描已安装软件（HKLM/HKCU、32/64 位）
- [ ] 卸载命令执行（标准 / 静默）

### 阶段三：界面开发（进行中）

- [x] 顶栏 + 侧边栏 + 圆角主区域布局
- [x] 软件列表（名称 / 版本 / 大小 / 发布者）
- [ ] 批量选择与卸载流程
- [ ] 卸载确认与进度反馈

### 阶段四：特色功能（未开始）

- [ ] Everything 集成
- [ ] 拖放快捷方式卸载

### 阶段五：测试与优化（未开始）

## 注意事项

- 窗口为无边框设计，顶栏区域可拖拽移动窗口
- 软件图标优先读取注册表指向的 exe；读取失败时显示占位图标
- 卸载功能涉及系统变更，实现时将提供明确的确认与安全提示
