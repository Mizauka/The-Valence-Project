<script setup lang="ts">
import { onMounted } from 'vue'
import { useCalibration } from '../composables/useCalibration'
import type { CalibrationModelType } from '../composables/useCalibration'

const { model, setModel, load } = useCalibration()

onMounted(() => {
  load()
})
</script>

<template>
  <div style="padding: 8px 12px">
    <p style="font-size: 0.8125rem; color: rgb(var(--mdui-color-on-surface-variant)); margin-bottom: 8px;">
      选择后验校准算法以优化血药浓度预测
    </p>
    <m3e-segmented-button>
      <m3e-button-segment
        :checked="model === 'ratio'"
        @click="setModel('ratio')"
      >比率插值</m3e-button-segment>
      <m3e-button-segment
        :checked="model === 'ou-kalman'"
        @click="setModel('ou-kalman')"
      >OU-卡尔曼</m3e-button-segment>
    </m3e-segmented-button>
    <div style="margin-top: 8px; font-size: 0.8125rem; color: rgb(var(--mdui-color-on-surface-variant));">
      <p v-if="model === 'ratio'">
        基于实测值与预测值的比率构建分段线性插值器，适用于少量观测数据。
      </p>
      <p v-else>
        基于 Ornstein-Uhlenbeck 过程的卡尔曼滤波器 + RTS
        反向平滑器，适用于有规律的采血检测，提供置信区间。
      </p>
    </div>
  </div>
</template>
