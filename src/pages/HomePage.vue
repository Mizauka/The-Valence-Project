<template>
  <div class="page home-page">
    <div class="page-header">
      <h2 class="page-title">血药浓度追踪</h2>
      <div class="header-actions" v-if="hasData">
        <mdui-dropdown>
          <mdui-button slot="trigger" variant="filled" icon="download">
            导出
          </mdui-button>
          <mdui-menu>
            <mdui-menu-item @click="exportCSV">CSV</mdui-menu-item>
            <mdui-menu-item @click="exportJSON">JSON</mdui-menu-item>
          </mdui-menu>
        </mdui-dropdown>
      </div>
    </div>

    <div class="main-row" v-if="hasData">
      <div class="side-panel">
        <mdui-card variant="elevated" class="side-stat">
          <mdui-icon name="event" class="side-icon"></mdui-icon>
          <div>
            <span class="side-num">{{ doseCount }}</span>
            <span class="side-sub">给药记录</span>
          </div>
        </mdui-card>
      </div>

      <mdui-card variant="elevated" class="chart-card">
        <div class="chart-card-inner">
          <div class="y-slider-container" ref="ySliderContainer" style="margin-top: 20px;">
            <mdui-slider
              ref="ySliderRef"
              class="y-mdui-slider"
              :min="ySliderMin"
              :max="ySliderMax"
              :step="ySliderStep"
              :value="ySliderMax"
              nolabel
              @input="onYSliderInput"
            ></mdui-slider>
          </div>
          <div class="chart-area">
            <canvas ref="chartCanvas"></canvas>
          </div>
        </div>
        <div class="time-slider-row" style="margin-right: 10px;">
          <mdui-range-slider
            ref="rangeSlider"
            :min="timeRangeMin"
            :max="timeRangeMax"
            :step="rangeStep"
            @input="onRangeInput"
            @change="onRangeChange"
          ></mdui-range-slider>
        </div>
      </mdui-card>
    </div>

    <div class="chart-placeholder-card" v-if="!hasData">
      <mdui-icon name="show_chart" class="placeholder-icon"></mdui-icon>
      <p>{{ placeholderText }}</p>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, nextTick, watch } from 'vue'
import { useRoute } from 'vue-router'
import { Chart, registerables } from 'chart.js'
import * as store from '../wasm/engineStore'

Chart.register(...registerables)

const route = useRoute()
const chartCanvas = ref(null)
const rangeSlider = ref(null)
const ySliderRef = ref(null)
const ySliderContainer = ref(null)
const doseCount = ref(0)
const hasData = ref(false)
const placeholderText = ref('添加给药记录后，浓度曲线将在此显示')

const timeRangeMin = ref(0)
const timeRangeMax = ref(100)
const rangeStep = ref(1)
const currentRangeStart = ref(0)
const currentRangeEnd = ref(100)

const currentYMax = ref(100)
const ySliderMin = ref(50)
const ySliderMax = ref(500)
const ySliderStep = ref(10)

const curveVisibility = ref({})

let chartInstance = null
let cachedSimResults = null
let resizeObserver = null

function calcNiceYMax(maxVal) {
  if (maxVal <= 0) return 50
  const rounded = Math.ceil(maxVal / 50) * 50
  return Math.max(rounded, 50)
}

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

function onYSliderInput(e) {
  const val = parseFloat(e.target.value)
  if (isNaN(val)) return
  currentYMax.value = val
  if (!chartInstance) return
  chartInstance.options.scales.y.max = val
  chartInstance.update('none')
}

