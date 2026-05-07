<template>
  <div class="sliding-page" :data-step="panels.step">
    <div class="sliding-viewport">
      <div class="sliding-track"
        :style="{ width: panels.trackWidthPercent + '%', transform: `translateX(-${panels.offsetPercent}%)` }">
        <div class="sliding-panel">
          <div class="sliding-panel-inner">
            <div class="header-actions" v-if="hasData">
              <mdui-dropdown>
                <mdui-button slot="trigger" variant="filled" icon="download">导出</mdui-button>
                <mdui-menu>
                  <mdui-menu-item @click="exportCSV">CSV</mdui-menu-item>
                  <mdui-menu-item @click="exportJSON">JSON</mdui-menu-item>
                </mdui-menu>
              </mdui-dropdown>
            </div>

            <div class="side-stat" v-if="hasData">
              <mdui-card variant="elevated" class="dose-stat">
                <mdui-icon name="event" class="side-icon" />
                <div><span class="side-num">{{ doseCount }}</span><span class="side-sub">给药记录</span></div>
              </mdui-card>
            </div>

            <div v-if="hasData" ref="chartContainer" class="chart-container"></div>

            <div v-if="!hasData" class="chart-placeholder">
              <mdui-icon name="show_chart" class="placeholder-icon" />
              <p>{{ placeholderText }}</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, watch } from 'vue'
import { useRoute } from 'vue-router'
import { useSlidingPanels } from '../composables/useSlidingPanels'
import { useHomeChart } from '../composables/useHomeChart'

const route = useRoute()
const panels = useSlidingPanels(1)

const { chartContainer, doseCount, hasData, placeholderText, renderChart, dispose, exportCSV, exportJSON } = useHomeChart()

onMounted(() => renderChart())
onUnmounted(() => dispose())
watch(() => route.path, async p => { if (p === '/') { await renderChart() } })
</script>
