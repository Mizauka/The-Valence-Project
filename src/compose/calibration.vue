<script setup lang="ts">
import { onMounted, computed } from 'vue'
import { useCalibration } from '../composables/useCalibration'

const { labResults, loading, load, removeLab } = useCalibration()

onMounted(() => {
  load()
})

// Group lab results by group_id
const groupedLabs = computed(() => {
  const map = new Map<string, typeof labResults.value>()
  for (const lab of labResults.value) {
    const key = lab.group_id || '全部'
    if (!map.has(key)) map.set(key, [])
    map.get(key)!.push(lab)
  }
  return Array.from(map.entries()).map(([group, labs]) => ({ group, labs }))
})

function formatTime(h: number): string {
  const days = Math.floor(h / 24)
  const hrs = (h % 24).toFixed(1)
  return days > 0 ? `${days}d${hrs}h` : `${hrs}h`
}

defineExpose({ load })
</script>

<template>
  <div v-if="loading" style="display:flex;align-items:center;justify-content:center;padding:32px">
    <mdui-circular-progress indeterminate></mdui-circular-progress>
  </div>

  <div v-else-if="!labResults.length" style="padding:32px;text-align:center;color:rgb(var(--mdui-color-on-surface-variant))">
    <p style="font-size:0.875rem">暂无校准数据</p>
    <p style="font-size:0.75rem;margin-top:4px">添加血药浓度实测值以校准预测曲线</p>
  </div>

  <mdui-list v-else>
    <template v-for="g in groupedLabs" :key="g.group">
      <mdui-list-subheader>{{ g.group === '' ? '全部' : g.group }}</mdui-list-subheader>
      <mdui-list-item
        v-for="(lab, idx) in g.labs"
        :key="idx"
        icon="biotech"
        rounded
      >
        {{ formatTime(lab.time_h) }}
        <span slot="description">{{ lab.conc_value }} {{ lab.unit }}</span>
        <mdui-button-icon slot="end-icon" icon="delete" @click="removeLab(idx)"></mdui-button-icon>
      </mdui-list-item>
    </template>
  </mdui-list>
</template>
