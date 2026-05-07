import { ref, computed } from 'vue'
import * as store from '../wasm/engineStore'

export function useHistory() {
  const doses = ref<any[]>([])
  const loading = ref(true)
  const deleteDialogOpen = ref(false)
  const pendingDelete = ref<any>(null)

  function formatTimestamp(ts: number) {
    const d = new Date(ts * 1000)
    return {
      date: d.toLocaleDateString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit' }),
      time: d.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' }),
    }
  }

  const groupedDoses = computed(() => {
    const groups = new Map<string, { date: string; items: any[] }>()
    const sorted = [...doses.value].sort((a, b) => b.timestamp - a.timestamp)
    for (const dose of sorted) {
      const { date, time } = formatTimestamp(dose.timestamp)
      dose.timeStr = time
      if (!groups.has(date)) groups.set(date, { date, items: [] })
      groups.get(date)!.items.push(dose)
    }
    return [...groups.values()]
  })

  async function loadData() {
    loading.value = true
    try { doses.value = await store.getAllDoses() }
    catch (e) { console.error('[HistoryPage] loadData failed:', e) }
    finally { loading.value = false }
  }

  function confirmDelete(dose: any) { pendingDelete.value = dose; deleteDialogOpen.value = true }

  async function doDelete() {
    if (!pendingDelete.value) return
    try {
      await store.removeDose(pendingDelete.value.dose_id)
      doses.value = doses.value.filter(d => d.dose_id !== pendingDelete.value.dose_id)
    } catch (e) { console.error('[HistoryPage] delete failed:', e) }
    deleteDialogOpen.value = false; pendingDelete.value = null
  }

  return { doses, loading, deleteDialogOpen, pendingDelete, groupedDoses, loadData, confirmDelete, doDelete }
}
