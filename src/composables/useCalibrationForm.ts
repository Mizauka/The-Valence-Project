// ─── useCalibrationForm ───────────────────────────────────────────
// Form state and logic for adding lab results.

import { ref, computed } from 'vue'
import { useCalibration } from './useCalibration'
import type { LabResult } from '../services/types'

export function useCalibrationForm() {
  const { addLab, labResults, load } = useCalibration()

  const timeH = ref('')
  const concValue = ref('')
  const unit = ref('pg/mL')
  const groupId = ref('')

  const availableGroups = computed(() => {
    const groups = new Set<string>()
    for (const lab of labResults.value) {
      if (lab.group_id) groups.add(lab.group_id)
    }
    return Array.from(groups)
  })

  const canSave = computed(() => {
    const t = parseFloat(timeH.value)
    const c = parseFloat(concValue.value)
    return !isNaN(t) && t >= 0 && !isNaN(c) && c > 0
  })

  function reset() {
    timeH.value = ''
    concValue.value = ''
    unit.value = 'pg/mL'
    groupId.value = ''
  }

  async function save() {
    if (!canSave.value) return
    const lab: LabResult = {
      id: crypto.randomUUID(),
      time_h: parseFloat(timeH.value),
      conc_value: parseFloat(concValue.value),
      unit: unit.value,
      group_id: groupId.value,
    }
    await addLab(lab)
    await load()
    reset()
  }

  return {
    timeH, concValue, unit, groupId,
    availableGroups, canSave,
    reset, save,
  }
}
