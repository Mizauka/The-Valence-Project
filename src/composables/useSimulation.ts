// ─── useSimulation ────────────────────────────────────────────────
// Module-level shared state for simulation results.

import { ref, readonly, shallowRef } from 'vue'
import {
  runSimulation, applyCalibration, getCalibrationBand,
} from '../services/engineStore'
import type { SimulationOutput, CalibrationBand } from '../services/types'

export type DisplayUnit = 'pg/mL' | 'ng/mL' | 'µg/mL' | 'mg/L'

const UNIT_ORDER: DisplayUnit[] = ['pg/mL', 'ng/mL', 'µg/mL', 'mg/L']
const UNIT_FACTOR: Record<DisplayUnit, number> = {
  'pg/mL': 1, 'ng/mL': 1e-3, 'µg/mL': 1e-6, 'mg/L': 1e-6,
}

export function autoPickUnit(peakConc: number, rawUnit: string): { unit: DisplayUnit; factor: number } {
  if (!peakConc || peakConc <= 0) return { unit: 'ng/mL', factor: 1e-3 }
  for (const u of UNIT_ORDER) {
    const scaled = peakConc * UNIT_FACTOR[u]
    if (scaled >= 1 && scaled < 1000) return { unit: u, factor: UNIT_FACTOR[u] }
  }
  if (peakConc >= 1000) return { unit: 'mg/L', factor: 1e-6 }
  return { unit: 'pg/mL', factor: 1 }
}

export function convertUnit(values: number[], factor: number): number[] {
  return values.map(v => v * factor)
}

// ── Module-level shared state ─────────────────────────────────────

const rawResults = shallowRef<SimulationOutput[]>([])
const calibratedResults = shallowRef<Map<string, SimulationOutput>>(new Map())
const calibrationBands = shallowRef<Map<string, CalibrationBand>>(new Map())
const loading = ref(false)
const error = ref<string | null>(null)
const selectedGroup = ref<string | null>(null)
const selectedUnit = ref<DisplayUnit>('ng/mL')
const unitFactor = ref(1)

export function useSimulation() {
  async function simulate(): Promise<void> {
    loading.value = true
    error.value = null
    try {
      const raw = await runSimulation()
      rawResults.value = raw
      const calibMap = new Map<string, SimulationOutput>()
      const bandMap = new Map<string, CalibrationBand>()
      for (const sim of raw) {
        if (!sim.drug_name || sim.time_h.length === 0) continue
        try {
          const calib = await applyCalibration(sim)
          if (calib) calibMap.set(sim.drug_name, calib)
        } catch { /* ignore */ }
        try {
          const band = await getCalibrationBand(sim)
          if (band) bandMap.set(sim.drug_name, band)
        } catch { /* ignore */ }
      }
      calibratedResults.value = calibMap
      calibrationBands.value = bandMap
      if (raw.length > 0 && !selectedGroup.value && raw[0]) {
        selectedGroup.value = raw[0].drug_name
      }
    } catch (e: any) {
      error.value = e?.message || 'Simulation failed'
    } finally {
      loading.value = false
    }
  }

  function getGroupSim(groupName?: string): SimulationOutput | null {
    const name = groupName || selectedGroup.value
    if (!name) return null
    return rawResults.value.find(s => s.drug_name === name) || null
  }

  function getCalibratedSim(groupName?: string): SimulationOutput | null {
    const name = groupName || selectedGroup.value
    if (!name) return null
    return calibratedResults.value.get(name) || null
  }

  function getBand(groupName?: string): CalibrationBand | null {
    const name = groupName || selectedGroup.value
    if (!name) return null
    return calibrationBands.value.get(name) || null
  }

  return {
    rawResults: readonly(rawResults),
    calibratedResults: readonly(calibratedResults),
    calibrationBands: readonly(calibrationBands),
    loading: readonly(loading),
    error: readonly(error),
    selectedGroup, selectedUnit, unitFactor,
    simulate, getGroupSim, getCalibratedSim, getBand,
  }
}
