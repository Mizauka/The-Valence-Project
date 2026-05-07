<template>
  <div class="page home-page">
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

    <div v-if="!hasData" class="chart-placeholder-card">
      <mdui-icon name="show_chart" class="placeholder-icon" />
      <p>{{ placeholderText }}</p>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, nextTick, watch } from 'vue'
import { useRoute } from 'vue-router'
import * as echarts from 'echarts'
import * as store from '../wasm/engineStore'

const route = useRoute()
const chartContainer = ref(null)
const doseCount = ref(0)
const hasData = ref(false)
const placeholderText = ref('添加给药记录后，浓度曲线将在此显示')

let chart = null
let cachedSimResults = null
let resizeObserver = null

/** Per-substance: { idx, name, nativeUnit, displayUnit, convFactor, color } */
let substanceMeta = []

const COLORS = ['#E91E63', '#2196F3', '#00BCD4', '#FF9800', '#9C27B0', '#4CAF50']

// ── Unit resolution ──────────────────────────────────────

const UNIT_TO_PGML = { 'pg/mL': 1, 'ng/mL': 1_000, 'µg/mL': 1_000_000, 'mg/L': 1_000_000 }
const ALL_UNITS = ['pg/mL', 'ng/mL', 'µg/mL', 'mg/L']

function autoPickUnit(nativeMax, nativeUnit, targetMax) {
  const valPg = nativeMax * (UNIT_TO_PGML[nativeUnit] || 1)
  let best = { unit: nativeUnit, val: nativeMax, score: Infinity }
  for (const unit of ALL_UNITS) {
    const val = valPg / (UNIT_TO_PGML[unit] || 1)
    if (val < targetMax * 0.005 || val > targetMax * 5) continue
    const score = Math.abs(val / targetMax - 0.5)
    if (score < best.score) best = { unit, val, score }
  }
  return isFinite(best.score) && best.score < 100 ? best : { unit: nativeUnit, val: nativeMax, score: 0 }
}

function resolveDisplayUnits(simResults, calibBands) {
  const raw = []
  for (let i = 0; i < simResults.length; i++) {
    const r = simResults[i]
    if (!r) continue
    const conc = Array.from(r.concentrations || [])
    const name = r.drug_name || `Drug ${i + 1}`
    const nativeUnit = r.display_unit || 'mg/L'
    let maxVal = 0
    for (const v of conc) { if (v > maxVal) maxVal = v }
    raw.push({ idx: i, name, nativeUnit, maxRaw: maxVal, color: COLORS[i % COLORS.length] })
  }
  if (!raw.length) return 1200

  // Anchor = largest raw numeric value
  let anchor = raw[0]
  for (const r of raw) { if (r.maxRaw > anchor.maxRaw) anchor = r }

  // Base yMax from anchor raw value
  let yMax = Math.ceil(anchor.maxRaw * 1.15 / 100) * 100 || 1200

  // Also consider 68% CI upper bound of anchor (in native unit, no conversion needed for anchor)
  const anchorBand = calibBands ? calibBands[anchor.idx] : null
  if (anchorBand && anchorBand.ci68_high && anchorBand.ci68_high.length) {
    let ci68Max = 0
    for (const v of anchorBand.ci68_high) { if (v > ci68Max) ci68Max = v }
    if (ci68Max > yMax) yMax = Math.ceil(ci68Max * 1.05 / 100) * 100
  }

  const meta = []
  for (const r of raw) {
    if (r === anchor) {
      meta.push({ ...r, displayUnit: r.nativeUnit, convFactor: 1 })
    } else {
      const pick = autoPickUnit(r.maxRaw, r.nativeUnit, yMax)
      const toPg = UNIT_TO_PGML[r.nativeUnit] || 1
      const fromDisp = 1 / (UNIT_TO_PGML[pick.unit] || 1)
      meta.push({ ...r, displayUnit: pick.unit, convFactor: toPg * fromDisp })
    }
  }
  substanceMeta = meta
  return yMax
}

// ── Formatting ────────────────────────────────────────────

function fmtTime(ms) {
  const d = new Date(ms)
  const pad = n => String(n).padStart(2, '0')
  return `${pad(d.getMonth() + 1)}/${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`
}

