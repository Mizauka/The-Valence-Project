<script setup lang="ts">
import { computed } from 'vue'
import { useDrugLibrary } from '../composables/useDrugLibrary'

const { categories, selectedDrugId, getSelectedDrug } = useDrugLibrary()

const drug = computed(() => getSelectedDrug())

const sourceLabel = computed(() => {
  if (!drug.value) return ''
  if (drug.value.source === 'hrt') return 'HRT'
  if (drug.value.source === 'journal') return 'Journal'
  return '自定义'
})

const modelLabel = computed(() => {
  const m = drug.value?.model_type || ''
  switch (m) {
    case 'one_compartment': return '一室模型'
    case 'two_compartment': return '二室模型'
    case 'multi_compartment': return '多室模型'
    default: return m
  }
})
</script>

<template>
  <div style="padding: 8px">
    <!-- No drug selected -->
    <div v-if="!drug" style="
      display: flex; align-items: center; justify-content: center;
      height: 100%; color: rgb(var(--mdui-color-on-surface-variant));
      font-size: 0.875rem;
    ">
      选择左侧药物查看详情
    </div>

    <!-- Drug Detail -->
    <template v-else>
      <m3e-breadcrumb>
        <m3e-breadcrumb-item>物质列表</m3e-breadcrumb-item>
        <m3e-breadcrumb-item>{{ sourceLabel }}</m3e-breadcrumb-item>
        <m3e-breadcrumb-item>{{ drug.name }}</m3e-breadcrumb-item>
      </m3e-breadcrumb>
      <m3e-card>
        <m3e-heading slot="header" variant="display" size="small">
          {{ drug.name }}
        </m3e-heading>
        <div slot="content" style="font-size: 0.875rem; line-height: 1.6;">
          <div style="display: grid; grid-template-columns: auto 1fr; gap: 4px 12px;">
            <span style="color: rgb(var(--mdui-color-on-surface-variant))">ID:</span>
            <span>{{ drug.drug_id }}</span>
            <span style="color: rgb(var(--mdui-color-on-surface-variant))">模型:</span>
            <span>{{ modelLabel }}</span>
            <span style="color: rgb(var(--mdui-color-on-surface-variant))">来源:</span>
            <span>{{ sourceLabel }}</span>
            <span style="color: rgb(var(--mdui-color-on-surface-variant))">剂量单位:</span>
            <span>{{ drug.dose_unit }}</span>
            <span style="color: rgb(var(--mdui-color-on-surface-variant))">显示单位:</span>
            <span>{{ drug.display_unit || 'mg/L' }}</span>
            <template v-if="drug.molecular_weight">
              <span style="color: rgb(var(--mdui-color-on-surface-variant))">分子量:</span>
              <span>{{ drug.molecular_weight.toFixed(2) }} g/mol</span>
            </template>
            <template v-if="drug.depot_model">
              <span style="color: rgb(var(--mdui-color-on-surface-variant))">前药模型:</span>
              <span>是</span>
            </template>
          </div>
        </div>
        <div slot="actions" end>
          <m3e-button variant="filled" disabled>编辑</m3e-button>
        </div>
      </m3e-card>

      <!-- Parameters -->
      <m3e-card style="margin-top: 8px;" v-if="drug.parameters && Object.keys(drug.parameters).length">
        <m3e-heading slot="header" variant="title" size="small">PK 参数</m3e-heading>
        <div slot="content" style="font-size: 0.8125rem;">
          <div v-for="(v, k) in drug.parameters" :key="k" style="display: flex; justify-content: space-between; padding: 2px 0; border-bottom: 1px solid rgb(var(--mdui-color-outline-variant))">
            <span>{{ k }}</span>
            <span style="font-family: monospace;">{{ typeof v === 'number' ? v.toPrecision(4) : v }}</span>
          </div>
        </div>
      </m3e-card>
    </template>
  </div>
</template>
