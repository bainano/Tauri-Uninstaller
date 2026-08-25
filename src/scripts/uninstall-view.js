// M2 卸载链路：确认面板 → 完美卸载 → 残留扫描 → 残留删除
import {
  uninstallApp,
  killUninstallProcess,
  scanResidue,
  removeResidue,
  forceUninstall,
  isElevated,
} from "./api.js";
import { fmtSize, fmtDate } from "./format.js";

const state = {
  app: null,
  outcome: null,
  report: null,
  checked: new Set(),
};

let root = null;

function ensureRoot() {
  if (root) return root;
  root = document.createElement("div");
  root.id = "uninstall-modal";
  root.className = "uninstall-modal";
  root.innerHTML = `
    <div class="uninstall-mask"></div>
    <div class="uninstall-dialog">
      <div class="uninstall-head">
        <div class="uninstall-title"></div>
        <button class="uninstall-close" title="关闭">×</button>
      </div>
      <div class="uninstall-body"></div>
    </div>`;
  document.body.appendChild(root);
  root.querySelector(".uninstall-close").addEventListener("click", close);
  root.querySelector(".uninstall-mask").addEventListener("click", close);
  return root;
}

export function openUninstall(app) {
  state.app = app;
  state.report = null;
  state.outcome = null;
  state.checked.clear();
  ensureRoot();
  const el = root.querySelector(".uninstall-title");
  el.textContent = `卸载：${app.name}`;
  renderConfirm();
  root.classList.add("open");
}

export function closeUninstall() {
  if (!root) return;
  root.classList.remove("open");
}

function setBody(html) {
  root.querySelector(".uninstall-body").innerHTML = html;
}

function escapeHtml(s) {
  return String(s ?? "").replace(/[&<>"']/g, (c) => ({
    "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;",
  })[c]);
}

// 1) 确认面板
function renderConfirm() {
  const app = state.app;
  const danger = app.is_system_component;
  setBody(`
    <div class="uninstall-confirm">
      <div class="confirm-rows">
        <div class="confirm-row"><span>发布者</span><b>${escapeHtml(app.publisher || "—")}</b></div>
        <div class="confirm-row"><span>版本</span><b>${escapeHtml(app.version || "—")}</b></div>
        <div class="confirm-row"><span>大小</span><b>${fmtSize(app.size_mb)}</b></div>
        <div class="confirm-row"><span>安装日期</span><b>${fmtDate(app.install_date)}</b></div>
        <div class="confirm-row"><span>安装位置</span><b>${escapeHtml(app.install_location || "—")}</b></div>
        <div class="confirm-row"><span>卸载器来源</span><b>${app.source === "uwp" ? "UWP / MSIX" : "注册表"}</b></div>
      </div>
      ${danger ? `<div class="uninstall-warn">该条目被标记为系统组件，卸载可能导致系统异常，请谨慎操作。</div>` : ""}
      <div class="uninstall-actions">
        <button class="btn btn-ghost" data-act="cancel">取消</button>
        <button class="btn btn-danger" data-act="perfect">完美卸载</button>
        ${app.registry_key ? `<button class="btn btn-ghost" data-act="force">强制卸载</button>` : ""}
      </div>
    </div>`);
  bindActions();
}

// 2) 执行中
function renderRunning(phaseText) {
  setBody(`
    <div class="uninstall-running">
      <div class="running-spinner"></div>
      <div class="running-text">${phaseText}</div>
      <div class="running-sub">请稍候，卸载器可能需要一些时间</div>
    </div>`);
}

// 3) 执行结果 + 残留扫描入口
function renderOutcome() {
  const o = state.outcome;
  const ok = o.status === "finished";
  const timedOut = o.status === "timed_out";
  const rows = [
    ["状态", ok ? "已完成" : timedOut ? "超时 / 可能挂起" : "执行失败"],
    ["退出码", o.exit_code == null ? "—" : o.exit_code],
    ["等待时长", `${o.waited_secs} 秒`],
  ];
  let extra = "";
  if (timedOut && o.pid) {
    extra = `<div class="uninstall-warn">卸载进程可能仍在运行（PID ${o.pid}）。可先结束进程再继续，或稍后手动检查。</div>`;
  }
  setBody(`
    <div class="uninstall-outcome ${ok ? "ok" : timedOut ? "warn" : "fail"}">
      ${rows.map(([k, v]) => `<div class="confirm-row"><span>${k}</span><b>${escapeHtml(v)}</b></div>`).join("")}
      ${extra}
      ${o.message ? `<div class="outcome-msg">${escapeHtml(o.message)}</div>` : ""}
      <div class="uninstall-actions">
        <button class="btn btn-ghost" data-act="close">关闭</button>
        ${timedOut && o.pid ? `<button class="btn btn-danger" data-act="kill">结束进程</button>` : ""}
        <button class="btn btn-primary" data-act="scan">扫描残留</button>
      </div>
    </div>`);
  bindActions();
}

