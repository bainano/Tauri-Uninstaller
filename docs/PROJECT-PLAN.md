---
AIGC:
    Label: "1"
    ContentProducer: 001191440300708461136T1XGW3
    ProduceID: 150e4de0cca33addc475f12ffbaf6619_663bda98a02d11f1a238525400e6dd8f
    ReservedCode1: jbdbvDrGDSKa5l+fn/GODtgBrE0DGnZppNGKXBbDua4edbnEpAHXKjnUbi5oaNgOOV9uZXyeSgR4JQ22ZZYD2OSZnxMyMChIqlYOJV32nvO6n35lXNKoU66D+lPCV29VIfATQCjXjM2LtPHl1xwqQ0xY3pQaeQEuvmpLmnLSo0dTgPZ0dhIIAzd8GiM=
    ContentPropagator: 001191440300708461136T1XGW3
    PropagateID: 150e4de0cca33addc475f12ffbaf6619_663bda98a02d11f1a238525400e6dd8f
    ReservedCode2: jbdbvDrGDSKa5l+fn/GODtgBrE0DGnZppNGKXBbDua4edbnEpAHXKjnUbi5oaNgOOV9uZXyeSgR4JQ22ZZYD2OSZnxMyMChIqlYOJV32nvO6n35lXNKoU66D+lPCV29VIfATQCjXjM2LtPHl1xwqQ0xY3pQaeQEuvmpLmnLSo0dTgPZ0dhIIAzd8GiM=
---

# Tauri-Uninstaller 项目规划书

> 基于原版 Bulk Crap Uninstaller（BCUninstaller，Apache-2.0）的现代化 Tauri 重制版
> 远程仓库：https://github.com/bainano/Tauri-Uninstaller
> 本地目录：C:\Users\KIN\Documents\Tauri-Uninstaller
> 规划日期：2026-08-25

---

## 1. 项目概述

### 1.1 定位

把功能强大但界面老旧的 BCUninstaller，重写为一个**界面现代、交互流畅、核心功能精炼**的 Windows 卸载工具。不追求功能大而全，而是聚焦高频场景：

1. **看得清**：一眼看清电脑装了哪些软件（含隐藏项、便携软件、UWP 应用）
2. **卸得净**：调用原版卸载器 + 卸载后残留扫描清理（文件、注册表、快捷方式）
3. **找得快**：集成 Everything 实现秒级文件搜索，支持"按文件反查软件"与文件级清理
4. **拖一下就卸**：把桌面快捷方式拖到本软件窗口或快捷方式图标上，直接弹出迷你卸载 UI

### 1.2 与远程仓库的关系

- 远程 `bainano/Tauri-Uninstaller` 已 fork 原版 BCUninstaller 源码（master 分支，commit `608321d`）
- 本项目在本地目录 `C:\Users\KIN\Documents\Tauri-Uninstaller` 新建独立 Tauri 工程（全新代码），通过 git 关联远程仓库，原版源码保留在独立分支/目录作为参考（详见 8.2 分支策略）
- 合规：继承 Apache-2.0 许可，保留 LICENSE 与本项目声明

### 1.3 设计原则

| 原则 | 说明 |
|---|---|
| 界面优先 | 浅色、克制、圆角、留白，对标 TraeWork 设计语言 |
| 核心精炼 | 只保留用户真正高频使用的功能，砍掉长尾复杂度 |
| 安全兜底 | 所有删除动作默认进回收站、先展示后执行、系统关键项黑名单保护 |
| Rust 主力 | 系统级操作全部下沉 Rust，前端只管展示与交互 |

---

## 2. 原版 BCUninstaller 功能盘点与取舍

### 2.1 原版功能全景（调研自官网与社区资料）

