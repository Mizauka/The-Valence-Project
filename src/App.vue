<template>
  <div class="app-shell">
    <mdui-navigation-rail contained>

      <mdui-button-icon icon="arrow_back" slot="top"  @click="navigateTo('home')" ></mdui-button-icon>
      <mdui-fab lowered icon="add" slot="top" :class="['nav-fab', { 'nav-fab--active': currentRoute === 'add-dose' }]"
        @click="navigateTo('add-dose')"></mdui-fab>

      <mdui-navigation-rail-item v-for="item in navItems" :key="item.route" :icon="item.icon"
        :active-icon="item.icon + '-filled'" :active="currentRoute === item.route" @click="navigateTo(item.route)">{{
          item.label
        }}</mdui-navigation-rail-item>

      <mdui-button-icon icon="settings" slot="bottom" @click="navigateTo('settings')"></mdui-button-icon>
    </mdui-navigation-rail>

    <div class="content-area">
      <div class="page-container">
        <router-view />
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'

const router = useRouter()
const route = useRoute()

const currentRoute = computed(() => route.name)

const navItems = [
  { route: 'home', label: '首页', icon: 'home--outlined' },
  { route: 'history', label: '记录', icon: 'history--outlined' },
  { route: 'drug-library', label: '药物库', icon: 'medication--outlined' },
  { route: 'calibration', label: '校准', icon: 'tune--outlined' },
]

function navigateTo(name) {
  router.push({ name })
}
</script>

<style scoped>

.nav-fab {
  border-radius: var(--mdui-shape-corner-large);
  box-shadow: var(--mdui-elevation-level3);
  transition: all var(--mdui-motion-duration-medium4)
    var(--mdui-motion-easing-emphasized-decelerate) !important;
}

.nav-fab:hover {
  box-shadow: var(--mdui-elevation-level5);
}
.nav-fab:active {
  box-shadow: var(--mdui-elevation-level0) !important;
  border-radius: 40% !important;
}

.nav-fab--active {
  box-shadow: var(--mdui-elevation-level0) !important;
  background: rgb(var(--mdui-color-primary));
  border-radius: 50% !important;
  color: #fff;
}

</style>
