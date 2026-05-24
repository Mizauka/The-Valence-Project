<script setup lang="ts">
import { vLit } from "../composables/useLitProps";
import { useSplitPane } from "../composables/useSplitPane";
import chart from "../compose/chart.vue";
import history from "../compose/history.vue";
import charttitle from "../compose/charttitle.vue";
import addPage from "./addPage.vue";

const { width, paneRef, minLeftPaneWidth, minRightPaneWidth, splitProps } =
  useSplitPane();
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
        <history />
      </div>
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
        <div style="padding:8px"><chart /><charttitle /></div>
      </div>
    </div>
  </m3e-split-pane>
  <div v-else style="width: 100%; overflow: auto">
    <div style="padding: 8px">
      <chart />
      <charttitle />
      <history />
      <p style="height: 120px"></p>
    </div>
  </div>
</template>