| # | 功能模块 | 说明 | 取舍 |
|---|---|---|---|
| 1 | 多来源软件检测 | 注册表卸载项（普通/隐藏/损坏）、便携软件、Steam/Oculus、UWP、Chocolatey、Windows 更新与功能 | ✅ 保留核心（注册表 + UWP + 便携） |
| 2 | 批量卸载队列 | 多选排队执行、碰撞预防、崩溃/挂起处理 | ✅ 保留（简化版） |
| 3 | 静默卸载 | 自动处理不支持静默的卸载器 | ✅ 保留（尝试静默参数） |
| 4 | 残留扫描清理 | 卸载后扫描文件/文件夹/注册表/快捷方式残留 | ✅ 保留（核心卖点） |
| 5 | 强制卸载 | 绕过/缺失卸载器时手动移除文件+注册表 | ✅ 保留 |
| 6 | 证书验证 | 卸载器数字签名验证，绿/蓝/红/黄标记 | ✅ 保留（仅展示不阻断） |
| 7 | 高级筛选搜索 | 预设筛选 + 自定义规则 + 正则 | ✅ 简化（即时搜索 + 快速筛选） |
| 8 | 详细数据展示 | 版本、发布者、安装路径、GUID、安装日期、大小 | ✅ 保留（详情面板） |
| 9 | 启动项管理器 | 查看/禁用开机启动项 | ⏸ V2 可选 |
| 10 | 通过窗口/快捷方式/目录卸载 | 拖快捷方式/选窗口/选目录反查软件 | ✅ 保留（升级为拖拽迷你卸载） |
| 11 | 预设卸载列表 | XML 列表自动批量卸载（IT 部署） | ⏸ V2 可选 |
| 12 | 前后置命令钩子 | 卸载前后执行自定义命令 | ❌ 不做（简化） |
| 13 | 控制台接口 | 命令行自动化卸载 | ❌ 不做（简化） |
| 14 | 便携模式/单文件设置 | 免安装、设置集中保存 | ⏸ V2 可选 |
| 15 | Program Files 空文件夹清理 | 清理无效/空目录 | ⏸ V2 可选 |
| 16 | 应用评分 | 社区评分辅助判断 | ❌ 不做（简化） |
| 17 | 多语言 | 翻译支持 | ✅ 仅中文（后续可扩展） |
| 18 | 数据导出 | 软件清单导出（库存审计） | ⏸ V2 可选（JSON/CSV） |

### 2.2 本项目新增能力（原版没有或体验不好的）

| # | 新增能力 | 说明 |
|---|---|---|
| N1 | Everything 文件搜索集成 | 秒级全局文件搜索；按文件反查所属软件；文件级清理/定位 |
| N2 | 快捷方式拖拽迷你卸载 | 拖 .lnk 到窗口 → 解析目标 → 反查软件 → 迷你 UI 一键完美卸载 |
| N3 | Windows 原生拖放启动 | 把 .lnk 拖到程序快捷方式图标上，Windows 自动以命令行参数启动本程序，直接进入卸载流程 |
| N4 | Web 现代化界面 | 浅色主题、侧边栏+顶栏一体、圆角主区，对标 TraeWork |

---

## 3. 功能规划（分阶段）

### 3.1 V1 核心功能（MVP，本期开发）

#### F1 软件列表（主界面）
- 数据来源：
  - 注册表卸载项：HKLM / HKCU × 32位 / 64位 四个视图（对应 `Software\Microsoft\Windows\CurrentVersion\Uninstall`）
  - UWP 应用：`Get-AppxPackage`（PowerShell 或 Windows API）
  - 便携软件：常见目录扫描（可选开关）
- 列表字段：名称、发布者、版本、安装大小、安装日期、来源类型、证书状态、卸载器状态
- 状态色标记：绿=已验证 / 蓝=未验证 / 黄=未注册 / 红=卸载器缺失（沿用原版语义）
- 排序：按名称/大小/日期；分组：按来源/发布者

#### F2 搜索与筛选
- 顶部搜索框即时过滤（名称/发布者/安装路径）
- 快速筛选：仅看可卸载、卸载器缺失、Microsoft 应用、UWP、便携软件、已勾选
- 选中计数 + 合计大小实时显示

