// 软件管理视图：列表渲染、搜索过滤、排序、多选、详情面板
import { listApps } from "./api.js";
import { openUninstall, appCanUninstall } from "./uninstall-view.js";
import { fmtSize, fmtDate } from "./format.js";

const state = {
  apps: [],
  query: "",
  filter: "all",
  sortKey: "name",
  sortDir: 1,
  selected: new Set(),
  loaded: false,
};

const FILTERS = {
  all: () => true,
  uninstallable: (a) => !a.uninstaller_missing,
  missing: (a) => a.uninstaller_missing,
  system: (a) => a.is_system_component,
  uwp: (a) => a.source === "uwp",
};

const AVATAR_COLORS = ["#6C5CE7", "#4B3FE3", "#8B5CF6", "#3B82F6", "#0EA5E9", "#10B981", "#F59E0B", "#EF4444"];

function hashHue(str) {
  let h = 0;
  for (let i = 0; i < str.length; i++) h = (h * 31 + str.charCodeAt(i)) >>> 0;
  return AVATAR_COLORS[h % AVATAR_COLORS.length];
}

export async function loadApps() {
  try {
    state.apps = await listApps();
    state.loaded = true;
    state.selected.clear();
    render();
  } catch (e) {
    const count = document.getElementById("app-count");
    if (count) count.textContent = "加载失败";
    document.getElementById("app-tbody").innerHTML =
      `<tr><td colspan="7" class="app-empty">加载失败：${e.message}</td></tr>`;
  }
}

function filteredApps() {
  const q = state.query.trim().toLowerCase();
  return state.apps
    .filter(FILTERS[state.filter] || FILTERS.all)
    .filter((a) => {
      if (!q) return true;
      return (
        a.name.toLowerCase().includes(q) ||
        (a.publisher || "").toLowerCase().includes(q) ||
        (a.install_location || "").toLowerCase().includes(q)
      );
    })
    .sort((x, y) => {
      const k = state.sortKey;
      let a = x[k], b = y[k];
      if (a == null) a = "";
      if (b == null) b = "";
      if (typeof a === "number" && typeof b === "number") return (a - b) * state.sortDir;
      return String(a).localeCompare(String(b), "zh-Hans-CN") * state.sortDir;
    });
}

function render() {
  const tbody = document.getElementById("app-tbody");
  const count = document.getElementById("app-count");
  const selectedInfo = document.getElementById("selected-info");
  const list = filteredApps();

  count.textContent = state.loaded
    ? `共 ${state.apps.length} 个应用 · 显示 ${list.length} 个`
    : "加载中…";

  if (!state.loaded) {
    tbody.innerHTML = `<tr><td colspan="7" class="app-empty">正在扫描注册表与 UWP 应用…</td></tr>`;
    return;
  }

  if (list.length === 0) {
    tbody.innerHTML = `<tr><td colspan="7" class="app-empty">没有匹配的应用</td></tr>`;
  } else {
    tbody.innerHTML = list.map((app, i) => {
      const checked = state.selected.has(app.name);
      const avatar = (app.name[0] || "?").toUpperCase();
      const sub = app.publisher || (app.is_64bit ? "64 位" : "32 位");
      const sourceTag =
        app.source === "uwp"
          ? `<span class="tag tag-uwp">UWP</span>`
          : app.is_system_component
            ? `<span class="tag tag-system">系统</span>`
            : app.uninstaller_missing
              ? `<span class="tag tag-warn">无卸载器</span>`
              : `<span class="tag tag-ok">可卸载</span>`;
      return `
      <tr class="app-row" data-name="${escapeAttr(app.name)}">
        <td class="col-check">
          <input type="checkbox" class="app-check" ${checked ? "checked" : ""} aria-label="选择 ${escapeAttr(app.name)}" />
        </td>
        <td class="col-name">
          <div class="app-ident">
            <span class="app-avatar" style="background:${hashHue(app.name)}">${escapeHtml(avatar)}</span>
            <div class="app-name-main">
              <span class="app-name">${escapeHtml(app.name)}</span>
              <span class="app-sub">${escapeHtml(sub)}</span>
            </div>
          </div>
        </td>
        <td class="col-publisher">${escapeHtml(app.publisher || "—")}</td>
        <td class="col-version">${escapeHtml(app.version || "—")}</td>
        <td class="col-size">${fmtSize(app.size_mb)}</td>
        <td class="col-date">${fmtDate(app.install_date)}</td>
        <td class="col-source">${sourceTag}</td>
      </tr>`;
    }).join("");
  }

  selectedInfo.textContent =
    state.selected.size > 0
      ? `已选择 ${state.selected.size} 个应用`
      : "点击行或复选框选择应用";

  syncSortArrows();
}

