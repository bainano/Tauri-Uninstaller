// Tauri Uninstaller — 主入口（M0 壳：导航切换 + 顶栏交互）
import { greet } from "./api.js";

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
    });
  });

  // 刷新按钮（M0 占位）
  document.getElementById("btn-refresh").addEventListener("click", () => {
    console.log("refresh placeholder");
  });

  // 验证 Rust 桥接（仅开发调试用）
  greet("Tauri").then((msg) => console.log(msg)).catch(() => {});
});