function fmtTooltip(val, unit) {
  if (!isFinite(val)) return '\u2014'
  switch (unit) {
    case 'pg/mL': return `${Math.round(val).toLocaleString()} ${unit}`
    case 'ng/mL': return `${val.toFixed(2)} ${unit}`
    case 'µg/mL': return `${val.toFixed(2)} ${unit}`
    case 'mg/L': return `${val.toFixed(4)} ${unit}`
    default: return `${val.toFixed(4)} ${unit}`
  }
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
      if (chart) { chart.dispose(); chart = null }
      return
    }

    // Run simulation
    const simResults = engine.runSimulation()
    if (!simResults || simResults.length === 0) {
      hasData.value = false
      placeholderText.value = '计算引擎返回空结果'
      return
    }

    // Get calibration bands FIRST (needed for y-axis CI bounds)
    const calibBands = []
    for (const r of simResults) {
      const band = engine.getCalibrationBand(r)
      calibBands.push(band)
    }

    // Resolve per-substance display units (uses CI for y-axis max)
    let yAxisMax = resolveDisplayUnits(simResults, calibBands)

    cachedSimResults = simResults
    hasData.value = true

    await nextTick()
    if (!chartContainer.value) return

    const viewWindow = getViewWindow()
    const series = []
    const legendData = []

    for (const sm of substanceMeta) {
      const i = sm.idx
      const r = simResults[i]
      if (!r) continue

      const timeH = Array.from(r.time_h || [])
      const rawConc = Array.from(r.concentrations || [])
      if (!timeH.length || !rawConc.length) continue

      const cf = sm.convFactor
      const color = sm.color
      const areaColor = color + '18'
      const areaColor68 = color + '28'

      const toPair = (arr) => timeH.map((t, j) => [t * 3600 * 1000, Math.max(0, (arr[j] || 0) * cf)])

      // CI bands
      const band = calibBands[i]
      const hasCalib = band && band.calibrated && band.calibrated.length > 0
      const hasCI = hasCalib && band.ci95_low && band.ci95_high && band.ci95_low.length > 0

      if (hasCI) {
        const ci95Low = Array.from(band.ci95_low)
        const ci95High = Array.from(band.ci95_high)
        const ciGroupName = `${sm.name} 校准 (${sm.displayUnit})`
        series.push({
          name: ciGroupName, type: 'line', data: toPair(ci95Low), smooth: true, symbol: 'none',
          lineStyle: { opacity: 0 }, areaStyle: { color: areaColor, origin: 'auto' },
          yAxisIndex: 0, legendHoverLink: false, silent: true, stack: `ci95-${i}`,
        })
        series.push({
          name: ciGroupName, type: 'line',
          data: timeH.map((t, j) => [t * 3600 * 1000, Math.max(0, (ci95High[j] - ci95Low[j]) * cf)]),
          smooth: true, symbol: 'none', lineStyle: { opacity: 0 }, areaStyle: { color: areaColor },
          yAxisIndex: 0, legendHoverLink: false, silent: true, stack: `ci95-${i}`,
        })
        const ci68Low = Array.from(band.ci68_low)
        const ci68High = Array.from(band.ci68_high)
        series.push({
          name: ciGroupName, type: 'line', data: toPair(ci68Low), smooth: true, symbol: 'none',
          lineStyle: { opacity: 0 }, areaStyle: { color: areaColor68, origin: 'auto' },
          yAxisIndex: 0, legendHoverLink: false, silent: true, stack: `ci68-${i}`,
        })
        series.push({
          name: ciGroupName, type: 'line',
          data: timeH.map((t, j) => [t * 3600 * 1000, Math.max(0, (ci68High[j] - ci68Low[j]) * cf)]),
          smooth: true, symbol: 'none', lineStyle: { opacity: 0 }, areaStyle: { color: areaColor68 },
          yAxisIndex: 0, legendHoverLink: false, silent: true, stack: `ci68-${i}`,
        })
      }

      // Calibrated main curve
      if (hasCalib) {
        const calData = Array.from(band.calibrated)
        series.push({
          name: `${sm.name} 校准 (${sm.displayUnit})`, type: 'line', data: toPair(calData), smooth: true, symbol: 'none',
          lineStyle: { color, width: 2, type: 'dashed' }, itemStyle: { color },
          yAxisIndex: 0, emphasis: { focus: 'series' },
        })
        legendData.push(`${sm.name} 校准 (${sm.displayUnit})`)
      }

      // Raw data
      series.push({
        name: `${sm.name} 原始 (${sm.displayUnit})`, type: 'line', data: toPair(rawConc), smooth: true, symbol: 'none',
        lineStyle: { color, width: 1.5, opacity: 0.55, type: 'dotted' }, itemStyle: { color },
        yAxisIndex: 0,
      })
      legendData.push(`${sm.name} 原始 (${sm.displayUnit})`)
    }

    if (!series.length) {
      hasData.value = false
      placeholderText.value = '浓度数据为空'
      return
    }

    if (chart) { chart.dispose(); chart = null }

    chart = echarts.init(chartContainer.value)
    chart.setOption({
      tooltip: {
        trigger: 'axis',
        formatter(params) {
          if (!params || !params.length) return ''
          const timeVal = params[0]?.value[0] || 0
          let t = `<div style="font-weight:600;margin-bottom:6px">${fmtTime(timeVal)}</div>`
          // Collect unique series data
          const seen = new Set()
          for (const p of params) {
            const sn = p.seriesName
            if (sn.includes('95%CI') || sn.includes('68%CI')) continue
            if (seen.has(sn)) continue
            seen.add(sn)
            const val = p.value[1]
            if (val == null || !isFinite(val)) continue
            const isCalib = sn.includes('校准')
            const isRaw = sn.includes('原始')
            const baseName = sn.replace(' 校准', '').replace(' 原始', '')
            const sm = substanceMeta.find(m => sn.startsWith(m.name))
            const du = sm ? sm.displayUnit : ''
            const prefix = isCalib ? '个体化模型 ' : isRaw ? 'RAW ' : ''
            t += `<div style="display:flex;align-items:center;gap:6px;margin:2px 0">
              <span style="width:10px;height:10px;border-radius:50%;background:${p.color};display:inline-block"></span>
              ${prefix}${baseName}: ${fmtTooltip(val, du)}
            </div>`
          }

          // CI info
          for (const sm of substanceMeta) {
            const band = calibBands[sm.idx]
            if (!band || !band.ci95_low) continue
            const r = simResults[sm.idx]
            if (!r) continue
            const rTimeH = Array.from(r.time_h || [])
            const idx = rTimeH.findIndex(th => Math.abs(th * 3600 * 1000 - timeVal) < 180000)
            if (idx < 0) continue
            const ci95l = band.ci95_low[idx]
            const ci95h = band.ci95_high[idx]
            const ci68l = band.ci68_low?.[idx]
            const ci68h = band.ci68_high?.[idx]
            if (ci95l != null && ci95h != null && isFinite(ci95l) && isFinite(ci95h)) {
              t += `<div style="font-size:11px;opacity:0.7;margin-top:2px">95% 置信区间 ${fmtTooltip(ci95l, sm.nativeUnit)} – ${fmtTooltip(ci95h, sm.nativeUnit)}</div>`
            }
            if (ci68l != null && ci68h != null && isFinite(ci68l) && isFinite(ci68h)) {
              t += `<div style="font-size:11px;opacity:0.7">68% 区间 ${fmtTooltip(ci68l, sm.nativeUnit)} – ${fmtTooltip(ci68h, sm.nativeUnit)}</div>`
            }
          }
          return t
        },
      },
      legend: {
        data: legendData,
        top: 8,
        textStyle: { fontSize: 12 },
        selected: {},
      },
      grid: { left: 12, right: 24, top: 50, bottom: 60 },
      xAxis: {
        type: 'time',
        min: viewWindow.startMs,
        max: viewWindow.endMs,
        axisLabel: { formatter: v => fmtTime(v), fontSize: 11 },
        splitLine: { show: false },
      },
      yAxis: {
        type: 'value',
        max: yAxisMax,
        splitLine: { lineStyle: { color: 'rgba(0,0,0,0.06)' } },
      },
      dataZoom: [
        {
          type: 'slider',
          bottom: 8,
          height: 24,
          labelFormatter: v => fmtTime(v),
        },
        {
          type: 'inside',
        },
      ],
      series,
    })

    // Resize observer
    if (chartContainer.value && !resizeObserver) {
      resizeObserver = new ResizeObserver(() => chart?.resize())
      resizeObserver.observe(chartContainer.value)
    }

    // Recalculate y-axis max when legend toggles
    chart.on('legendselectchanged', function () {
      const option = chart.getOption()
      const selected = option.legend[0].selected || {}
      let visMax = 0
      for (const sr of option.series) {
        const sn = sr.name
        if (!sn || selected[sn] === false) continue
        if (sn.includes('95%CI') || sn.includes('68%CI')) continue
        const data = sr.data
        if (!data || !data.length) continue
        for (const pt of data) {
          if (Array.isArray(pt) && pt[1] > visMax) visMax = pt[1]
        }
      }
      if (visMax > 0) {
        const newMax = Math.ceil(visMax * 1.1 / 100) * 100 || yAxisMax
        chart.setOption({ yAxis: { max: newMax } })
      }
    })
  } catch (e) {
    console.error('[HomePage] renderChart failed:', e)
    hasData.value = false
    placeholderText.value = '数据加载失败: ' + e.message
  }
}

