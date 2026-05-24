<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { useChart } from '../composables/useChart'
import { useBreakpoint } from '../composables/useBreakpoint'

const { isPhone } = useBreakpoint()
const containerRef = ref<HTMLElement | null>(null)
const { sim, timeWindow, render, setTimeWindow, dispose } = useChart(containerRef)

watch(timeWindow, () => render())

onMounted(() => {
  setTimeout(() => render(), 500) // Wait for WASM engine init
})

onUnmounted(() => {
  dispose()
})
</script>

<template>
  <div>
    <mdui-layout>
      <mdui-layout-item placement="bottom">
        <m3e-segmented-button>
          <m3e-button-segment :checked="timeWindow === 'day'" @click="setTimeWindow('day')">日</m3e-button-segment>
          <m3e-button-segment :checked="timeWindow === 'week'" @click="setTimeWindow('week')">周</m3e-button-segment>
          <m3e-button-segment :checked="timeWindow === 'month'" @click="setTimeWindow('month')">月</m3e-button-segment>
        </m3e-segmented-button>
        <div style="flex-grow: 1"></div>
        <m3e-split-button>
          <m3e-button slot="leading-button" @click="render">
            <m3e-icon slot="icon" name="refresh"></m3e-icon>刷新
          </m3e-button>
          <m3e-icon-button slot="trailing-button">
            <m3e-icon name="keyboard_arrow_down"></m3e-icon>
            <m3e-menu-trigger for="chart-menu"></m3e-menu-trigger>
          </m3e-icon-button>
        </m3e-split-button>
        <m3e-menu id="chart-menu" position-x="before">
          <m3e-menu-item>JSON</m3e-menu-item>
          <m3e-menu-item>CSV</m3e-menu-item>
          <m3e-menu-item>TXT</m3e-menu-item>
        </m3e-menu>
      </mdui-layout-item>
      <mdui-layout-main style="margin-bottom: 10px">
        <div
          ref="containerRef"
          style="
            width: 100%;
            height: 300px;
            background: rgb(var(--mdui-color-surface-dim));
            border-radius: var(--mdui-shape-corner-large);
          "
        >
          <div
            v-if="!sim.rawResults.value.length && !sim.loading.value"
            style="
              display: flex;
              align-items: center;
              justify-content: center;
              height: 100%;
              color: rgb(var(--mdui-color-on-surface-variant));
              font-size: 0.875rem;
            "
          >
            {{ sim.error.value || '添加给药记录后，浓度曲线将在此显示' }}
          </div>
          <div
            v-if="sim.loading.value"
            style="
              display: flex;
              align-items: center;
              justify-content: center;
              height: 100%;
              color: rgb(var(--mdui-color-on-surface-variant));
            "
          >
            <mdui-circular-progress indeterminate></mdui-circular-progress>
          </div>
        </div>
      </mdui-layout-main>
    </mdui-layout>
  </div>
</template>
