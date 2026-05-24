<script setup lang="ts">
import { vLit } from "../composables/useLitProps";
import { useSplitPane } from "../composables/useSplitPane";
import { usePlan } from "../composables/usePlan";
import plan from "../compose/plan.vue";
import planEditor from "../compose/planEditor.vue";

const { width, paneRef, minLeftPaneWidth, minRightPaneWidth, splitProps } =
  useSplitPane({
    rightFraction: 0.25,
    rightMin: 170,
    initValue: 50,
  });

const { startNew } = usePlan();
</script>

<template>
  <m3e-split-pane
    ref="paneRef"
    v-lit="splitProps"
    style="height: 100%"
    v-if="820 < width"
  >
    <div
      slot="start"
      style="position: relative; height: 100%; overflow-x: hidden; min-width: 0"
    >
      <div
        :style="{
          position: 'absolute',
          top: '0',
          left: '0',
          right: '0',
          bottom: '0',
          minWidth: minLeftPaneWidth,
        }"
      >
        <div style="padding: 8px">
          <m3e-search-bar clearable>
            <m3e-icon name="search" slot="leading"></m3e-icon>
            <input slot="input" placeholder="搜索方案…" />
          </m3e-search-bar>
          <plan />
        </div>
      </div>
      <m3e-fab
        variant="primary"
        size="medium"
        style="position: absolute; right: 16px; bottom: 96px"
        @click="startNew()"
      >
        <m3e-icon name="add"></m3e-icon>
      </m3e-fab>
    </div>
    <div
      slot="end"
      style="position: relative; height: 100%; overflow-x: hidden; min-width: 0"
    >
      <div
        :style="{
          position: 'absolute',
          top: '0',
          left: '0',
          right: '0',
          bottom: '0',
          minWidth: minRightPaneWidth,
        }"
      >
        <planEditor />
      </div>
    </div>
  </m3e-split-pane>
  <div v-else style="width: 100%; overflow: auto">
    <div>
      <m3e-search-bar clearable style="margin-top: 8px">
        <m3e-icon name="search" slot="leading"></m3e-icon>
        <input slot="input" placeholder="搜索方案…" />
      </m3e-search-bar>
      <plan />
      <p style="height: 120px"></p>
    </div>
  </div>
</template>
