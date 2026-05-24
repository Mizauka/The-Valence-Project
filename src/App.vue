<script setup lang="ts">
import { ref, computed, nextTick, watch, onMounted } from "vue";
import { useRouter, useRoute } from "vue-router";
import { useBreakpoint } from "./composables/useBreakpoint";
import { useTheme } from "./composables/useTheme";
import { useEngine } from "./composables/useEngine";
import { navItems, routeMeta } from "./composables/routes";
import type { RouteMeta } from "./composables/routes";
import navrail from "./compose/app/navrail.vue";
import bottombar from "./compose/app/bottombar.vue";
import topappbar from "./compose/app/topappbar.vue";
import settingsPage from "./pages/settingsPage.vue";

const { isPhone, isDesktop } = useBreakpoint();
const { themeMode, seedColor } = useTheme();
const { isReady, isLoading, error, init } = useEngine();

// Initialize WASM engine on mount
onMounted(() => {
  init();
});

const effectiveColorScheme = computed(() =>
  themeMode.value === "auto" ? null : themeMode.value,
);

const router = useRouter();
const route = useRoute();

const activeTab = computed(() => (route.name as string) ?? "");
const settingsMeta: RouteMeta = {
  name: "settings",
  icon: "settings",
  title: "设置",
};

const drawerOpen = ref(false);
const drawerModal = computed(() => !isDesktop.value);

// 响应断点变化：桌面端展开、移动端收起
watch(isDesktop, (desktop) => {
  drawerOpen.value = desktop;
});

// 首次渲染时 mdui navigation-drawer hasUpdated=false 会跳过 updateLayout
// 用 nextTick 延迟到 layout 就绪后再打开
nextTick(() => {
  if (isDesktop.value) {
    drawerOpen.value = true;
  }
});

function navigate(name: string) {
  router.push({ name });
}

function toggleDrawer() {
  drawerOpen.value = !drawerOpen.value;
}
</script>

<template>
  <m3e-theme
    :scheme="effectiveColorScheme"
    :color="seedColor"
    style="display: block; height: 100%; overflow: hidden"
  >
    <mdui-layout style="height: 100%; overflow: hidden">
      <mdui-layout-item placement="left" order="-2">
        <navrail
          :drawer-open="drawerOpen"
          :navigator="navigate"
          :nav-items="navItems"
          @toggle-drawer="toggleDrawer"
          v-if="!isPhone"
        />
      </mdui-layout-item>

      <mdui-layout-main style="height: 100%; overflow: hidden" order="1">
        <mdui-layout style="height: 100%">
          <mdui-layout-main style="height: 100%; overflow: hidden">
            <div style="height: 100%; overflow: auto" id="app-view">
              <topappbar :meta="routeMeta[activeTab]" />
              <router-view />
              <bottombar
                :active-tab="activeTab"
                :navigator="navigate"
                :nav-items="navItems"
                v-if="isPhone"
              />
            </div>
          </mdui-layout-main>
        </mdui-layout>
      </mdui-layout-main>

      <mdui-navigation-drawer
        v-if="!isPhone"
        contained
        order="-1"
        placement="right"
        :open="drawerOpen"
        :modal="drawerModal"
        close-on-esc
        close-on-overlay-click
      >
        <div
          style="position: relative; overflow: hidden; height: 100%"
          @pointerdown.stop
          @mousedown.stop
          @click.stop
        >
          <mdui-layout style="height: 100%">
            <topappbar :meta="settingsMeta" scroll-target="#settings-view" />
            <mdui-layout-main style="height: 100%; overflow: hidden">
              <div style="height: 100%; overflow: auto" id="settings-view">
                <settingsPage />
              </div>
            </mdui-layout-main>
          </mdui-layout>
        </div>
      </mdui-navigation-drawer>
    </mdui-layout>
  </m3e-theme>
</template>
