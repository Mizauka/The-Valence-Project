// ─── useDoseHistory ───────────────────────────────────────────────
// Module-level shared state — all components see the same dose data.

import { ref, shallowRef } from 'vue'
import { getAllDoses, addDose, removeDose } from '../services/engineStore'
import type { DisplayDose, DoseInput } from '../services/types'

export interface DoseGroup {
  date: string
  timestamp: number
  doses: DisplayDose[]
}

function formatDate(ts: number): string {
  const d = new Date(ts * 1000)
  return `${d.getFullYear()}/${d.getMonth() + 1}/${d.getDate()}`
}

function formatTime(ts: number): string {
  const d = new Date(ts * 1000)
  return [
    d.getHours().toString().padStart(2, '0'),
    d.getMinutes().toString().padStart(2, '0'),
    d.getSeconds().toString().padStart(2, '0'),
  ].join(':')
}

// ── Module-level shared state ─────────────────────────────────────

const doses = shallowRef<DisplayDose[]>([])
const groups = shallowRef<DoseGroup[]>([])
const loading = ref(false)
const selectedDoseId = ref<string | null>(null)
let _loaded = false

function updateGroups(): void {
  const map = new Map<string, DisplayDose[]>()
  for (const d of doses.value) {
    const date = formatDate(d.timestamp)
    if (!map.has(date)) map.set(date, [])
    map.get(date)!.push(d)
  }
  const entries = Array.from(map.entries())
  entries.sort((a, b) => {
    const ta = a[1][0]?.timestamp ?? 0
    const tb = b[1][0]?.timestamp ?? 0
    return tb - ta
  })
  groups.value = entries.map(([date, doseList]) => ({
    date,
    timestamp: doseList[0]?.timestamp ?? 0,
    doses: doseList.sort((a, b) => b.timestamp - a.timestamp),
  }))
}

export function useDoseHistory() {
  async function load(): Promise<void> {
    loading.value = true
    try {
      doses.value = await getAllDoses()
      updateGroups()
      _loaded = true
    } finally {
      loading.value = false
    }
  }

  function selectDose(id: string): void {
    selectedDoseId.value = id
  }

  async function createDose(data: DoseInput): Promise<void> {
    await addDose(data)
    await load()
  }

  async function deleteDose(id: string): Promise<void> {
    await removeDose(id)
    if (selectedDoseId.value === id) selectedDoseId.value = null
    await load()
  }

  const doseDescription = (d: DisplayDose): string =>
    `${formatTime(d.timestamp)} ${d.display_amount}${d.display_unit}`

  // Auto-load on first use
  if (!_loaded) {
    load()
  }

  return {
    doses, groups, loading, selectedDoseId,
    load, selectDose, createDose, deleteDose, doseDescription,
  }
}