#### F3 完美卸载（核心链路）
1. 确认面板：软件名、发布者、大小、安装路径、卸载器来源、风险提示
2. 执行：优先调用原始卸载器（`UninstallString` / `QuietUninstallString`）；超时/挂起检测
3. 残留扫描：卸载完成后扫描
   - 文件残留：安装目录、ProgramData、AppData、用户目录下同名/同发布者路径
   - 注册表残留：卸载项自身、Software 下的同名键
   - 快捷方式残留：开始菜单、桌面
4. 残留结果页：分类列出（文件/注册表/快捷方式），默认勾选安全项，**删除前确认**
5. 完成报告

#### F4 强制卸载
- 适用于：卸载器缺失、损坏、执行失败
- 流程：列出待移除的注册表项 + 安装目录文件 → 用户确认 → 删除（文件进回收站）

#### F5 批量卸载
- 多选软件 → 队列执行，逐个走完美卸载流程，失败项自动跳过并汇总
- 提示：同批次卸载器冲突预防（同一安装根目录的软件不建议同批）

#### F6 软件详情面板
- 右侧滑出面板：完整元数据（GUID、卸载命令、注册表路径、安装位置、DisplayIcon、SystemComponent 等）
- 快捷操作：打开安装目录 / 打开注册表项 / 官网 / 复制信息

#### F7 快捷方式拖拽迷你卸载（亮点）
- 触发方式 A：把 `.lnk` / `.exe` / 文件夹 拖入主窗口 → 弹出迷你卸载窗口
- 触发方式 B：把 `.lnk` 拖到桌面/开始菜单的 Tauri-Uninstaller 快捷方式图标上 → Windows 原生传参启动 → 直接打开迷你卸载窗口（无需先开主程序）
- 解析链路：`.lnk` → IShellLink COM 解析目标路径 → 遍历注册表卸载项反查（匹配 DisplayIcon / InstallLocation / UninstallString）→ 兜底：Everything 搜 exe 文件信息 / 提示手动匹配
- 迷你窗口 UI：软件图标 + 名称 + 大小 + [一键完美卸载] [取消]；紧凑卡片，随主窗口关闭而关闭
- 支持的拖放对象：快捷方式(.lnk)、可执行文件(.exe)、目录（便携软件）、URL 快捷方式（识别后转浏览器卸载场景提示）

#### F8 Everything 文件搜索集成
- 能力：
  1. 全局文件搜索（文件名秒级匹配，支持通配符）
  2. "按文件反查软件"：选中某文件 → 定位所属软件 → 直接发起卸载
  3. 文件级清理：搜索残留/大文件 → 勾选删除（进回收站）
  4. "打开所在位置"：在 Everything 中定位文件
- 后端适配（自动降级）：
  1. **Everything SDK**（Everything64.dll + Rust FFI）：首选，需本机安装 Everything
  2. **HTTP JSON API**（Everything 开启 HTTP 服务器）：备选
  3. **es.exe 命令行**：兜底
  4. 未安装 Everything：检测注册表/进程提示引导安装（可选内置下载引导）
- 集成点：软件详情页"残留文件"可用 Everything 辅助搜索同名文件；独立"文件搜索"页

### 3.2 V2 增强功能（规划预留，本期不开发）

- 启动项管理器（查看/禁用开机启动项）
- 数据导出（JSON / CSV 软件清单）
- Steam 游戏检测
- Program Files 空文件夹/无效目录清理
- 预设卸载列表（XML 批量方案）
- 便携模式（设置文件集中保存）

---

## 4. 技术架构

### 4.1 技术栈总览

| 层 | 选型 | 理由 |
|---|---|---|
| 桌面框架 | Tauri v2 + Rust 1.97 | 体积小、性能好、系统 API 强 |
| 前端构建 | Vite + 原生 HTML/CSS/JS | 与闪念胶囊项目一致；TraeWork 本就是 CSS 组件库，无需框架；减少依赖复杂度 |
| UI 资产 | TraeWork 设计系统（token + components.css + 671 个 SVG 图标） | 浅色语言完全匹配项目诉求 |
| 系统 API | `windows` crate（COM/注册表/进程）+ `winreg` crate | 官方 Rust 绑定 + 成熟注册表库 |
| 数据序列化 | serde / serde_json | 前后端 command 通信 |

