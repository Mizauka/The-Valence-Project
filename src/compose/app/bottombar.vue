<script setup lang="ts">
import type { RouteMeta } from "../../composables/routes";

const props = defineProps<{
  activeTab: string
  navigator: (name: string) => void
  navItems: RouteMeta[]
}>()
</script>

<template>
  <mdui-navigation-bar
    scroll-behavior="hide"
    scroll-threshold="30"
    scroll-target="#app-view"
    :value="activeTab"
  >
    <mdui-navigation-bar-item
      v-for="item in navItems.filter(i => i.name !== 'add' && i.name !== 'plan')"
      :key="item.name"
      :icon="item.icon + '--outlined'"
      :active-icon="item.icon"
      :value="item.name"
      @click="navigator(item.name)"
      >{{ item.title }}</mdui-navigation-bar-item
    >
  </mdui-navigation-bar>
</template>
