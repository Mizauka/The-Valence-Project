<script setup lang="ts">
import { computed } from "vue";
import { useRoute } from "vue-router";
import { useBreakpoint } from "../../composables/useBreakpoint";
import { useNavRailMode } from "../../composables/useNavRailMode";
import { vLit } from "../../composables/useLitProps";
import type { RouteMeta } from "../../composables/routes";

const props = defineProps<{
  drawerOpen: boolean;
  navigator: (name: string) => void;
  navItems: RouteMeta[];
}>();

const emit = defineEmits<{
  'toggle-drawer': [];
}>();

const route = useRoute();
const { width } = useBreakpoint();
const { navRef, isNavExpanded } = useNavRailMode();

const activeTab = computed(() => route.name as string);
</script>

<template>
  <m3e-nav-rail
    id="nav-rail"
    ref="navRef"
    :mode="width > 1300 ? 'expanded' : 'compact'"
  >
    <m3e-icon-button
      toggle
      variant="filled"
      id="menubutton"
      :selected="isNavExpanded"
    >
      <m3e-icon name="menu"></m3e-icon>
      <m3e-icon slot="selected" name="menu_open"></m3e-icon>
      <m3e-nav-rail-toggle for="nav-rail"></m3e-nav-rail-toggle>
    </m3e-icon-button>
    <m3e-fab
      id="interfaceaddfab"
      size="small"
      lowered
      :variant="
        isNavExpanded || activeTab === 'add' ? 'primary' : 'primary-container'
      "
      @click="props.navigator('add')"
    >
      <m3e-icon name="add"></m3e-icon>
      <span slot="label">添加用药</span>
    </m3e-fab>
    <m3e-nav-item
      v-for="item in navItems.filter(
        (i) => i.name !== 'add' && i.name !== 'settings',
      )"
      :key="item.name"
      :selected="activeTab === item.name"
      @click="props.navigator(item.name)"
    >
      <m3e-icon slot="icon" :name="item.icon"></m3e-icon>{{ item.title }}
    </m3e-nav-item>
    <m3e-icon-button
      toggle
      variant="filled"
      style="position: absolute; bottom: 16px"
      v-lit="{ selected: drawerOpen }"
      @input.prevent
      @click="emit('toggle-drawer')"
    >
      <m3e-icon name="settings"></m3e-icon>
    </m3e-icon-button>
  </m3e-nav-rail>
</template>
