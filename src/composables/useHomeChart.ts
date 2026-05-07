import { ref, nextTick } from 'vue'
import * as echarts from 'echarts'
import * as store from '../wasm/engineStore'
import { fmtTime, fmtTooltip } from '../utils/format'

// ── Constants ──────────────────────────────────────────────

const COLORS = ['#E91E63', '#2196F3', '#00BCD4', '#FF9800', '#9C27B0', '#4CAF50']
const UNIT_TO_PGML: Record<string, number> = { 'pg/mL': 1, 'ng/mL': 1_000, 'µg/mL': 1_000_000, 'mg/L': 1_000_000 }
const ALL_UNITS = ['pg/mL', 'ng/mL', 'µg/mL', 'mg/L']

export interface SubstanceMeta {
  idx: number; name: string; nativeUnit: string; displayUnit: string
  convFactor: number; color: string; maxRaw: number
}

// ── Unit Resolution ────────────────────────────────────────

function autoPickUnit(nativeMax: number, nativeUnit: string, targetMax: number) {
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

function resolveDisplayUnits(simResults: any[], calibBands: any[]): { meta: SubstanceMeta[]; yMax: number } {
  const raw: any[] = []
  for (let i = 0; i < simResults.length; i++) {
    const r = simResults[i]
    if (!r) continue
    const conc = Array.from(r.concentrations || []) as number[]
    const name: string = r.drug_name || `Drug ${i + 1}`
    const nativeUnit: string = r.display_unit || 'mg/L'
    let maxVal = 0
    for (const v of conc) { if (v > maxVal) maxVal = v }
    raw.push({ idx: i, name, nativeUnit, maxRaw: maxVal, color: COLORS[i % COLORS.length] })
  }
  if (!raw.length) return { meta: [], yMax: 1200 }

  let anchor = raw[0]
  for (const r of raw) { if (r.maxRaw > anchor.maxRaw) anchor = r }

  let yMax = Math.ceil(anchor.maxRaw * 1.15 / 100) * 100 || 1200

  const anchorBand = calibBands ? calibBands[anchor.idx] : null
  if (anchorBand?.ci68_high?.length) {
    let ci68Max = 0
    for (const v of anchorBand.ci68_high) { if (v > ci68Max) ci68Max = v }
    if (ci68Max > yMax) yMax = Math.ceil(ci68Max * 1.05 / 100) * 100
  }

  const meta: SubstanceMeta[] = []
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
  return { meta, yMax }
}

// ── Main Chart Render ──────────────────────────────────────

export function useHomeChart() {
  const chartContainer = ref<HTMLElement | null>(null)
  const doseCount = ref(0)
  const hasData = ref(false)
  const placeholderText = ref('添加给药记录后，浓度曲线将在此显示')

  let chart: echarts.ECharts | null = null
  let cachedSimResults: any[] | null = null
  let resizeObserver: ResizeObserver | null = null
  let substanceMeta: SubstanceMeta[] = []

  function getViewWindow() {
    const nowMs = Date.now()
    return { startMs: nowMs - 7 * 24 * 3600 * 1000, endMs: nowMs + 14 * 24 * 3600 * 1000 }
  }

  async function renderChart() {
    try {
      const engine = await store.getEngine()
      if (!engine) { hasData.value = false; placeholderText.value = '计算引擎加载失败'; return }

      const doses = await store.getAllDoses()
      doseCount.value = doses.length
      if (!doses.length) {
        hasData.value = false; placeholderText.value = '添加给药记录后，浓度曲线将在此显示'
        if (chart) { chart.dispose(); chart = null }
        return
      }

      const simResults = engine.runSimulation()
      if (!simResults?.length) { hasData.value = false; placeholderText.value = '计算引擎返回空结果'; return }

      const calibBands: any[] = []
      for (const r of simResults) { calibBands.push(engine.getCalibrationBand(r)) }

      const { meta, yMax } = resolveDisplayUnits(simResults, calibBands)
      substanceMeta = meta
      let yAxisMax = yMax
      cachedSimResults = simResults
      hasData.value = true

      await nextTick()
      if (!chartContainer.value) return

      const viewWindow = getViewWindow()
      const series: any[] = []
      const legendData: string[] = []

      for (const sm of substanceMeta) {
        const i = sm.idx
        const r = simResults[i]
        if (!r) continue

        const timeH = Array.from(r.time_h || []) as number[]
        const rawConc = Array.from(r.concentrations || []) as number[]
        if (!timeH.length || !rawConc.length) continue

        const cf = sm.convFactor
        const color = sm.color
        const areaColor = color + '18'
        const areaColor68 = color + '28'
        const toPair = (arr: any[]) => timeH.map((t, j) => [t * 3600 * 1000, Math.max(0, (arr[j] || 0) * cf)])

        const band = calibBands[i]
        const hasCalib = band?.calibrated?.length > 0
        const hasCI = hasCalib && band.ci95_low?.length > 0

        if (hasCI) {
          const ci95Low = Array.from(band.ci95_low) as number[]
          const ci95High = Array.from(band.ci95_high) as number[]
          const ciGroupName = `${sm.name} 校准 (${sm.displayUnit})`
          series.push({ name: ciGroupName, type: 'line', data: toPair(ci95Low), smooth: true, symbol: 'none', lineStyle: { opacity: 0 }, areaStyle: { color: areaColor, origin: 'auto' }, yAxisIndex: 0, legendHoverLink: false, silent: true, stack: `ci95-${i}` })
          series.push({ name: ciGroupName, type: 'line', data: timeH.map((t, j) => [t * 3600 * 1000, Math.max(0, (ci95High[j] - ci95Low[j]) * cf)]), smooth: true, symbol: 'none', lineStyle: { opacity: 0 }, areaStyle: { color: areaColor }, yAxisIndex: 0, legendHoverLink: false, silent: true, stack: `ci95-${i}` })
          const ci68Low = Array.from(band.ci68_low) as number[]
          const ci68High = Array.from(band.ci68_high) as number[]
          series.push({ name: ciGroupName, type: 'line', data: toPair(ci68Low), smooth: true, symbol: 'none', lineStyle: { opacity: 0 }, areaStyle: { color: areaColor68, origin: 'auto' }, yAxisIndex: 0, legendHoverLink: false, silent: true, stack: `ci68-${i}` })
          series.push({ name: ciGroupName, type: 'line', data: timeH.map((t, j) => [t * 3600 * 1000, Math.max(0, (ci68High[j] - ci68Low[j]) * cf)]), smooth: true, symbol: 'none', lineStyle: { opacity: 0 }, areaStyle: { color: areaColor68 }, yAxisIndex: 0, legendHoverLink: false, silent: true, stack: `ci68-${i}` })
        }

        if (hasCalib) {
          series.push({ name: `${sm.name} 校准 (${sm.displayUnit})`, type: 'line', data: toPair(band.calibrated), smooth: true, symbol: 'none', lineStyle: { color, width: 2, type: 'dashed' }, itemStyle: { color }, yAxisIndex: 0, emphasis: { focus: 'series' } })
          legendData.push(`${sm.name} 校准 (${sm.displayUnit})`)
        }

        series.push({ name: `${sm.name} 原始 (${sm.displayUnit})`, type: 'line', data: toPair(rawConc), smooth: true, symbol: 'none', lineStyle: { color, width: 1.5, opacity: 0.55, type: 'dotted' }, itemStyle: { color }, yAxisIndex: 0 })
        legendData.push(`${sm.name} 原始 (${sm.displayUnit})`)
      }

      if (!series.length) { hasData.value = false; placeholderText.value = '浓度数据为空'; return }

      if (chart) { chart.dispose(); chart = null }
      chart = echarts.init(chartContainer.value)

      chart.setOption({
        tooltip: {
          trigger: 'axis',
          formatter(params: any[]) {
            if (!params?.length) return ''
            const timeVal = params[0]?.value[0] || 0
            let t = `<div style="font-weight:600;margin-bottom:6px">${fmtTime(timeVal)}</div>`
            const seen = new Set<string>()
            for (const p of params) {
              const sn = p.seriesName
              if (sn.includes('95%CI') || sn.includes('68%CI')) continue
              if (seen.has(sn)) continue; seen.add(sn)
              const val = p.value[1]
              if (val == null || !isFinite(val)) continue
              const isCalib = sn.includes('校准'); const isRaw = sn.includes('原始')
              const baseName = sn.replace(' 校准', '').replace(' 原始', '')
              const sm2 = substanceMeta.find(m => sn.startsWith(m.name))
              const du = sm2 ? sm2.displayUnit : ''
              const prefix = isCalib ? '个体化模型 ' : isRaw ? 'RAW ' : ''
              t += `<div style="display:flex;align-items:center;gap:6px;margin:2px 0"><span style="width:10px;height:10px;border-radius:50%;background:${p.color};display:inline-block"></span>${prefix}${baseName}: ${fmtTooltip(val, du)}</div>`
            }
            for (const sm2 of substanceMeta) {
              const band = calibBands[sm2.idx]
              if (!band?.ci95_low) continue
              const r = simResults[sm2.idx]; if (!r) continue
              const rTimeH = Array.from(r.time_h || []) as number[]
              const idx = rTimeH.findIndex(th => Math.abs(th * 3600 * 1000 - timeVal) < 180000)
              if (idx < 0) continue
              if (band.ci95_low[idx] != null && isFinite(band.ci95_low[idx])) {
                t += `<div style="font-size:11px;opacity:0.7;margin-top:2px">95% 置信区间 ${fmtTooltip(band.ci95_low[idx], sm2.nativeUnit)} – ${fmtTooltip(band.ci95_high[idx], sm2.nativeUnit)}</div>`
              }
              if (band.ci68_low?.[idx] != null && isFinite(band.ci68_low[idx])) {
                t += `<div style="font-size:11px;opacity:0.7">68% 区间 ${fmtTooltip(band.ci68_low[idx], sm2.nativeUnit)} – ${fmtTooltip(band.ci68_high[idx], sm2.nativeUnit)}</div>`
              }
            }
            return t
          },
        },
        legend: { data: legendData, top: 8, textStyle: { fontSize: 12 }, selected: {} },
        grid: { left: 12, right: 24, top: 50, bottom: 60 },
        xAxis: { type: 'time', min: viewWindow.startMs, max: viewWindow.endMs, axisLabel: { formatter: (v: number) => fmtTime(v), fontSize: 11 }, splitLine: { show: false } },
        yAxis: { type: 'value', max: yAxisMax, splitLine: { lineStyle: { color: 'rgba(0,0,0,0.06)' } } },
        dataZoom: [{ type: 'slider', bottom: 8, height: 24, labelFormatter: (v: number) => fmtTime(v) }, { type: 'inside' }],
        series,
      })

      if (chartContainer.value && !resizeObserver) {
        resizeObserver = new ResizeObserver(() => chart?.resize())
        resizeObserver.observe(chartContainer.value)
      }

      chart.on('legendselectchanged', function () {
        const option = chart!.getOption()
        const selected = option.legend[0].selected || {}
        let visMax = 0
        for (const sr of option.series as any[]) {
          const sn = sr.name; if (!sn || selected[sn] === false) continue
          if (sn.includes('95%CI') || sn.includes('68%CI')) continue
          if (!sr.data?.length) continue
          for (const pt of sr.data) { if (Array.isArray(pt) && pt[1] > visMax) visMax = pt[1] }
        }
        if (visMax > 0) chart!.setOption({ yAxis: { max: Math.ceil(visMax * 1.1 / 100) * 100 || yAxisMax } })
      })
    } catch (e: any) {
      console.error('[HomePage] renderChart failed:', e)
      hasData.value = false; placeholderText.value = '数据加载失败: ' + e.message
    }
  }

  function dispose() {
    if (chart) { chart.dispose(); chart = null }
    if (resizeObserver) { resizeObserver.disconnect(); resizeObserver = null }
  }

  function exportCSV() {
    if (!cachedSimResults?.length) return
    const rs: any[] = []
    for (const r of cachedSimResults) {
      if (!r) continue
      const rc = Array.from(r.concentrations || [])
      const unit = r.display_unit || 'mg/L'
      rs.push({ name: r.drug_name || 'unknown', unit, th: Array.from(r.time_h || []), conc: rc })
    }
    if (!rs.length) return
    const hdr = ['datetime', ...rs.map((r: any) => `${r.name} (${r.unit})`)]
    const lines = [hdr.join(',')]
    const maxLen = Math.max(...rs.map((r: any) => r.th.length))
    for (let i = 0; i < maxLen; i++) {
      const row = [rs[0].th[i] != null ? new Date(rs[0].th[i] * 3600000).toISOString() : '']
      for (const r of rs) row.push(r.conc[i] != null ? Math.max(0, r.conc[i]).toFixed(4) : '')
      lines.push(row.join(','))
    }
    downloadBlob(lines.join('\n'), `valence_chart_${new Date().toISOString().slice(0, 10)}.csv`, 'text/csv')
  }

  function exportJSON() {
    if (!cachedSimResults?.length) return
    const payload: any[] = []
    for (const r of cachedSimResults) {
      if (!r) continue
      const rc = Array.from(r.concentrations || [])
      const th = Array.from(r.time_h || [])
      const unit = r.display_unit || 'mg/L'
      payload.push({ drug_name: r.drug_name, display_unit: unit, datetime: th.map((h: number) => new Date(h * 3600000).toISOString()), concentrations: rc.map((v: number) => Math.max(0, v)) })
    }
    downloadBlob(JSON.stringify(payload, null, 2), `valence_chart_${new Date().toISOString().slice(0, 10)}.json`, 'application/json')
  }

  return { chartContainer, doseCount, hasData, placeholderText, renderChart, dispose, exportCSV, exportJSON }
}

function downloadBlob(content: string, filename: string, mime: string) {
  const blob = new Blob([content], { type: mime })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url; a.download = filename; a.click()
  URL.revokeObjectURL(url)
}