function syncYSliderSize() {
  if (!ySliderContainer.value || !ySliderRef.value) return
  const h = ySliderContainer.value.clientHeight
  if (h > 40) {
    ySliderRef.value.style.width = `${h - 8}px`
  }
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

function resolveDisplayUnit(rawValues, simResult) {
  const du = simResult?.display_unit
  if (du && du !== 'mg/L') {
    const v = { 'pg/mL': 1e9, 'ng/mL': 1e6, 'µg/mL': 1000, 'mg/L': 1 }
    return { factor: v[du] || 1, unit: du }
  }
  return autoScaleUnit(rawValues)
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
      { border: '#E91E63', bg: 'rgba(233,30,99,0.08)' },
      { border: '#2196F3', bg: 'rgba(33,150,243,0.08)' },
      { border: '#00BCD4', bg: 'rgba(0,188,212,0.08)' },
      { border: '#FF9800', bg: 'rgba(255,152,0,0.08)' },
      { border: '#9C27B0', bg: 'rgba(156,39,176,0.08)' },
      { border: '#4CAF50', bg: 'rgba(76,175,80,0.08)' },
    ]
    let globalMaxY = 0

    for (let i = 0; i < simResults.length; i++) {
      const r = simResults[i]
      if (!r) continue
      const timeH = r.time_h ? Array.from(r.time_h) : []
      const rawConc = r.concentrations ? Array.from(r.concentrations) : []
      const drugName = r.drug_name || `Drug ${i + 1}`
      if (timeH.length === 0 || rawConc.length === 0) continue

      const { factor, unit } = resolveDisplayUnit(rawConc, r)
      const points = timeH.map((t, j) => ({
        x: t * 3600 * 1000,
        y: Math.max(0, rawConc[j]) * factor,
      }))
      for (const p of points) { if (p.y > globalMaxY) globalMaxY = p.y }

      const c = colors[i % colors.length]
      datasets.push({
        label: drugName,
        data: points,
        borderColor: c.border,
        backgroundColor: c.bg,
        borderWidth: 2.5,
        fill: true,
        tension: 0.35,
        pointRadius: 0,
        pointHoverRadius: 4,
        pointHoverBackgroundColor: c.border,
        unit,
        hidden: curveVisibility.value[drugName] === false,
      })
    }

    if (datasets.length === 0) {
      hasData.value = false
      placeholderText.value = '浓度数据为空，请检查药物参数'
      return
    }

    const niceYMax = calcNiceYMax(globalMaxY)
    currentYMax.value = niceYMax
    ySliderMin.value = 50
    ySliderMax.value = niceYMax
    ySliderStep.value = 1

    const viewWindow = getViewWindow()
    timeRangeMin.value = viewWindow.startMs
    timeRangeMax.value = viewWindow.endMs
    rangeStep.value = Math.max((viewWindow.endMs - viewWindow.startMs) / 200, 3600000)
    currentRangeStart.value = viewWindow.startMs
    currentRangeEnd.value = viewWindow.endMs

    if (chartInstance) { chartInstance.destroy(); chartInstance = null }

    chartInstance = new Chart(chartCanvas.value, {
      type: 'line',
      data: { datasets },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        interaction: { mode: 'nearest', intersect: false },
        plugins: {
          legend: {
            display: true,
            position: 'top',
            align: 'start',
            labels: {
              usePointStyle: true,
              pointStyleWidth: 14,
              padding: 20,
              generateLabels: (chart) => chart.data.datasets.map((ds, i) => ({
                text: `${ds.label} (${ds.unit || 'mg/L'})`,
                fillStyle: ds.borderColor,
                strokeStyle: ds.borderColor,
                lineWidth: 3,
                hidden: !chart.isDatasetVisible(i),
                index: i,
                datasetIndex: i,
                pointStyle: 'circle',
              })),
            },
            onClick: (e, item, legend) => {
              const i = item.datasetIndex
              const ci = legend.chart
              if (ci.isDatasetVisible(i)) {
                ci.hide(i)
                curveVisibility.value[ci.data.datasets[i].label] = false
              } else {
                ci.show(i)
                curveVisibility.value[ci.data.datasets[i].label] = true
              }
            },
          },
          tooltip: {
            padding: 10,
            cornerRadius: 8,
            callbacks: {
              label: (ctx) => ` ${ctx.dataset.label}: ${formatTooltipValue(ctx.parsed.y, ctx.dataset.unit || 'mg/L')}`,
              title: (items) => items.length ? formatXLabel(items[0].parsed.x) : '',
            },
          },
        },
        scales: {
          x: {
            type: 'linear',
            min: viewWindow.startMs,
            max: viewWindow.endMs,
            title: { display: true, text: '时间', color: '#666' },
            grid: { color: 'rgba(0,0,0,0.04)', drawBorder: false },
            ticks: { maxTicksLimit: 8, color: '#999', padding: 8, callback: v => formatXLabel(v) },
          },
          y: {
            title: { display: true, text: '浓度', color: '#666' },
            beginAtZero: true,
            max: niceYMax,
            grid: { color: 'rgba(0,0,0,0.06)', drawBorder: false },
            ticks: { maxTicksLimit: 5, color: '#999', padding: 8 },
          },
        },
      },
    })

    await nextTick()
    if (rangeSlider.value) {
      rangeSlider.value.value = [viewWindow.startMs, viewWindow.endMs]
      rangeSlider.value.labelFormatter = (v) => formatXLabel(v)
    }
    if (ySliderRef.value) ySliderRef.value.value = niceYMax
    await nextTick()
    syncYSliderSize()
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
  a.href = url; a.download = filename; a.click()
  URL.revokeObjectURL(url)
}

