import { createRouter, createWebHashHistory } from 'vue-router'

const routes = [
  { path: '/', name: 'home', component: () => import('../pages/HomePage.vue') },
  { path: '/add-dose', name: 'add-dose', component: () => import('../pages/AddDosePage.vue') },
  { path: '/history', name: 'history', component: () => import('../pages/HistoryPage.vue') },
  { path: '/drug-library', name: 'drug-library', component: () => import('../pages/DrugLibraryPage.vue') },
  { path: '/calibration', name: 'calibration', component: () => import('../pages/CalibrationPage.vue') },
  { path: '/settings', name: 'settings', component: () => import('../pages/SettingsPage.vue') },
]

const router = createRouter({
  history: createWebHashHistory(),
  routes,
})

export default router
