<template>
  <div class="dose-form" style="width: 100%;">
    <mdui-card variant="" class="form-card" style="width: 100%;">
      <div class="form-content">
        <div class="selected-drug-banner" v-if="drug">
          <mdui-icon name="medication" class="banner-icon"></mdui-icon>
          <div class="banner-info">
            <span class="banner-name">{{ drug.name || drug.drugName }}</span>
            <span class="banner-detail">
              {{ modelLabel(drug.model_type) }}
              <template v-if="(drug.parameters as any)?.equivalence_factor">· 等效系数={{ (drug.parameters as any).equivalence_factor }}</template>
              · t½={{ (drug.parameters as any)?.half_life || '?' }}h
            </span>
          </div>
        </div>

        <mdui-text-field
          :value="doseAmount" :label="'剂量 (' + currentDoseUnit + ')'"
          type="number" variant="outlined" @input="(e: any) => $emit('update:doseAmount', e.target.value)"
        ></mdui-text-field>

        <mdui-select :value="route" label="给药方式" variant="outlined" @change="(e: any) => $emit('update:route', e.target.value)">
          <mdui-menu-item v-for="r in routes" :key="r.route" :value="r.route">{{ routeLabel(r.route) }}</mdui-menu-item>
        </mdui-select>

        <mdui-text-field
          :value="timestamp" label="给药时间" type="datetime-local" variant="outlined"
          @input="(e: any) => $emit('update:timestamp', e.target.value)"
        ></mdui-text-field>

        <mdui-button variant="filled" full-width :disabled="!canSave" @click="$emit('save')">
          {{ saveLabel }}
        </mdui-button>
      </div>
    </mdui-card>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { modelLabel, routeLabel } from '../utils/format'

const props = defineProps<{
  drug: any
  doseAmount: string
  route: string
  timestamp: string
  routes: { route: string; unit: string }[]
  currentDoseUnit: string
  canSave: boolean
  saveLabel?: string
}>()

defineEmits<{
  'update:doseAmount': [value: string]
  'update:route': [value: string]
  'update:timestamp': [value: string]
  save: []
}>()
</script>