function hoursToISO(h) { return new Date(h * 3600 * 1000).toISOString() }

function exportCSV() {
  if (!cachedSimResults?.length) return
  const rs = []
  for (const r of cachedSimResults) {
    if (!r) continue
    const rc = Array.from(r.concentrations || [])
    const { factor, unit } = resolveDisplayUnit(rc, r)
    rs.push({ name: r.drug_name || 'unknown', unit, factor, th: Array.from(r.time_h || []), conc: rc })
  }
  if (!rs.length) return
  const hdr = ['datetime', ...rs.map(r => `${r.name} (${r.unit})`)]
  const lines = [hdr.join(',')]
  const maxLen = Math.max(...rs.map(r => r.th.length))
  for (let i = 0; i < maxLen; i++) {
    const row = [rs[0].th[i] != null ? hoursToISO(rs[0].th[i]) : '']
    for (const r of rs) row.push(r.conc[i] != null ? (Math.max(0, r.conc[i]) * r.factor).toFixed(4) : '')
    lines.push(row.join(','))
  }
  downloadBlob(lines.join('\n'), `valence_chart_${new Date().toISOString().slice(0, 10)}.csv`, 'text/csv')
}

function exportJSON() {
  if (!cachedSimResults?.length) return
  const payload = []
  for (const r of cachedSimResults) {
    if (!r) continue
    const rc = Array.from(r.concentrations || [])
    const th = Array.from(r.time_h || [])
    const { factor, unit } = resolveDisplayUnit(rc, r)
    payload.push({
      drug_name: r.drug_name,
      display_unit: unit,
      datetime: th.map(h => hoursToISO(h)),
      concentrations: rc.map(v => Math.max(0, v) * factor),
    })
  }
  downloadBlob(JSON.stringify(payload, null, 2), `valence_chart_${new Date().toISOString().slice(0, 10)}.json`, 'application/json')
}

onMounted(async () => {
  await renderChart()
  if (ySliderContainer.value) {
    resizeObserver = new ResizeObserver(() => syncYSliderSize())
    resizeObserver.observe(ySliderContainer.value)
  }
})

onUnmounted(() => {
  if (chartInstance) { chartInstance.destroy(); chartInstance = null }
  if (resizeObserver) { resizeObserver.disconnect(); resizeObserver = null }
})

watch(() => route.path, async (newPath) => {
  if (newPath === '/') { await nextTick(); await renderChart() }
})
</script>

<style scoped>
.home-page {
  margin: 0 auto;
}

.page-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
}

.page-title {
  font-size: 22px;
  font-weight: 600;
  margin: 0;
}

.header-actions {
  flex-shrink: 0;
}

.main-row {
  display: flex;
  gap: 12px;
  margin-bottom: 12px;
  align-items: stretch;
}

.side-panel {
  width: 120px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.side-stat {
  padding: 16px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  text-align: center;
}

.side-icon {
  font-size: 28px;
  color: var(--mdui-color-primary);
  opacity: 0.7;
}

.side-num {
  font-size: 28px;
  font-weight: 700;
  display: block;
  line-height: 1.1;
}

.side-sub {
  font-size: 12px;
  opacity: 0.6;
  display: block;
  margin-top: 2px;
}

.chart-card {
  flex: 1;
  overflow: hidden;
  min-width: 0;
}

.chart-card-inner {
  display: flex;
  height: 360px;
}

.y-slider-container {
  width: 44px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px 0;
  border-right: 1px solid rgba(0,0,0,0.06);
  position: relative;
}

.y-mdui-slider {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%) rotate(-90deg);
  transform-origin: center center;
}

.chart-area {
  flex: 1;
  padding: 16px 16px 16px 4px;
  min-width: 0;
  position: relative;
}

.chart-placeholder-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 80px 0;
  opacity: 0.4;
  background: var(--mdui-color-surface-container);
  border-radius: 16px;
  margin-bottom: 12px;
}

.placeholder-icon {
  font-size: 48px;
}

.chart-placeholder-card p {
  font-size: 14px;
}

.time-slider-row {
  padding: 4px 16px 12px 60px;
}
</style>
