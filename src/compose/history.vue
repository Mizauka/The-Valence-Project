<script setup lang="ts">
import { onMounted } from 'vue'
import { vLit } from '../composables/useLitProps'
import { useDoseHistory } from '../composables/useDoseHistory'

const { groups, loading, load, selectDose, doseDescription } = useDoseHistory()

onMounted(() => {
  load()
})

defineExpose({ load })
</script>

<template>
  <div v-if="loading" style="display:flex;align-items:center;justify-content:center;padding:32px">
    <mdui-circular-progress indeterminate></mdui-circular-progress>
  </div>

  <div v-else-if="!groups.length" style="padding:32px;text-align:center;color:rgb(var(--mdui-color-on-surface-variant))">
    <p style="font-size:0.875rem">暂无用药记录</p>
    <p style="font-size:0.75rem;margin-top:4px">点击右下角 + 添加第一条记录</p>
  </div>

  <mdui-list v-else>
    <template v-for="group in groups" :key="group.date">
      <mdui-list-subheader>{{ group.date }}</mdui-list-subheader>
      <mdui-list-item
        v-for="dose in group.doses"
        :key="dose.dose_id"
        icon="medication"
        end-icon="edit"
        rounded
        @click="selectDose(dose.dose_id)"
      >
        <m3e-bottom-sheet-trigger for="dose-editor">
          {{ dose.drugName }}
        </m3e-bottom-sheet-trigger>
        <span slot="description">{{ doseDescription(dose) }}</span>
      </mdui-list-item>
    </template>
  </mdui-list>
</template>