// 4) 残留扫描结果
function renderResidue() {
  const r = state.report;
  if (!r) return;
  const groups = [
    ["files", "文件残留", r.files],
    ["shortcuts", "快捷方式", r.shortcuts],
    ["registry", "注册表", r.registry],
  ];
  const html = groups
    .map(([kind, label, items]) => {
      if (!items.length) return "";
      return `<div class="residue-group">
        <div class="residue-group-head">
          <span>${label}（${items.length}）</span>
          <button class="btn btn-mini" data-act="check-${kind}">全选</button>
        </div>
        ${items.map((it, idx) => {
          const key = `${kind}-${idx}`;
          const checked = state.checked.has(key) ? "checked" : "";
          return `<label class="residue-item">
            <input type="checkbox" class="residue-check" data-key="${key}" ${checked}>
            <span class="residue-name" title="${escapeHtml(it.path)}">${escapeHtml(it.name)}</span>
            <span class="residue-path">${escapeHtml(it.path)}</span>
            <span class="residue-size">${kind === "files" && it.sizeBytes != null ? fmtSize(Math.max(1, Math.round(it.sizeBytes / 1048576))) : ""}</span>
            ${it.safe ? "" : '<span class="residue-tag">需谨慎</span>'}
          </label>`;
        }).join("")}
      </div>`;
    })
    .join("");

  setBody(`
    <div class="residue-scan">
      <div class="residue-summary">共发现 ${r.files.length + r.shortcuts.length + r.registry.length} 项残留（文件合计约 ${fmtSize(Math.max(1, Math.round(r.total_size_bytes / 1048576)))}）</div>
      ${html || '<div class="residue-none">未发现明显残留</div>'}
      <div class="uninstall-actions">
        <button class="btn btn-ghost" data-act="close">关闭</button>
        <button class="btn btn-danger" data-act="remove">删除选中残留</button>
      </div>
    </div>`);
  bindActions();
  // 复选框绑定
  root.querySelectorAll(".residue-check").forEach((cb) => {
    cb.addEventListener("change", () => {
      if (cb.checked) state.checked.add(cb.dataset.key);
      else state.checked.delete(cb.dataset.key);
    });
  });
  // 全选
  root.querySelectorAll("[data-act^='check-']").forEach((btn) => {
    btn.addEventListener("click", () => {
      const kind = btn.dataset.act.replace("check-", "");
      root.querySelectorAll(`.residue-check`).forEach((cb) => {
        const key = cb.dataset.key;
        if (key.startsWith(kind + "-")) {
          cb.checked = true;
          state.checked.add(key);
        }
      });
    });
  });
}

function bindActions() {
  root.querySelectorAll("[data-act]").forEach((btn) => {
    btn.addEventListener("click", () => onAction(btn.dataset.act));
  });
}