function getViewWindow() {
  const nowMs = Date.now()
  return { startMs: nowMs - 7 * 24 * 3600 * 1000, endMs: nowMs + 14 * 24 * 3600 * 1000 }
}

// ─── Export ───────────────────────────────────────────────

function downloadBlob(content, filename, mime) {
  const blob = new Blob([content], { type: mime })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url; a.download = filename; a.click()
  URL.revokeObjectURL(url)
}

function exportCSV() {
  if (!cachedSimResults?.length) return
  const rs = []
  for (const r of cachedSimResults) {
    if (!r) continue
    const rc = Array.from(r.concentrations || [])
    const unit = r.display_unit || 'mg/L'
    rs.push({ name: r.drug_name || 'unknown', unit, th: Array.from(r.time_h || []), conc: rc })
  }
  if (!rs.length) return
  const hdr = ['datetime', ...rs.map(r => `${r.name} (${r.unit})`)]
  const lines = [hdr.join(',')]
  const maxLen = Math.max(...rs.map(r => r.th.length))
  for (let i = 0; i < maxLen; i++) {
    const row = [rs[0].th[i] != null ? new Date(rs[0].th[i] * 3600000).toISOString() : '']
    for (const r of rs) row.push(r.conc[i] != null ? Math.max(0, r.conc[i]).toFixed(4) : '')
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
    const unit = r.display_unit || 'mg/L'
    payload.push({
      drug_name: r.drug_name,
      display_unit: unit,
      datetime: th.map(h => new Date(h * 3600000).toISOString()),
      concentrations: rc.map(v => Math.max(0, v)),
    })
  }
  downloadBlob(JSON.stringify(payload, null, 2), `valence_chart_${new Date().toISOString().slice(0, 10)}.json`, 'application/json')
}

