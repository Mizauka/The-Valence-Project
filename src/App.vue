<template>
  <!-- Desktop layout: navigation-rail + scroll-target container -->
  <template v-if="!isMobile">
    <mdui-navigation-rail alignment="start" class="app-rail">
      <mdui-button-icon icon="medical_services--outlined" slot="top" @click="navigateTo('home')"></mdui-button-icon>
      <mdui-fab lowered icon="add" slot="top" :class="['nav-fab', { 'nav-fab--active': currentRoute === 'add-dose' }]"
        @click="navigateTo('add-dose')"></mdui-fab>
      <mdui-navigation-rail-item v-for="item in navItems" :key="item.route" :icon="item.icon"
        :active-icon="item.icon + '-filled'" :active="currentRoute === item.route" @click="navigateTo(item.route)">{{
          item.label
        }}</mdui-navigation-rail-item>
      <mdui-button-icon icon="settings--outlined" selected-icon="settings" slot="bottom" @click="navigateTo('settings')"
        :selected="currentRoute === 'settings'"></mdui-button-icon>
    </mdui-navigation-rail>

    <div class="app-desktop-container">
      <mdui-top-app-bar scroll-behavior="elevate" scroll-target=".app-scroll-content" class="app-topbar">
        <mdui-top-app-bar-title>{{ pageTitle }}</mdui-top-app-bar-title>
      </mdui-top-app-bar>
      <div class="app-scroll-content">
        <router-view />
      </div>
    </div>
  </template>

  <!-- Mobile layout: top-app-bar + bottom-app-bar with scroll-target -->
  <template v-else>
    <div class="app-mobile-container">
      <mdui-top-app-bar scroll-behavior="shrink" scroll-target=".app-scroll-content" class="app-topbar">
        <mdui-top-app-bar-title>{{ pageTitle }}</mdui-top-app-bar-title>
      </mdui-top-app-bar>
      <mdui-bottom-app-bar class="app-bottom-bar" scroll-behavior="hide" fab-detach scroll-threshold="30"
        scroll-target=".app-scroll-content">
        <mdui-button-icon v-for="item in navItems"
          :class="['nav-btn', { 'nav-btn--active': currentRoute === item.route }]" :active="currentRoute === item.route"
          :icon="item.icon + '--outlined'" :selected-icon="item.icon + '-filled'" @click="navigateTo(item.route)"
          :selected="currentRoute === item.route">
        </mdui-button-icon>
        <mdui-button-icon icon="settings--outlined" selected-icon="settings"
          :class="['nav-btn', { 'nav-btn--active': currentRoute === 'settings' }]" @click="navigateTo('settings')"
          :selected="currentRoute === 'settings'"></mdui-button-icon>
        <div style="flex-grow:1"></div>
        <mdui-fab icon="add" @click="navigateTo('add-dose')"
          :class="['nav-btn-fab', { 'nav-btn-fab--active': currentRoute === 'add-dose' }]"></mdui-fab>
      </mdui-bottom-app-bar>
      <div class="app-scroll-content">
        <router-view />
      </div>
    </div>
  </template>
</template>

<script setup>
import { computed, ref, onMounted, onUnmounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'

const router = useRouter()
const route = useRoute()

const isMobile = ref(false)
function checkMobile() { isMobile.value = window.innerWidth < 768 }
onMounted(() => { checkMobile(); window.addEventListener('resize', checkMobile) })
onUnmounted(() => window.removeEventListener('resize', checkMobile))

const currentRoute = computed(() => route.name)

const titleMap = {
  home: 'The Valence Project',
  'add-dose': '记录剂量',
  history: '给药记录',
  'drug-library': '药物库',
  calibration: '模型校准',
  settings: '设置',
}
const pageTitle = computed(() => titleMap[currentRoute.value] || 'The Valence Project')

const navItems = [
  { route: 'home', label: '首页', icon: 'home--outlined' },
  { route: 'history', label: '记录', icon: 'history--outlined' },
  { route: 'drug-library', label: '药物库', icon: 'medication--outlined' },
  { route: 'calibration', label: '校准', icon: 'tune--outlined' },
]

function navigateTo(name) { router.push({ name }) }
</script>

<style scoped>
.app-desktop-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  position: relative;
  overflow: hidden;
}

.app-mobile-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  position: relative;
  overflow: hidden;
}

.app-scroll-content {
  flex: 1;
  overflow: auto;
}

.nav-fab {
  border-radius: var(--mdui-shape-corner-large);
  box-shadow: var(--mdui-elevation-level3);
  transition: all var(--mdui-motion-duration-medium4) var(--mdui-motion-easing-emphasized-decelerate) !important;
}

.nav-fab:hover {
  box-shadow: var(--mdui-elevation-level5);
}

.nav-fab:active {
  box-shadow: var(--mdui-elevation-level0) !important;
  border-radius: 40% !important;
}

.nav-fab--active {
  background: rgb(var(--mdui-color-primary));
  border-radius: 50% !important;
  color: #fff;
}

.nav-btn-fab {
  border-radius: var(--mdui-shape-corner-large);

}

.nav-btn-fab:active {
  border-radius: 40% !important;
}

.nav-btn-fab--active {
  background: rgb(var(--mdui-color-primary));
  border-radius: 50% !important;
  color: #fff;
}

.nav-btn {
  background: rgb(var(--mdui-color-primary-container));
  color: rgb(var(--mdui-color-on-primary-container));
  border-radius: 50% !important;

}

.nav-btn:active {
  border-radius: 35% !important;
}

.nav-btn--active {
  background: rgb(var(--mdui-color-primary));
  color: rgb(var(--mdui-color-on-primary));
  border-radius: var(--mdui-shape-corner-medium) !important;
}
</style>
