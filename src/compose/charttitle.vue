<script setup lang="ts">
import { computed } from 'vue'
import { useSimulation } from '../composables/useSimulation'

const sim = useSimulation()

const summaryCards = computed(() => {
  const results = sim.rawResults.value
  if (!results.length) return []

  return results.slice(0, 2).map(r => {
    const peak = Math.max(...r.concentrations, 0)
    const peakIdx = r.concentrations.indexOf(peak)
    const peakTime = peakIdx >= 0 ? r.time_h[peakIdx]! : 0

    const days = Math.floor(peakTime / 24)
    const hrs = (peakTime % 24).toFixed(0)
    const timeLabel = days > 0 ? `${days}d${hrs}h` : `${hrs}h`

    return {
      name: r.drug_name,
      peak: peak.toFixed(1),
      unit: r.display_unit || '',
      timeLabel,
    }
  })
})

function formatPeak(peak: string, unit: string): string {
  const val = parseFloat(peak)
  if (val >= 1000) return (val / 1000).toFixed(2) + ' ' + unit.replace(/^./, 'µ')
  if (val >= 1) return parseFloat(peak).toFixed(1) + ' ' + unit
  return parseFloat(peak).toFixed(2) + ' ' + unit
}
</script>

<template>
  <div v-if="summaryCards.length">
    <div style="display: flex; flex-direction: row; gap: 8px; margin-top: 8px">
      <mdui-card
        v-for="card in summaryCards"
        :key="card.name"
        variant="filled"
        style="flex: 1; min-height: 80px; padding: 12px;"
      >
        <span style="
          border-bottom: 1px solid rgb(var(--mdui-color-primary));
          font-weight: 500;
          font-size: 0.8125rem;
        ">{{ card.name }}</span>
        <p style="margin:4px 0 0 0;font-size:1.125rem;font-weight:600;">
          {{ formatPeak(card.peak, card.unit) }}
        </p>
        <p style="margin:0;font-size:0.6875rem;color:rgb(var(--mdui-color-on-surface-variant))">
          峰值 · {{ card.timeLabel }}
        </p>
      </mdui-card>
    </div>
  </div>
</template>
