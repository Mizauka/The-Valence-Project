import { createRouter, createWebHistory } from "vue-router";

const routes = [
  { path: "/", name: "home", component: () => import("../pages/homePage.vue"), meta: { title: "浓度曲线" } },
  { path: "/history", name: "history", component: () => import("../pages/historyPage.vue"), meta: { title: "药历核对" } },
  { path: "/library", name: "library", component: () => import("../pages/libPage.vue"), meta: { title: "物质列表" } },
  { path: "/plan", name: "plan", component: () => import("../pages/planPage.vue"), meta: { title: "用药方案" } },
  { path: "/add", name: "add", component: () => import("../pages/addPage.vue"), meta: { title: "添加记录" } },
  { path: "/settings", name: "settings", component: () => import("../pages/settingsPage.vue"), meta: { title: "设置" } },
];

// 与 vite.config.ts 的 base 保持一致（仓库名）
const BASE = "/reValenceGUI/";

const router = createRouter({
  history: createWebHistory(BASE),
  routes,
});

export default router;
export { BASE };

