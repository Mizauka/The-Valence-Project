<script setup lang="ts">
import { vLit } from "../composables/useLitProps";
import { useSplitPane } from "../composables/useSplitPane";
import drugTree from "../compose/drugTree.vue";
import drugEditor from "../compose/drugEditor.vue";
import addPage from "./addPage.vue";

const { width, paneRef, minLeftPaneWidth, minRightPaneWidth, splitProps } = useSplitPane({
  initValue: 50,
});
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
        <drugTree />
      </div>
      <m3e-fab
        variant="primary"
        size="medium"
        style="position: absolute; right: 16px; bottom: 96px"
      >
        <m3e-bottom-sheet-trigger for="lib-add-sheet">
          <m3e-icon name="add"></m3e-icon>
        </m3e-bottom-sheet-trigger>
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
        <drugEditor />
      </div>
    </div>
  </m3e-split-pane>
  <div v-else style="width: 100%; overflow: auto">

  </div>

  <m3e-bottom-sheet
    id="lib-add-sheet"
    modal
    v-lit="{ handle: true, detents: ['full'], detent: 0 }"
  >
    <div style="padding: 16px">
      <h3 style="margin: 0 0 8px 0">添加物质</h3>
      <addPage />
    </div>
  </m3e-bottom-sheet>
</template>
