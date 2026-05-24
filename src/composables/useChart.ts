// ─── useChart ─────────────────────────────────────────────────────
// ECharts-based concentration curve rendering with calibration bands.

import { ref, shallowRef, onUnmounted, type Ref } from 'vue'
import * as echarts from 'echarts'
import type { SimulationOutput, CalibrationBand } from '../services/types'
import { useSimulation, autoPickUnit, convertUnit } from './useSimulation'
import type { DisplayUnit } from './useSimulation'

// ─── Constants ────────────────────────────────────────────────────

const COLORS = ['#E91E63', '#2196F3', '#00BCD4', '#FF9800', '#9C27B0', '#4CAF50']
const CI95_COLOR = 'rgba(128,128,128,0.15)'
const CI68_COLOR = 'rgba(128,128,128,0.25)'

// ─── Format Helpers ───────────────────────────────────────────────

function fmtHours(h: number): string {
  const abs = Math.abs(h)
  const days = Math.floor(abs / 24)
  const hrs = abs % 24
  if (days === 0) return `${h >= 0 ? '' : '-'}${hrs.toFixed(1)}h`
  return `${h >= 0 ? '' : '-'}${days}d${hrs.toFixed(0)}h`
}

function fmtDate(tsMs: number): string {
  const d = new Date(tsMs)
  return `${d.getMonth() + 1}/${d.getDate()} ${d.getHours().toString().padStart(2, '0')}:${d.getMinutes().toString().padStart(2, '0')}`
}

// ─── Composable ───────────────────────────────────────────────────

