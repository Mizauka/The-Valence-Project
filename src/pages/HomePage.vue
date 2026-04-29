<template>
  <div class="page home-page">
    <div class="page-header">
      <h2 class="page-title">血药浓度追踪</h2>
      <div v-if="hasData" class="chart-actions">
        <mdui-button variant="text" @click="exportCSV">
          <mdui-icon slot="icon" name="download"></mdui-icon>
          导出 CSV
        </mdui-button>
        <mdui-button variant="text" @click="exportJSON">
          <mdui-icon slot="icon" name="data_object"></mdui-icon>
          导出 JSON
        </mdui-button>
      </div>
    </div>
    <div class="chart-container">
      <canvas ref="chartCanvas"></canvas>
      <div v-if="!hasData" class="chart-placeholder">
        <mdui-icon name="show_chart"></mdui-icon>
        <p>{{ placeholderText }}</p>
      </div>
    </div>

    <div v-if="hasData" class="time-range-control">
      <div class="range-labels">
        <span class="range-label-start">{{ rangeStartLabel }}</span>
        <span class="range-label-end">{{ rangeEndLabel }}</span>
      </div>
      <mdui-range-slider
        ref="rangeSlider"
        :min="timeRangeMin"
        :max="timeRangeMax"
        :step="rangeStep"
        @input="onRangeInput"
        @change="onRangeChange"
      ></mdui-range-slider>
    </div>

    <mdui-card variant="elevated" class="summary-card">
      <div class="card-content">
        <mdui-icon name="event"></mdui-icon>
        <div class="card-text">
          <span class="card-value">{{ doseCount }}</span>
          <span class="card-label">给药记录</span>
        </div>
      </div>
    </mdui-card>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, nextTick, watch } from 'vue'
import { useRoute } from 'vue-router'
import { Chart, registerables } from 'chart.js'
import * as store from '../wasm/engineStore'

Chart.register(...registerables)

const route = useRoute()
const chartCanvas = ref(null)
const rangeSlider = ref(null)
const doseCount = ref(0)
const hasData = ref(false)
const placeholderText = ref('添加给药记录后，浓度曲线将在此显示')

const timeRangeMin = ref(0)
const timeRangeMax = ref(100)
const rangeStep = ref(1)
const currentRangeStart = ref(0)
const currentRangeEnd = ref(100)

let chartInstance = null
let cachedSimResults = null

const rangeStartLabel = computed(() => formatXLabel(currentRangeStart.value))
const rangeEndLabel = computed(() => formatXLabel(currentRangeEnd.value))

function getViewWindow() {
  const nowSec = Date.now() / 1000
  const startSec = nowSec - 7 * 24 * 3600
  const endSec = nowSec + 14 * 24 * 3600
  return { startMs: startSec * 1000, endMs: endSec * 1000 }
}

function formatXLabel(ms) {
  const d = new Date(ms)
  const m = String(d.getMonth() + 1).padStart(2, '0')
  const day = String(d.getDate()).padStart(2, '0')
  const h = String(d.getHours()).padStart(2, '0')
  const min = String(d.getMinutes()).padStart(2, '0')
  return `${m}/${day} ${h}:${min}`
}

function onRangeInput(e) {
  const val = e.target.value
  if (!val || val.length !== 2) return
  currentRangeStart.value = val[0]
  currentRangeEnd.value = val[1]
  applyRangeToChart()
}

function onRangeChange(e) {
  const val = e.target.value
  if (!val || val.length !== 2) return
  currentRangeStart.value = val[0]
  currentRangeEnd.value = val[1]
  applyRangeToChart()
}

function applyRangeToChart() {
  if (!chartInstance) return
  chartInstance.options.scales.x.min = currentRangeStart.value
  chartInstance.options.scales.x.max = currentRangeEnd.value
  chartInstance.update('none')
}

function formatTooltipValue(val, unit) {
  if (!isFinite(val)) return '\u2014'
  switch (unit) {
    case 'pg/mL': return `${Math.round(val).toLocaleString()} ${unit}`
    case 'ng/mL': return `${val.toFixed(2)} ${unit}`
    case 'µg/mL': return `${val.toFixed(2)} ${unit}`
    case 'mg/L': return `${val.toFixed(3)} ${unit}`
    default: return `${val.toFixed(4)} ${unit}`
  }
}