function syncSortArrows() {
  document.querySelectorAll(".sortable").forEach((th) => {
    const active = th.dataset.sort === state.sortKey;
    th.classList.toggle("sorted", active);
    th.classList.toggle("desc", active && state.sortDir === -1);
  });
}

function escapeHtml(s) {
  return String(s).replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]));
}

function escapeAttr(s) {
  return String(s).replace(/"/g, "&quot;");
}

export function initSoftwareView({ searchInput, refreshBtn }) {
  const tbody = document.getElementById("app-tbody");

  // 搜索过滤
  searchInput.addEventListener("input", (e) => {
    state.query = e.target.value;
    render();
  });

  // 刷新
  refreshBtn.addEventListener("click", loadApps);

  // 筛选 chips
  document.querySelectorAll(".chip").forEach((chip) => {
    chip.addEventListener("click", () => {
      document.querySelectorAll(".chip").forEach((c) => c.classList.remove("active"));
      chip.classList.add("active");
      state.filter = chip.dataset.filter;
      render();
    });
  });

  // 表头排序
  document.querySelectorAll(".sortable").forEach((th) => {
    th.addEventListener("click", () => {
      const key = th.dataset.sort;
      if (state.sortKey === key) {
        state.sortDir *= -1;
      } else {
        state.sortKey = key;
        state.sortDir = 1;
      }
      render();
    });
  });

  // 行点击：复选框切换选中，其余区域打开详情
  tbody.addEventListener("click", (e) => {
    const check = e.target.closest(".app-check");
    const row = e.target.closest(".app-row");
    if (!row) return;
    const name = row.dataset.name;
    if (check) {
      toggleSelect(name, check.checked);
      return;
    }
    const app = state.apps.find((a) => a.name === name);
    if (app) openDetail(app);
  });

  // 全选（后续可在表头加入）
  loadApps();
}

function toggleSelect(name, checked) {
  if (checked) state.selected.add(name);
  else state.selected.delete(name);
  render();
}

function openDetail(app) {
  const panel = document.getElementById("detail-panel");
  const mask = document.getElementById("detail-mask");
  const body = document.getElementById("detail-body");

  const rows = [
    ["名称", app.name],
    ["发布者", app.publisher || "—"],
    ["版本", app.version || "—"],
    ["大小", fmtSize(app.size_mb)],
    ["安装日期", fmtDate(app.install_date)],
    ["安装位置", app.install_location || "—"],
    ["来源", app.source === "uwp" ? "UWP / MSIX" : "注册表"],
    ["架构", app.is_64bit == null ? "—" : app.is_64bit ? "64 位" : "32 位"],
    ["注册表键", app.registry_key || "—"],
    ["卸载命令", app.uninstall_string || app.fallback_uninstall_string || "—"],
    ["状态", app.uninstaller_missing ? "无可用卸载器" : app.is_system_component ? "系统组件" : "可卸载"],
  ];

  body.innerHTML = rows
    .map(([k, v]) => `<div class="detail-row"><span class="detail-k">${k}</span><span class="detail-v">${escapeHtml(v)}</span></div>`)
    .join("");

  // 操作按钮
  const actions = document.createElement("div");
  actions.className = "detail-actions";
  if (appCanUninstall(app)) {
    const btn = document.createElement("button");
    btn.className = "btn btn-danger";
    btn.textContent = "完美卸载";
    btn.addEventListener("click", () => openUninstall(app));
    actions.appendChild(btn);
  }
  const forceBtn = document.createElement("button");
  forceBtn.className = "btn btn-ghost";
  forceBtn.textContent = "强制卸载";
  forceBtn.addEventListener("click", () => openUninstall(app));
  actions.appendChild(forceBtn);
  body.appendChild(actions);

  panel.classList.add("open");
  mask.classList.add("show");
}

export function closeDetail() {
  document.getElementById("detail-panel").classList.remove("open");
  document.getElementById("detail-mask").classList.remove("show");
}