export function useChart(containerRef: Ref<HTMLElement | null>) {
  const sim = useSimulation()
  let chart: echarts.ECharts | null = null
  let resizeObserver: ResizeObserver | null = null

  // ─── Time Window ────────────────────────────────────────────────

  type TimeWindow = 'day' | 'week' | 'month'
  const timeWindow = ref<TimeWindow>('week')

  function getTimeRange(): [number, number] {
    const nowH = Date.now() / 3600_000
    switch (timeWindow.value) {
      case 'day': return [nowH - 24, nowH + 48]
      case 'week': return [nowH - 168, nowH + 336]
      case 'month': return [nowH - 720, nowH + 720]
    }
  }

  // ─── Build ECharts Option ───────────────────────────────────────

  async function buildOption() {
    await sim.simulate()
    const results = sim.rawResults.value
    const bands = sim.calibrationBands.value

    if (!results.length) return null

    const [tMin, tMax] = getTimeRange()
    // Convert time to ms for xAxis
    const tMinMs = tMin * 3600_000
    const tMaxMs = tMax * 3600_000

    // Determine Y-axis based on anchor substance
    let yMax = 0
    let anchorResult: SimulationOutput | null = null
    let anchorIdx = 0
    for (let i = 0; i < results.length; i++) {
      const r = results[i]!
      const maxVal = Math.max(...r.concentrations, 0)
      if (maxVal > yMax) {
        yMax = maxVal
        anchorResult = r as unknown as SimulationOutput
        anchorIdx = i
      }
    }
    yMax = Math.ceil(yMax * 1.2 / 100) * 100 || 1200

    const series: any[] = []
    const legendData: string[] = []

    for (let i = 0; i < results.length; i++) {
      const r = results[i]!
      const color = COLORS[i % COLORS.length]
      const name = r.drug_name || `Drug ${i + 1}`

      // Time in ms
      const timesMs = r.time_h.map(t => t * 3600_000)

      // Normalize concentrations for display
      let displayConc: number[]
      let displayUnit: DisplayUnit

      const concArr = Array.from(r.concentrations) as number[]

      if (i === anchorIdx) {
        displayConc = [...concArr]
        displayUnit = (r.display_unit as DisplayUnit) || 'ng/mL'
      } else {
        const peak = Math.max(...concArr, 0)
        const pick = autoPickUnit(peak, r.display_unit)
        displayConc = convertUnit(concArr, pick.factor)
        displayUnit = pick.unit
      }

      const seriesName = `${name} (${displayUnit})`

      // Main concentration line
      series.push({
        name: seriesName,
        type: 'line',
        data: timesMs.map((t, j) => [t, displayConc[j]]),
        smooth: true,
        symbol: 'none',
        lineStyle: { color, width: 2 },
        itemStyle: { color },
        yAxisIndex: 0,
      })
      legendData.push(seriesName)

      // Calibration band
      const band = bands.get(r.drug_name)
      if (band && band.ci95_low?.length && displayConc.length > 0 && concArr.length > 0) {
        const ciFactor = i === anchorIdx ? 1 : displayConc[0]! / concArr[0]!
        const low95 = band.ci95_low.map(v => v * ciFactor)
        const high95 = band.ci95_high.map(v => v * ciFactor)
        const low68 = band.ci68_low.map(v => v * ciFactor)
        const high68 = band.ci68_high.map(v => v * ciFactor)

        // 95% CI band
        series.push({
          name: `${name} 95%CI`,
          type: 'line',
          data: timesMs.map((t, j) => [t, high95[j]]),
          lineStyle: { opacity: 0 },
          symbol: 'none',
          stack: 'ci95',
          areaStyle: { color: CI95_COLOR },
          silent: true,
          legendHoverLink: false,
        })
        series.push({
          name: `${name} 95%CI`,
          type: 'line',
          data: timesMs.map((t, j) => [t, low95[j]]),
          lineStyle: { opacity: 0 },
          symbol: 'none',
          stack: 'ci95',
          areaStyle: { color: 'transparent' },
          silent: true,
          legendHoverLink: false,
        })

        // 68% CI band
        series.push({
          name: `${name} 68%CI`,
          type: 'line',
          data: timesMs.map((t, j) => [t, high68[j]]),
          lineStyle: { opacity: 0 },
          symbol: 'none',
          stack: 'ci68',
          areaStyle: { color: CI68_COLOR },
          silent: true,
          legendHoverLink: false,
        })
        series.push({
          name: `${name} 68%CI`,
          type: 'line',
          data: timesMs.map((t, j) => [t, low68[j]]),
          lineStyle: { opacity: 0 },
          symbol: 'none',
          stack: 'ci68',
          areaStyle: { color: 'transparent' },
          silent: true,
          legendHoverLink: false,
        })
      }
    }

    // Anchor unit for Y axis label
    const anchorUnit = anchorResult?.display_unit || 'ng/mL'

    return {
      animation: true,
      animationDuration: 300,
      tooltip: {
        trigger: 'axis',
        formatter: (params: any[]) => {
          if (!params?.length) return ''
          const t = fmtDate(params[0].value[0])
          let s = `<div style="font-weight:500;margin-bottom:4px">${t}</div>`
          for (const p of params) {
            if (p.seriesName?.includes('CI')) continue
            const marker = `<span style="display:inline-block;width:10px;height:10px;border-radius:50%;background:${p.color};margin-right:6px"></span>`
            s += `<div>${marker}${p.seriesName}: <b>${p.value[1]?.toFixed(1)}</b></div>`
          }
          return s
        },
      },
      legend: {
        data: legendData,
        bottom: 0,
        textStyle: { fontSize: 11 },
      },
      grid: {
        left: 60, right: 20, top: 20, bottom: 40,
      },
      xAxis: {
        type: 'time',
        min: tMinMs,
        max: tMaxMs,
        axisLabel: {
          formatter: (_v: number, _idx: number) => {
            // Let ECharts auto-format with time axis
            return ''
          },
        },
        splitLine: { show: false },
      },
      yAxis: {
        type: 'value',
        name: anchorUnit,
        max: yMax,
        min: 0,
        axisLabel: { fontSize: 11 },
        splitLine: { lineStyle: { color: 'rgba(128,128,128,0.15)' } },
      },
      dataZoom: [
        {
          type: 'slider',
          start: 0,
          end: 100,
          height: 24,
          bottom: 30,
          borderColor: 'transparent',
          backgroundColor: 'rgba(128,128,128,0.1)',
        },
      ],
      series,
    }
  }

  // ─── Render ─────────────────────────────────────────────────────

  async function render() {
    if (!containerRef.value) return
    const option = await buildOption()
    if (!option) {
      // Clear chart if no data
      if (chart) { chart.dispose(); chart = null }
      return
    }

    if (!chart) {
      chart = echarts.init(containerRef.value)
      if (!resizeObserver) {
        resizeObserver = new ResizeObserver(() => chart?.resize())
        resizeObserver.observe(containerRef.value)
      }
    }
    chart.setOption(option, true)
  }

  function setTimeWindow(w: TimeWindow) {
    timeWindow.value = w
    render()
  }

  function dispose() {
    resizeObserver?.disconnect()
    resizeObserver = null
    chart?.dispose()
    chart = null
  }

  return {
    sim,
    timeWindow,
    render,
    setTimeWindow,
    dispose,
  }
}