function autoScaleUnit(rawValues) {
  let maxVal = 0
  for (const v of rawValues) { if (v > maxVal) maxVal = v }
  if (maxVal <= 0) return { factor: 1, unit: 'mg/L' }
  if (maxVal >= 1) return { factor: 1, unit: 'mg/L' }
  if (maxVal >= 0.001) return { factor: 1000, unit: 'µg/mL' }
  if (maxVal >= 0.000001) return { factor: 1000000, unit: 'ng/mL' }
  return { factor: 1e9, unit: 'pg/mL' }
}

async function renderChart() {
  try {
    const engine = await store.getEngine()
    if (!engine) {
      hasData.value = false
      placeholderText.value = '计算引擎加载失败'
      return
    }

    const doses = await store.getAllDoses()

    doseCount.value = doses.length

    if (doses.length === 0) {
      hasData.value = false
      placeholderText.value = '添加给药记录后，浓度曲线将在此显示'
      if (chartInstance) { chartInstance.destroy(); chartInstance = null }
      return
    }

    const simResults = engine.runSimulation()
    cachedSimResults = simResults

    if (!simResults || simResults.length === 0) {
      hasData.value = false
      placeholderText.value = '计算引擎返回空结果'
      return
    }

    hasData.value = true
    await nextTick()
    if (!chartCanvas.value) return

    const datasets = []
    const colors = [
      'rgba(255, 99, 132, 1)',
      'rgba(54, 162, 235, 1)',
      'rgba(75, 192, 192, 1)',
      'rgba(255, 206, 86, 1)',
      'rgba(153, 102, 255, 1)',
      'rgba(255, 159, 64, 1)',
    ]

    for (let i = 0; i < simResults.length; i++) {
      const result = simResults[i]
      if (!result) continue

      const timeH = result.time_h ? Array.from(result.time_h) : []
      const rawConc = result.concentrations ? Array.from(result.concentrations) : []
      const drugName = result.drug_name || `Drug ${i + 1}`

      if (timeH.length === 0 || rawConc.length === 0) continue

      const { factor, unit } = autoScaleUnit(rawConc)
      const points = timeH.map((t, j) => ({
        x: t * 3600 * 1000,
        y: Math.max(0, rawConc[j]) * factor,
      }))

      const color = colors[i % colors.length]
      datasets.push({
        label: drugName,
        data: points,
        borderColor: color,
        backgroundColor: color.replace('1)', '0.1)'),
        borderWidth: 2,
        fill: true,
        tension: 0.3,
        pointRadius: 0,
        unit,
      })
    }

    if (datasets.length === 0) {
      hasData.value = false
      placeholderText.value = '浓度数据为空，请检查药物参数'
      return
    }

    const viewWindow = getViewWindow()
    const totalRangeMs = viewWindow.endMs - viewWindow.startMs
    const stepMs = Math.max(totalRangeMs / 200, 3600000)
    timeRangeMin.value = viewWindow.startMs
    timeRangeMax.value = viewWindow.endMs
    rangeStep.value = stepMs
    currentRangeStart.value = viewWindow.startMs
    currentRangeEnd.value = viewWindow.endMs

    if (chartInstance) {
      chartInstance.destroy()
      chartInstance = null
    }

    chartInstance = new Chart(chartCanvas.value, {
      type: 'line',
      data: { datasets },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        interaction: {
          mode: 'nearest',
          intersect: false,
        },
        plugins: {
          legend: {
            display: true,
            position: 'top',
          },
          tooltip: {
            callbacks: {
              label: (ctx) => {
                const ds = ctx.dataset
                const unit = ds.unit || 'mg/L'
                return `${ds.label}: ${formatTooltipValue(ctx.parsed.y, unit)}`
              },
              title: (items) => {
                if (!items.length) return ''
                return formatXLabel(items[0].parsed.x)
              },
            },
          },
        },
        scales: {
          x: {
            type: 'linear',
            min: viewWindow.startMs,
            max: viewWindow.endMs,
            title: { display: true, text: '时间' },
            ticks: {
              maxTicksLimit: 8,
              callback: function(value) {
                return formatXLabel(value)
              },
            },
          },
          y: {
            title: { display: true, text: '浓度' },
            beginAtZero: true,
          },
        },
      },
    })

    await nextTick()
    if (rangeSlider.value) {
      rangeSlider.value.value = [viewWindow.startMs, viewWindow.endMs]
      rangeSlider.value.labelFormatter = (val) => formatXLabel(val)
    }

    console.log('[HomePage] chart OK, datasets:', datasets.map(d => d.label))
  } catch (e) {
    console.error('[HomePage] renderChart failed:', e)
    hasData.value = false
    placeholderText.value = '数据加载失败: ' + e.message
  }
}

