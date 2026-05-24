// ─── usePlan ──────────────────────────────────────────────────────
// Medication plan management: list, create, edit, delete.
// Plans are stored as JSON in localStorage/OPFS with a simple schema.

import { ref, computed } from 'vue'
import type { DrugRecord } from '../services/types'

export interface PlanDose {
  drug_id: string
  drugName: string
  dose_amount: number
  dose_unit: string
  route: string
  interval_h: number   // dosing interval in hours
  duration_d: number   // total duration in days
}

export interface Plan {
  id: string
  name: string
  description: string
  createdAt: string
  updatedAt: string
  doses: PlanDose[]
}

const STORAGE_KEY = 'valence_plans'

function loadPlans(): Plan[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    return raw ? JSON.parse(raw) : []
  } catch {
    return []
  }
}

function savePlans(plans: Plan[]) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(plans))
}

export function usePlan() {
  const plans = ref<Plan[]>(loadPlans())
  const selectedPlanId = ref<string | null>(null)
  const editingPlan = ref<Plan | null>(null)

  const selectedPlan = computed(() =>
    plans.value.find(p => p.id === selectedPlanId.value) || null
  )

  const groupedPlans = computed(() => {
    const map = new Map<string, Plan[]>()
    for (const plan of plans.value) {
      const date = plan.createdAt.slice(0, 10).replace(/-/g, '/')
      if (!map.has(date)) map.set(date, [])
      map.get(date)!.push(plan)
    }
    return Array.from(map.entries())
      .sort((a, b) => b[0].localeCompare(a[0]))
      .map(([date, items]) => ({ date, plans: items }))
  })

  function selectPlan(id: string) {
    selectedPlanId.value = id
  }

  function startNew() {
    editingPlan.value = {
      id: crypto.randomUUID(),
      name: '',
      description: '',
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
      doses: [],
    }
  }

  function startEdit(plan?: Plan) {
    editingPlan.value = plan
      ? { ...plan, doses: plan.doses.map(d => ({ ...d })) }
      : null
  }

  function cancelEdit() {
    editingPlan.value = null
  }

  function addDoseToPlan(dose: PlanDose) {
    if (!editingPlan.value) return
    editingPlan.value.doses.push(dose)
  }

  function removeDoseFromPlan(index: number) {
    if (!editingPlan.value) return
    editingPlan.value.doses.splice(index, 1)
  }

  async function savePlan() {
    if (!editingPlan.value) return
    const plan = editingPlan.value
    plan.updatedAt = new Date().toISOString()

    const idx = plans.value.findIndex(p => p.id === plan.id)
    if (idx >= 0) {
      plans.value[idx] = { ...plan }
    } else {
      plans.value.push({ ...plan })
    }
    savePlans(plans.value)
    selectedPlanId.value = plan.id
    editingPlan.value = null
  }

  async function deletePlan(id: string) {
    plans.value = plans.value.filter(p => p.id !== id)
    if (selectedPlanId.value === id) selectedPlanId.value = null
    savePlans(plans.value)
  }

  // Quickly add all doses from a plan as actual dose records
  function getDoseRecordsFromPlan(plan: Plan): Array<{
    drug_id: string
    dose_amount: number
    timestamp: number
    route: string
  }> {
    const records: Array<{
      drug_id: string
      dose_amount: number
      timestamp: number
      route: string
    }> = []

    const startTime = Date.now() / 3600_000 // current time in hours
    for (const dose of plan.doses) {
      const numDoses = Math.floor((dose.duration_d * 24) / dose.interval_h)
      for (let i = 0; i < numDoses; i++) {
        records.push({
          drug_id: dose.drug_id,
          dose_amount: dose.dose_amount,
          timestamp: startTime + i * dose.interval_h,
          route: dose.route,
        })
      }
    }
    return records
  }

  return {
    plans, selectedPlanId, selectedPlan, groupedPlans,
    editingPlan,
    selectPlan, startNew, startEdit, cancelEdit,
    addDoseToPlan, removeDoseFromPlan, savePlan, deletePlan,
    getDoseRecordsFromPlan,
  }
}
