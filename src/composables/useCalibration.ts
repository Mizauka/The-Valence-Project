// ─── useCalibration ───────────────────────────────────────────────
// Module-level shared state — all components see the same calibration data.

import { ref, shallowRef, computed } from 'vue'
import {
  getCalibrationModel, setCalibrationModel,
  addLabResult, clearLabResults, getLabResults,
} from '../services/engineStore'
import type { LabResult } from '../services/types'

export type CalibrationModelType = 'ratio' | 'ou-kalman'

// ── Module-level shared state ─────────────────────────────────────

const model = ref<CalibrationModelType>('ratio')
const labResults = shallowRef<LabResult[]>([])
const loading = ref(false)
let _loaded = false

export function useCalibration() {
  async function load(): Promise<void> {
    loading.value = true
    try {
      model.value = (await getCalibrationModel()) as CalibrationModelType
      labResults.value = await getLabResults()
      _loaded = true
    } finally {
      loading.value = false
    }
  }

  async function setModel(m: CalibrationModelType): Promise<void> {
    await setCalibrationModel(m)
    model.value = m
  }

  async function addLab(lab: LabResult): Promise<void> {
    await addLabResult(lab)
    labResults.value = await getLabResults()
  }

  async function clearLabs(): Promise<void> {
    await clearLabResults()
    labResults.value = []
  }

  async function removeLab(index: number): Promise<void> {
    const current = [...labResults.value]
    current.splice(index, 1)
    await clearLabResults()
    for (const l of current) {
      await addLabResult(l)
    }
    labResults.value = await getLabResults()
  }

  const labCount = computed(() => labResults.value.length)

  if (!_loaded) {
    load()
  }

  return {
    model, labResults, loading, labCount,
    load, setModel, addLab, clearLabs, removeLab,
  }
}