// ─── Lifecycle ────────────────────────────────────────────

onMounted(() => renderChart())

onUnmounted(() => {
  if (chart) { chart.dispose(); chart = null }
  if (resizeObserver) { resizeObserver.disconnect(); resizeObserver = null }
})

watch(() => route.path, async p => { if (p === '/') { await nextTick(); renderChart() } })
</script>

<style scoped>
.home-page { margin: 0 auto; display: flex; flex-direction: column; gap: 8px; }
.header-actions { align-self: flex-end; }
.side-stat { align-self: flex-start; }
.dose-stat { display: flex; align-items: center; gap: 10px; padding: 10px 16px; }
.side-icon { font-size: 24px; color: var(--mdui-color-primary); opacity: 0.7; }
.side-num { font-size: 24px; font-weight: 700; display: block; line-height: 1.1; }
.side-sub { font-size: 12px; opacity: 0.6; display: block; }

.chart-container { width: 100%; height: 440px; }

.chart-placeholder-card {
  display: flex; flex-direction: column; align-items: center; justify-content: center;
  gap: 12px; padding: 80px 0; opacity: 0.4;
  background: var(--mdui-color-surface-container); border-radius: 16px;
}
.placeholder-icon { font-size: 48px; }
.chart-placeholder-card p { font-size: 14px; }
</style>
