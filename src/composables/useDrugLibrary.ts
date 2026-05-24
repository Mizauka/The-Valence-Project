// ─── useDrugLibrary ───────────────────────────────────────────────
// Module-level shared state — all components see the same drug data.

import { ref, shallowRef } from 'vue'
import { getAllDrugsWithSource, addDrug, deleteDrug } from '../services/engineStore'
import type { DrugRecord, RawDrugRecord } from '../services/types'

export interface DrugCategory {
  source: 'hrt' | 'journal' | 'custom'
  label: string
  drugs: DrugRecord[]
}

// ── Module-level shared state ─────────────────────────────────────

const allDrugs = shallowRef<DrugRecord[]>([])
const categories = shallowRef<DrugCategory[]>([])
const loading = ref(false)
const searchQuery = ref('')
const selectedDrugId = ref<string | null>(null)
let _loaded = false

function updateCategories(): void {
  const drugs = allDrugs.value
  const q = searchQuery.value.toLowerCase().trim()
  const hrtDrugs = drugs.filter(d => d.source === 'hrt' && (!q || d.name.toLowerCase().includes(q)))
  const journalDrugs = drugs.filter(d => d.source === 'journal' && (!q || d.name.toLowerCase().includes(q)))
  const customDrugs = drugs.filter(d => d.source === 'custom' && (!q || d.name.toLowerCase().includes(q)))
  const result: DrugCategory[] = []
  if (journalDrugs.length) result.push({ source: 'journal', label: 'Journal', drugs: journalDrugs })
  if (hrtDrugs.length) result.push({ source: 'hrt', label: 'HRT', drugs: hrtDrugs })
  if (customDrugs.length) result.push({ source: 'custom', label: '自定义', drugs: customDrugs })
  categories.value = result
}

export function useDrugLibrary() {
  async function load(): Promise<void> {
    loading.value = true
    try {
      allDrugs.value = await getAllDrugsWithSource()
      updateCategories()
      _loaded = true
    } finally {
      loading.value = false
    }
  }

  function setSearch(q: string): void {
    searchQuery.value = q
    updateCategories()
  }

  function selectDrug(id: string): void {
    selectedDrugId.value = id
  }

  function getSelectedDrug(): DrugRecord | null {
    return allDrugs.value.find(d => d.drug_id === selectedDrugId.value) || null
  }

  async function createDrug(data: RawDrugRecord): Promise<void> {
    await addDrug(data)
    await load()
  }

  async function removeDrug(id: string): Promise<void> {
    await deleteDrug(id)
    if (selectedDrugId.value === id) selectedDrugId.value = null
    await load()
  }

  if (!_loaded) {
    load()
  }

  return {
    allDrugs, categories, loading, searchQuery, selectedDrugId,
    load, setSearch, selectDrug, getSelectedDrug, createDrug, removeDrug,
  }
}
