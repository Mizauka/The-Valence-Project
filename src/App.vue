<template>
  <mdui-layout class="app-layout">
    <mdui-navigation-drawer
      v-if="isDesktop"
      :open="drawerOpen"
      close-on-overlay-click
      @closed="drawerOpen = false"
    >
      <mdui-divider></mdui-divider>
      <mdui-list>
        <mdui-list-item
          v-for="item in navItems"
          :key="item.route"
          :icon="item.icon"
          :active="currentRoute === item.route"
          @click="navigateTo(item.route)"
        >
          {{ item.label }}
        </mdui-list-item>
      </mdui-list>
    </mdui-navigation-drawer>

    <mdui-layout-main class="main-content">
      <mdui-top-app-bar>
        <mdui-button-icon v-if="isDesktop" icon="menu" @click="drawerOpen = !drawerOpen"></mdui-button-icon>
        <mdui-top-app-bar-title>
          <div class="app-title">
            <mdui-icon name="medication" class="app-title-icon"></mdui-icon>
            <span>Valence</span>
          </div>
        </mdui-top-app-bar-title>
      </mdui-top-app-bar>

      <div class="page-container">
        <router-view />
      </div>

      <mdui-bottom-app-bar v-if="!isDesktop" class="mobile-nav">
        <mdui-navigation-bar :value="currentRoute" @change="onMobileNavChange">
          <mdui-navigation-bar-item
            v-for="item in navItems"
            :key="item.route"
            :value="item.route"
            :icon="item.icon"
          >
            {{ item.label }}
          </mdui-navigation-bar-item>
        </mdui-navigation-bar>
      </mdui-bottom-app-bar>
    </mdui-layout-main>
  </mdui-layout>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'

const router = useRouter()
const route = useRoute()

const drawerOpen = ref(true)
const windowWidth = ref(window.innerWidth)

const isDesktop = computed(() => windowWidth.value >= 768)
const currentRoute = computed(() => route.name)

const navItems = [
  { route: 'home', label: '首页', icon: 'home' },
  { route: 'add-dose', label: '记录剂量', icon: 'add_circle' },
  { route: 'history', label: '记录', icon: 'history' },
  { route: 'drug-library', label: '药物库', icon: 'medication' },
  { route: 'calibration', label: '校准', icon: 'tune' },
  { route: 'settings', label: '设置', icon: 'settings' },
]

function navigateTo(name) {
  router.push({ name })
}

function onMobileNavChange(e) {
  const value = e.target.value
  if (value) navigateTo(value)
}

function onResize() {
  windowWidth.value = window.innerWidth
}

onMounted(() => {
  window.addEventListener('resize', onResize)
})

onUnmounted(() => {
  window.removeEventListener('resize', onResize)
})
</script>

<style scoped>
.app-layout {
  height: 100vh;
  width: 100vw;
}

.app-title {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 20px;
  font-weight: 600;
  letter-spacing: 0.5px;
}

.app-title-icon {
  font-size: 24px;
  color: var(--mdui-color-primary);
}

.main-content {
  display: flex;
  flex-direction: column;
  height: 100vh;
}

.page-container {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
  padding-bottom: 80px;
}

.mobile-nav {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  z-index: 100;
}

@media (min-width: 768px) {
  .page-container {
    padding-bottom: 16px;
  }
}
</style>