### 4.2 Rust 关键模块设计

```
src-tauri/src/
├── main.rs / lib.rs          # Tauri 入口，command 注册
├── commands.rs               # 前端可调用命令层（列表/卸载/搜索/拖拽）
├── uninstall/
│   ├── registry.rs           # 注册表卸载项枚举（4 视图合并去重）
│   ├── uwp.rs                # UWP 应用枚举
│   ├── portable.rs           # 便携软件目录扫描
│   ├── runner.rs             # 卸载器执行（静默参数探测、超时、挂起处理）
│   ├── leftovers.rs          # 残留扫描（文件/注册表/快捷方式）+ 清理
│   ├── force.rs              # 强制卸载
│   └── shortcut.rs           # .lnk 解析（IShellLink）+ 反查软件
├── search/
│   ├── everything.rs         # Everything 三后端适配（SDK/HTTP/es）
│   └── resolver.rs           # 文件 → 软件 反查
└── safety/
    └── blacklist.rs          # 系统关键项黑名单（内核/驱动/本程序自身等）
```

### 4.3 前端页面结构

```
src/
├── index.html                # 主窗口壳（侧边栏+顶栏+主区）
├── mini.html                 # 迷你卸载窗口
├── styles/
│   ├── tokens.css            # TraeWork token（裁剪）
│   ├── components.css        # TraeWork .ds-* 组件
│   └── app.css               # 应用布局与页面样式
├── scripts/
│   ├── app.js                # 主入口、路由（单页视图切换）
│   ├── api.js                # invoke 封装
│   ├── views/                # list / detail / uninstall / leftovers / search / settings
│   └── components/           # 表格、筛选器、详情面板、进度条、弹窗、Toast
└── assets/icons/             # TraeWork SVG（按需复制，勿全量）
```

### 4.4 Everything 集成方案对比（已在 3.1 F8 列出自适应降级策略）

| 方案 | 依赖 | 优点 | 缺点 |
|---|---|---|---|
| SDK DLL（FFI） | 本机安装 Everything | 快、功能全、无端口 | 需 Rust 绑定编写 |
| HTTP JSON API | Everything 开启 HTTP 服务 | 实现简单、跨端 | 需用户手动开启服务（可引导） |
| es.exe 命令行 | Everything 附带/单独下载 | 最简实现 | 每查一次起进程，慢；需配置 |

开发实现优先级：SDK FFI 为主，HTTP API 为辅，未安装时引导下载 Everything。

---

## 5. UI/UX 设计规范（基于 TraeWork 设计系统）

### 5.1 布局（对标 Trae Work / TraeWork 设计包截图）

```
┌──────────────────────────────────────────────────────────┐
│ 顶栏 64px：页面标题（左）+ 搜索/操作按钮（右）           │
├──────────────┬───────────────────────────────────────────┤
│ 侧边栏       │ 主区域（圆角卡片容器，四周留 16-24px）    │
│ 200-240px    │                                           │
│ · 品牌标题   │  ┌─────────────────────────────────────┐  │
│ · 导航分组   │  │ 内容区：列表/详情/向导/搜索          │  │
│   - 软件管理 │  │ 垂直滚动，无横向滚动                 │  │
│   - 文件搜索 │  └─────────────────────────────────────┘  │
│   - 残留清理 │                                           │
│   - 设置     │                                           │
└──────────────┴───────────────────────────────────────────┘
```

- 侧边栏与顶栏视觉融合（同底色浅灰 #F8F9FA），主区域独立白色圆角卡片（圆角 16-20px，与容器有间距）
- 选中导航项：浅灰底高亮 + 小圆角

### 5.2 设计 Token（从 TraeWork colors_and_type.css 提取，开发时裁剪导入）

