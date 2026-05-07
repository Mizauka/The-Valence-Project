import { ref, onMounted } from 'vue'
import * as store from '../wasm/engineStore'

export function useSettings() {
  const weight = ref(60)
  const weightDisplay = ref('60')
  const importInput = ref<HTMLInputElement | null>(null)
  const savedDirName = ref('')
  const dirPermissionGranted = ref(false)

  onMounted(async () => {
    const w = await store.getWeight()
    if (w > 0) { weight.value = w; weightDisplay.value = String(w) }
    const name = await store.getExternalDirName()
    if (name) { savedDirName.value = name; dirPermissionGranted.value = await store.checkExternalDirPermission() }
  })

  function onWeightInput(e: any) {
    weightDisplay.value = e.target.value
    const num = parseFloat(e.target.value)
    if (!isNaN(num) && num > 0) weight.value = num
  }

  async function saveWeight() {
    const v = parseFloat(weightDisplay.value)
    if (!isNaN(v) && v > 0) { weight.value = v; await store.setWeight(v) }
  }

  async function syncToFolder() {
    try { savedDirName.value = await store.pickDataDirectory(); dirPermissionGranted.value = true }
    catch (e: any) { console.error('[Settings] sync failed:', e); alert('同步失败: ' + e.message) }
  }

  async function reauthorizeDir() {
    try {
      const granted = await store.requestExternalDirPermission()
      dirPermissionGranted.value = granted
      if (granted) savedDirName.value = await store.getExternalDirName()
    } catch (e: any) { console.error('[Settings] reauthorize failed:', e) }
  }

  async function exportData() {
    try {
      const jsonStr = await store.exportAllData()
      const blob = new Blob([jsonStr], { type: 'application/json' })
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url; a.download = `valence-export-${new Date().toISOString().slice(0, 10)}.json`; a.click()
      URL.revokeObjectURL(url)
    } catch (e: any) { console.error('[Settings] export failed:', e) }
  }

  function triggerImport() { importInput.value?.click() }

  async function importData(event: any) {
    const file = event.target.files?.[0]; if (!file) return
    const reader = new FileReader()
    reader.onload = async (e) => {
      try {
        await store.importAllData(e.target?.result as string)
        const w = await store.getWeight()
        if (w > 0) { weight.value = w; weightDisplay.value = String(w) }
      } catch (err) { console.error('Import failed:', err) }
    }
    reader.readAsText(file)
  }

  return {
    weight, weightDisplay, importInput, savedDirName, dirPermissionGranted,
    onWeightInput, saveWeight, syncToFolder, reauthorizeDir, exportData, triggerImport, importData,
  }
}
