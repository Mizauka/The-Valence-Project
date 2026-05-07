import { ref, computed, onMounted } from 'vue'
import * as store from '../wasm/engineStore'
import { getCurrentTimestamp } from '../utils/format'

export function useDoseForm() {
  const searchQuery = ref('')
  const activeSource = ref('all')
  const allDrugs = ref<any[]>([])
  const selectedDrug = ref<any>(null)
  const doseAmount = ref('')
  const route = ref('oral')
  const timestamp = ref('')
  const displayedCount = ref(80)

  const canSave = computed(() => { const v = parseFloat(doseAmount.value); return !isNaN(v) && v > 0 })

  const availableRoutes = computed(() => {
    if (!selectedDrug.value) return [{ route: 'oral', unit: 'mg' }]
    return selectedDrug.value.routes || [{ route: 'oral', unit: selectedDrug.value.dose_unit || 'mg' }]
  })

  const currentDoseUnit = computed(() => {
    const matched = availableRoutes.value.find((r: any) => r.route === route.value)
    return matched ? matched.unit : (selectedDrug.value?.dose_unit || 'mg')
  })

  const sourceFilteredDrugs = computed(() => {
    if (activeSource.value === 'all') return allDrugs.value
    return allDrugs.value.filter((d: any) => d.source === activeSource.value)
  })

  const filteredDrugs = computed(() => {
    const q = searchQuery.value.trim().toLowerCase()
    const list = sourceFilteredDrugs.value
    if (!q) return list
    return list.filter((d: any) => d.name.toLowerCase().includes(q) || d.drug_id.toLowerCase().includes(q))
  })

  const displayDrugs = computed(() => filteredDrugs.value.slice(0, displayedCount.value))
  const hasMore = computed(() => displayedCount.value < filteredDrugs.value.length)

  function onSearchInput(e: any) { searchQuery.value = e.target.value; displayedCount.value = 80 }
  function onSearchClear() { searchQuery.value = ''; displayedCount.value = 80 }
  function onSourceChange(e: any) { activeSource.value = e.target.value; displayedCount.value = 80 }
  function onDoseInput(e: any) { doseAmount.value = e.target.value }
  function onRouteChange(e: any) { route.value = e.target.value }
  function onTimestampInput(e: any) { timestamp.value = e.target.value }
  function loadMore() { displayedCount.value = Math.min(displayedCount.value + 80, filteredDrugs.value.length) }

  onMounted(async () => { allDrugs.value = await store.getAllDrugsWithSource() })

  async function selectDrug(drug: any) {
    selectedDrug.value = drug
    timestamp.value = getCurrentTimestamp()
    doseAmount.value = ''
    const routes = drug.routes || [{ route: 'oral', unit: drug.dose_unit || 'mg' }]
    route.value = routes[0]?.route || 'oral'
    await store.addDrug({
      drug_id: drug.drug_id, name: drug.name, model_type: drug.model_type,
      dose_unit: drug.dose_unit || 'mg',
      routes: drug.routes || [{ route: 'oral', unit: drug.dose_unit || 'mg' }],
      parameters: JSON.parse(JSON.stringify(drug.parameters)),
    })
  }

  async function saveDose(router: any) {
    if (!selectedDrug.value) return
    const amount = parseFloat(doseAmount.value)
    if (isNaN(amount) || amount <= 0) return
    const unit = currentDoseUnit.value
    let amountMG = amount
    if (unit === 'µg') amountMG = amount / 1000
    else if (unit === 'ng') amountMG = amount / 1000000
    else if (unit === 'pg') amountMG = amount / 1000000000
    else if (unit === 'mL') amountMG = amount
    await store.addDose({
      dose_id: crypto.randomUUID(),
      drug_id: selectedDrug.value.drug_id,
      dose_amount: amountMG,
      timestamp: new Date(timestamp.value).getTime() / 1000 / 3600,
      route_of_administration: route.value,
    })
    router.push({ name: 'home' })
  }

  return {
    searchQuery, activeSource, selectedDrug, doseAmount, route, timestamp,
    canSave, availableRoutes, currentDoseUnit, displayDrugs, hasMore,
    onSearchInput, onSearchClear, onSourceChange, onDoseInput, onRouteChange, onTimestampInput,
    loadMore, selectDrug, saveDose,
  }
}