| 类别 | 值 |
|---|---|
| 主区背景 | #FFFFFF |
| 侧边栏/顶栏背景 | #F8F9FA |
| 品牌强调色 | #6C5CE7（主按钮） |
| 交互蓝 | #0D6EFD（开关/链接） |
| 标题文本 | #212529 |
| 正文文本 | #495057 |
| 辅助文本 | #6C757D |
| 边框 | #DEE2E6 |
| 圆角 | 控件 8px / 卡片 12px / 主区域 16-20px / 胶囊 full |
| 间距 | 区块 24-32px / 元素 12-16px / 导航 8px |
| 字体 | SF Pro Text / PingFang SC / system-ui；标题 24px/700，区块标题 16px/600，正文 14px，辅助 12px |
| 状态色 | 绿=已验证、蓝=未验证、黄=未注册、红=卸载器缺失（淡底深字标签） |
| 阴影 | 扁平化为主，仅弹窗/悬浮层轻微阴影 |

### 5.3 页面清单

| 页面 | 说明 |
|---|---|
| 软件列表 | 主页面：搜索框 + 筛选条 + 软件表格（卡片式行）+ 批量操作栏 + 多选 |
| 软件详情 | 右侧滑出面板（F6） |
| 卸载向导 | 确认 → 执行进度 → 残留扫描 → 完成报告（全屏步骤条） |
| 残留结果 | 分类勾选列表（文件/注册表/快捷方式） |
| 文件搜索 | Everything 集成页：搜索框 + 结果列表 + 反查/删除/定位操作 |
| 设置 | 扫描来源开关、Everything 配置、回收站策略、黑名单管理 |
| 迷你卸载窗口 | 拖拽触发的小卡片窗口 |

---

## 6. 快捷方式拖拽迷你卸载（核心亮点详解）

### 6.1 两种触发路径

**路径 A（窗口内拖拽）**
1. 主窗口注册 `onDragDropEvent`（Tauri v2 原生支持）
2. 拖入 `.lnk` → 调用 Rust `shortcut::resolve` 解析目标路径
3. 反查软件 → 打开迷你窗口

**路径 B（拖到程序快捷方式上启动，Windows 原生机制）**
1. 用户把任意 `.lnk` 拖到桌面/任务栏/开始菜单的 Tauri-Uninstaller 快捷方式上
2. Windows 原生行为：以被拖文件路径作为命令行参数启动本程序
3. 程序启动时检查 `std::env::args()`，若存在 `.lnk` 参数 → 直接进入迷你卸载流程（不打开主界面）
4. 零额外代码实现系统级"拖放启动"，体验接近原生 App

### 6.2 解析与反查链路

```
.lnk → IShellLink 解析 → 目标 exe 绝对路径
     → 遍历注册表卸载项匹配：
         a) InstallLocation 前缀匹配
         b) DisplayIcon / UninstallString 中的路径匹配
         c) exe 文件名 + 目录名模糊匹配
     → 命中：打开迷你卸载 UI（图标/名称/大小/一键完美卸载）
     → 未命中：提示"未找到匹配软件"，提供 [手动搜索 Everything] [仅删除该快捷方式]
```

### 6.3 迷你窗口设计

- 尺寸约 360×200，无边框圆角卡片，可置顶、可拖动
- 内容：软件图标 + 名称 + 版本 + 大小 + 安装路径（截断） + [一键完美卸载] [取消]
- 卸载走 F3 完整链路，完成后显示残留扫描结果，窗口自动收起

---

## 7. 安全设计（卸载类工具的第一优先级）

| 风险 | 对策 |
|---|---|
| 误删系统组件 | 内置黑名单（本程序自身、关键系统组件、驱动、运行时库）；`SystemComponent=1` 默认标记警告 |
| 误删用户数据 | 文件删除一律进回收站（可配置）；残留扫描默认只读展示，勾选后才执行 |
| 卸载器恶意行为 | 展示卸载器来源与证书状态；卸载前可查 UninstallString 原文 |
| 权限不足 | 检测进程是否管理员，必要时引导以管理员身份重启（不做静默提权） |
| 批量误操作 | 批量卸载前二次确认清单；同目录冲突检测 |
| 拖拽解析安全 | 仅接受本机文件路径，拒绝网络路径/远程位置；.lnk 解析不执行目标 |

