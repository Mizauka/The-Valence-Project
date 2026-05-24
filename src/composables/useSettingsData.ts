// ─── useSettingsData ──────────────────────────────────────────────
// Weight, data sync, export/import logic for Settings page.

import { ref, onMounted } from 'vue'
import {
  getWeight, setWeight,
  exportAllData, importAllData,
} from '../services/engineStore'
import {
  pickDataDirectory, getExternalDirName,
  checkExternalDirPermission, requestExternalDirPermission,
} from '../services/persistence'

export function useSettingsData() {
  const weight = ref(60)
  const weightDisplay = ref('60')
  const savedDirName = ref<string | null>(null)
  const dirPermissionGranted = ref(false)
  const importInput = ref<HTMLInputElement | null>(null)
  const importing = ref(false)
  const exporting = ref(false)
  const syncMessage = ref('')

  onMounted(async () => {
    try {
      const w = await getWeight()
      if (w > 0) { weight.value = w; weightDisplay.value = String(w) }
    } catch { /* engine may not be ready */ }

    try {
      const name = await getExternalDirName()
      if (name) {
        savedDirName.value = name
        dirPermissionGranted.value = await checkExternalDirPermission()
      }
    } catch { /* not supported */ }
  })

  function onWeightInput(e: Event) {
    const val = (e.target as HTMLInputElement).value
    weightDisplay.value = val
    const num = parseFloat(val)
    if (!isNaN(num) && num > 0) weight.value = num
  }

  async function saveWeight() {
    const v = parseFloat(weightDisplay.value)
    if (!isNaN(v) && v > 0) {
      weight.value = v
      await setWeight(v)
    }
  }

  async function syncToFolder() {
    try {
      syncMessage.value = '正在选择文件夹...'
      savedDirName.value = await pickDataDirectory()
      dirPermissionGranted.value = true
      syncMessage.value = '同步成功！数据已保存到 ' + savedDirName.value
      setTimeout(() => syncMessage.value = '', 3000)
    } catch (e: any) {
      console.error('[Settings] sync failed:', e)
      syncMessage.value = '同步失败: ' + (e.message || '未知错误')
    }
  }

  async function reauthorizeDir() {
    try {
      const granted = await requestExternalDirPermission()
      dirPermissionGranted.value = granted
      if (granted) savedDirName.value = await getExternalDirName()
    } catch (e: any) {
      console.error('[Settings] reauthorize failed:', e)
    }
  }

  async function doExport() {
    exporting.value = true
    try {
      const jsonStr = await exportAllData()
      const blob = new Blob([jsonStr], { type: 'application/json' })
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `valence-export-${new Date().toISOString().slice(0, 10)}.json`
      a.click()
      URL.revokeObjectURL(url)
    } catch (e: any) {
      console.error('[Settings] export failed:', e)
    } finally {
      exporting.value = false
    }
  }

  function triggerImport() {
    importInput.value?.click()
  }

  async function doImport(event: Event) {
    const file = (event.target as HTMLInputElement).files?.[0]
    if (!file) return
    importing.value = true
    const reader = new FileReader()
    reader.onload = async (e) => {
      try {
        await importAllData(e.target?.result as string)
        const w = await getWeight()
        if (w > 0) { weight.value = w; weightDisplay.value = String(w) }
        syncMessage.value = '导入成功！'
        setTimeout(() => syncMessage.value = '', 3000)
      } catch (err: any) {
        console.error('Import failed:', err)
        syncMessage.value = '导入失败: ' + (err.message || '未知错误')
      } finally {
        importing.value = false
      }
    }
    reader.readAsText(file)
  }

  return {
    weight, weightDisplay, savedDirName, dirPermissionGranted,
    importInput, importing, exporting, syncMessage,
    onWeightInput, saveWeight, syncToFolder, reauthorizeDir,
    doExport, triggerImport, doImport,
  }
}
