import { createRouter, createWebHistory } from "vue-router";

const routes = [
  {
    path: "/overlay",
    name: "overlay",
    component: () => import("@/views/OverlayView.vue"),
  },
  {
    path: "/pin",
    name: "pin",
    component: () => import("@/views/PinView.vue"),
  },
  {
    path: "/settings",
    name: "settings",
    component: () => import("@/views/SettingsView.vue"),
  },
  {
    path: "/history",
    name: "history",
    component: () => import("@/views/HistoryView.vue"),
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;
