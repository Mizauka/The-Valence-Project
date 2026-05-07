import { ref, computed, onMounted } from 'vue'
import * as store from '../wasm/engineStore'

export function useCalibration() {
  const calibModel = ref('ratio')
  const labCV = ref(''); const labUnit = ref('pg/ml'); const labTS = ref('')
  const labResults = ref<any[]>([]); const drugSQ = ref(''); const drugGroups = ref<any[]>([]); const pendingGroup = ref<any>(null)

  const editDialogOpen = ref(false)
  const editingIndex = ref<number | null>(null)
  const editCV = ref(''); const editUnit = ref('pg/ml'); const editTS = ref('')

  const canAddLab = computed(() => labCV.value && parseFloat(labCV.value) > 0 && labTS.value && pendingGroup.value)
  const filteredGroups = computed(() => {
    const q = drugSQ.value.toLowerCase().trim()
    return q ? drugGroups.value.filter((g: any) => g.name.toLowerCase().includes(q) || g.id.toLowerCase().includes(q)) : drugGroups.value
  })

  onMounted(async () => {
    calibModel.value = await store.getCalibrationModel()
    labResults.value = await store.getLabResults() || []
    const all = await store.getAllDrugsWithSource()
    const seen = new Set<string>(); const gs: any[] = []
    for (const d of all) {
      const gid = d.group_id || d.drug_id
      if (!seen.has(gid)) { seen.add(gid); gs.push({ id: gid, name: d.group_id ? `${d.group_id} (${d.name})` : d.name }) }
    }
    drugGroups.value = gs
  })

  async function onModelChange(e: any) { calibModel.value = e.target.value; await store.setCalibrationModel(calibModel.value) }

  function selectGroup(g: any) { pendingGroup.value = g }

  async function addLab() {
    const v = parseFloat(labCV.value)
    if (!v || !labTS.value || !pendingGroup.value) return
    await store.addLabResult({ id: crypto.randomUUID(), time_h: new Date(labTS.value).getTime() / 3600000, conc_value: v, unit: labUnit.value, group_id: pendingGroup.value.id })
    labResults.value = await store.getLabResults() || []
    labCV.value = ''; labTS.value = ''; pendingGroup.value = null; drugSQ.value = ''
  }

  async function removeLab(i: number) {
    labResults.value.splice(i, 1)
    await store.clearLabResults()
    for (const l of labResults.value) await store.addLabResult(l)
  }

  function openEdit(i: number) {
    const l = labResults.value[i]; if (!l) return
    editingIndex.value = i
    editCV.value = String(l.conc_value); editUnit.value = l.unit || 'pg/ml'
    editTS.value = new Date(l.time_h * 3600000 - new Date().getTimezoneOffset() * 60000).toISOString().slice(0, 16)
    editDialogOpen.value = true
  }

  async function saveEdit() {
    const i = editingIndex.value
    if (i === null || !labResults.value[i]) return
    const v = parseFloat(editCV.value); if (!v || !editTS.value) return
    labResults.value[i] = { ...labResults.value[i], conc_value: v, unit: editUnit.value, time_h: new Date(editTS.value).getTime() / 3600000 }
    await store.clearLabResults()
    for (const l of labResults.value) await store.addLabResult(l)
    labResults.value = await store.getLabResults() || []
    editDialogOpen.value = false
  }

  return {
    calibModel, labCV, labUnit, labTS, labResults, drugSQ, drugGroups, pendingGroup,
    editDialogOpen, editingIndex, editCV, editUnit, editTS,
    canAddLab, filteredGroups,
    onModelChange, selectGroup, addLab, removeLab, openEdit, saveEdit,
  }
}
