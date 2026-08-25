// Tauri Uninstaller — 主入口（导航切换 + 顶栏交互 + 软件列表视图）
import { initSoftwareView, closeDetail } from "./software-view.js";

const VIEW_TITLES = {
  software: "软件管理",
  filesearch: "文件搜索",
  leftovers: "残留清理",
  settings: "设置",
};

document.addEventListener("DOMContentLoaded", () => {
  const navItems = document.querySelectorAll(".nav-item");
  const views = document.querySelectorAll(".view");
  const pageTitle = document.getElementById("page-title");

  // 导航切换
  navItems.forEach((item) => {
    item.addEventListener("click", () => {
      navItems.forEach((n) => n.classList.remove("active"));
      views.forEach((v) => v.classList.remove("active"));
      item.classList.add("active");
      const view = item.dataset.view;
      document.getElementById(`view-${view}`)?.classList.add("active");
      pageTitle.textContent = VIEW_TITLES[view] || view;
      closeDetail();
    });
  });

  // 初始化软件管理视图（搜索框、刷新按钮共用顶栏元素）
  initSoftwareView({
    searchInput: document.getElementById("search-input"),
    refreshBtn: document.getElementById("btn-refresh"),
  });

  // 详情面板关闭
  document.getElementById("detail-mask").addEventListener("click", closeDetail);
  document.getElementById("detail-close").addEventListener("click", closeDetail);
});