async function onAction(act) {
  const app = state.app;
  switch (act) {
    case "cancel":
    case "close":
      closeUninstall();
      break;
    case "perfect": {
      renderRunning("正在调用原始卸载器…");
      try {
        const outcome = await uninstallApp(app.registry_key || "", { silent: true });
        state.outcome = outcome;
        renderOutcome();
      } catch (e) {
        setBody(`<div class="uninstall-error">${escapeHtml(e.message || String(e))}</div>
          <div class="uninstall-actions"><button class="btn btn-ghost" data-act="close">关闭</button></div>`);
        bindActions();
      }
      break;
    }
    case "kill": {
      if (!state.outcome?.pid) return;
      try {
        await killUninstallProcess(state.outcome.pid);
        state.outcome.status = "finished";
        state.outcome.message = "已强制结束卸载进程";
        renderOutcome();
      } catch (e) {
        setBody(`<div class="uninstall-error">${escapeHtml(e.message || String(e))}</div>
          <div class="uninstall-actions"><button class="btn btn-ghost" data-act="close">关闭</button></div>`);
        bindActions();
      }
      break;
    }
    case "scan": {
      renderRunning("正在扫描残留…");
      try {
        const report = await scanResidue(app.registry_key || "");
        state.report = report;
        // 默认勾选全部文件与快捷方式，注册表不默认勾选
        state.checked.clear();
        report.files.forEach((_, i) => state.checked.add(`files-${i}`));
        report.shortcuts.forEach((_, i) => state.checked.add(`shortcuts-${i}`));
        renderResidue();
      } catch (e) {
        setBody(`<div class="uninstall-error">${escapeHtml(e.message || String(e))}</div>
          <div class="uninstall-actions"><button class="btn btn-ghost" data-act="close">关闭</button></div>`);
        bindActions();
      }
      break;
    }
    case "remove": {
      if (!state.checked.size) return;
      const r = state.report;
      const items = [];
      r.files.forEach((it, i) => { if (state.checked.has(`files-${i}`)) items.push(it); });
      r.shortcuts.forEach((it, i) => { if (state.checked.has(`shortcuts-${i}`)) items.push(it); });
      r.registry.forEach((it, i) => { if (state.checked.has(`registry-${i}`)) items.push(it); });
      renderRunning("正在删除选中残留…");
      try {
        const res = await removeResidue(items);
        const lines = [
          `成功删除 ${res.ok.length} 项`,
          res.failed.length ? `失败 ${res.failed.length} 项` : "",
        ].filter(Boolean);
        const failedHtml = res.failed.length
          ? `<div class="residue-fail">${res.failed.map(([p, e]) => `<div>${escapeHtml(p)} — ${escapeHtml(e)}</div>`).join("")}</div>`
          : "";
        setBody(`
          <div class="uninstall-outcome ok">
            <div class="outcome-msg">${lines.join("，")}</div>
            ${failedHtml}
            <div class="uninstall-actions">
              <button class="btn btn-ghost" data-act="close">关闭</button>
              <button class="btn btn-primary" data-act="scan">重新扫描</button>
            </div>
          </div>`);
        bindActions();
      } catch (e) {
        setBody(`<div class="uninstall-error">${escapeHtml(e.message || String(e))}</div>
          <div class="uninstall-actions"><button class="btn btn-ghost" data-act="close">关闭</button></div>`);
        bindActions();
      }
      break;
    }
    case "force": {
      renderRunning("正在强制卸载（删除注册表项）…");
      try {
        const res = await forceUninstall(app.registry_key || "", true);
        const lines = [
          `注册表项${res.ok.some((p) => p.startsWith("HKLM") || p.startsWith("HKCU")) ? "已删除" : "未处理"}`,
          res.failed.length ? `失败 ${res.failed.length} 项` : "",
        ].filter(Boolean);
        const failedHtml = res.failed.length
          ? `<div class="residue-fail">${res.failed.map(([p, e]) => `<div>${escapeHtml(p)} — ${escapeHtml(e)}</div>`).join("")}</div>`
          : "";
        setBody(`
          <div class="uninstall-outcome ${res.failed.length ? "warn" : "ok"}">
            <div class="outcome-msg">${lines.join("，")}</div>
            ${failedHtml}
            <div class="uninstall-warn">若卸载项位于 HKLM，可能需要管理员权限。可右键本程序"以管理员身份运行"后重试。</div>
            <div class="uninstall-actions"><button class="btn btn-ghost" data-act="close">关闭</button></div>
          </div>`);
        bindActions();
      } catch (e) {
        setBody(`<div class="uninstall-error">${escapeHtml(e.message || String(e))}</div>
          <div class="uninstall-actions"><button class="btn btn-ghost" data-act="close">关闭</button></div>`);
        bindActions();
      }
      break;
    }
  }
}

// 供详情面板调用：判断是否有卸载能力
export function appCanUninstall(app) {
  return !!(app.uninstall_string || app.fallback_uninstall_string);
}
