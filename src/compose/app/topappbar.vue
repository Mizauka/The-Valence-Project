<script setup lang="ts">
import type { RouteMeta } from "../../composables/routes";
import { useBreakpoint } from "../../composables/useBreakpoint";
import { useSettings } from "../../composables/useSettings";

const { isPhone } = useBreakpoint();
const { settings } = useSettings();

defineProps<{
  meta?: RouteMeta;
  scrollTarget?: string;
}>();
</script>

<template>
  <mdui-top-app-bar
    :scroll-target="scrollTarget ?? '#app-view'"
    :scroll-behavior="isPhone && settings.topBarShrink ? 'shrink elevate' : 'elevate'"
    :variant="isPhone && settings.topBarShrink ? 'medium' : 'small'"
    style="z-index: 2"
  >
    <m3e-icon-button>
      <m3e-icon :name="meta?.icon ?? 'home'"></m3e-icon>
    </m3e-icon-button>
    <mdui-top-app-bar-title>{{
      meta?.title ?? "药历管理系统"
    }}</mdui-top-app-bar-title>
    <div style="flex-grow: 1"></div>
    <m3e-icon-button>
      <m3e-icon name="more_vert"></m3e-icon>
    </m3e-icon-button>
  </mdui-top-app-bar>
</template>