function downloadBlob(content, filename, mime) {
  const blob = new Blob([content], { type: mime })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = filename
  a.click()
  URL.revokeObjectURL(url)
}

function hoursToISO(h) {
  const ms = h * 3600 * 1000
  return new Date(ms).toISOString()
}

function exportCSV() {
  if (!cachedSimResults || cachedSimResults.length === 0) return
  const results = []
  for (const r of cachedSimResults) {
    if (!r) continue
    const rawConc = Array.from(r.concentrations || [])
    const { factor, unit } = autoScaleUnit(rawConc)
    results.push({
      name: r.drug_name || 'unknown',
      unit,
      factor,
      timeH: Array.from(r.time_h || []),
      conc: rawConc,
    })
  }
  if (results.length === 0) return
  const header = ['datetime']
  for (const r of results) header.push(`${r.name} (${r.unit})`)
  const lines = [header.join(',')]
  const maxLen = Math.max(...results.map(r => r.timeH.length))
  for (let i = 0; i < maxLen; i++) {
    const row = [results[0].timeH[i] != null ? hoursToISO(results[0].timeH[i]) : '']
    for (const r of results) {
      const v = r.conc[i]
      row.push(v != null ? (Math.max(0, v) * r.factor).toFixed(4) : '')
    }
    lines.push(row.join(','))
  }
  const ts = new Date().toISOString().slice(0, 10)
  downloadBlob(lines.join('\n'), `valence_chart_${ts}.csv`, 'text/csv')
}

function exportJSON() {
  if (!cachedSimResults || cachedSimResults.length === 0) return
  const payload = []
  for (const r of cachedSimResults) {
    if (!r) continue
    const rawConc = Array.from(r.concentrations || [])
    const timeH = Array.from(r.time_h || [])
    const { factor, unit } = autoScaleUnit(rawConc)
    const scaledConc = rawConc.map(v => Math.max(0, v) * factor)
    const datetime = timeH.map(h => hoursToISO(h))
    payload.push({
      drug_name: r.drug_name,
      display_unit: unit,
      datetime,
      concentrations: scaledConc,
    })
  }
  const ts = new Date().toISOString().slice(0, 10)
  downloadBlob(JSON.stringify(payload, null, 2), `valence_chart_${ts}.json`, 'application/json')
}

onMounted(async () => {
  await renderChart()
})

onUnmounted(() => {
  if (chartInstance) {
    chartInstance.destroy()
    chartInstance = null
  }
})

watch(() => route.path, async (newPath) => {
  if (newPath === '/') {
    await nextTick()
    await renderChart()
  }
})
</script>

<style scoped>
.home-page {
  max-width: 960px;
  margin: 0 auto;
}

.page-title {
  font-size: 24px;
  font-weight: 600;
  margin-bottom: 16px;
}

.page-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.chart-actions {
  display: flex;
  gap: 4px;
}

.chart-container {
  width: 100%;
  height: 320px;
  margin-bottom: 12px;
  background: var(--mdui-color-surface-container);
  border-radius: 16px;
  padding: 16px;
  position: relative;
}

.chart-placeholder {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  opacity: 0.5;
}

.chart-placeholder p {
  font-size: 14px;
  text-align: center;
}

.time-range-control {
  margin-bottom: 16px;
  padding: 0 8px;
}

.range-labels {
  display: flex;
  justify-content: space-between;
  font-size: 12px;
  opacity: 0.7;
  margin-bottom: 4px;
}

.summary-card {
  padding: 16px;
}

.card-content {
  display: flex;
  align-items: center;
  gap: 12px;
}

.card-text {
  display: flex;
  flex-direction: column;
}

.card-value {
  font-size: 28px;
  font-weight: 700;
}

.card-label {
  font-size: 13px;
  opacity: 0.7;
}
</style>