---

## 8. 工程规划

### 8.1 目录结构（目标形态）

```
C:\Users\KIN\Documents\Tauri-Uninstaller\
├── .git/                    # git 仓库（关联远程 bainano/Tauri-Uninstaller）
├── .gitignore               # node_modules / dist / target / temp
├── package.json             # vite + @tauri-apps/cli + @tauri-apps/api
├── vite.config.js
├── index.html → src/index.html
├── src/                     # 前端（见 4.3）
├── src-tauri/               # Rust 后端（见 4.2）
│   ├── Cargo.toml
│   ├── build.rs
│   ├── tauri.conf.json      # 多窗口（main + mini）、withGlobalTauri
│   ├── capabilities/default.json
│   └── src/
├── docs/                    # 规划、设计文档
├── assets/                  # 应用图标、安装包素材
├── reference/               # 原版 BCUninstaller 源码参考（fork 内容，独立保留）
└── LICENSE                  # Apache-2.0（保留原版声明）
```

### 8.2 git 与分支策略

1. `git init` 本地仓库，关联远程 `origin https://github.com/bainano/Tauri-Uninstaller.git`
2. 原版 fork 内容保留在 `reference/` 或独立分支 `upstream-bcu`（`git fetch` 原版仓库到本地参考，不混入主开发分支）
3. 主分支 `main` 只放 Tauri 全新工程
4. 初始提交：骨架 + 本规划文档；后续按里程碑提交
5. 推送策略：骨架完成后推 `main`，后续每完成一个里程碑推送一次

### 8.3 开发里程碑

| 里程碑 | 内容 | 预估 |
|---|---|---|
| M0 工程骨架 | git 初始化+关联远程；Tauri v2 骨架；TraeWork token/组件/图标裁剪导入；主窗口壳（侧边栏+顶栏+主区）跑通 | 0.5 天 | ✅ |
| M1 数据层 | 注册表卸载项枚举（4 视图去重）、UWP 枚举、软件列表渲染、搜索筛选、详情面板 | 2 天 | ✅ |
| M2 卸载链路 | 完美卸载（原卸载器+静默探测+超时）、残留扫描清理、强制卸载、批量卸载、卸载向导 UI | 3 天 |
| M3 亮点功能 | 快捷方式解析+反查、迷你窗口、拖拽（窗口内+系统拖放启动）、Everything 集成（SDK 优先） | 2.5 天 |
| M4 打磨发布 | 设置页、证书状态展示、黑名单完善、导出（如做）、打包（NSIS/MSI）、图标与安装包 | 2 天 |

### 8.4 技术风险与对策

| 风险 | 对策 |
|---|---|
| IShellLink 解析复杂 | 先用 Windows COM 标准接口；备选：PowerShell WScript.Shell 兜底解析 |
| UWP 枚举权限 | 用系统 PowerShell `Get-AppxPackage` 输出 JSON，进程级调用 |
| Everything 未安装 | 检测 + 引导页；搜索功能优雅降级为普通文件遍历（限定目录） |
| 卸载器弹窗拦截 | 第一版不自动化弹窗，提示用户手动完成卸载器向导，完成后点击"继续" |
| 静默参数探测误伤 | 仅对已知安全参数（/S /silent /qn 等）尝试，默认走可视化卸载 |
| 管理员权限 | 核心命令检测提权状态，需要时引导重启提权 |

---

## 9. 下一步行动（确认后立即执行）

1. **M0 工程骨架**：git init + 关联远程 + Tauri v2 骨架 + TraeWork 资产导入 + 主窗口壳
2. 交付可运行的空壳应用（`npm run tauri dev` 可启动，可见侧边栏/顶栏/圆角主区布局）
3. 再进入 M1 数据层开发

> 规划确认或调整后，即从 M0 开始。
*（内容由AI生成，仅供参考）*
