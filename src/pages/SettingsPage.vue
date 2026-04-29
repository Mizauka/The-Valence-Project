<template>
  <div class="page settings-page">
    <h2 class="page-title">设置</h2>

    <mdui-card variant="filled" class="settings-card">
      <div class="settings-content">
        <h3 class="card-title">体重设置</h3>
        <p class="card-desc">体重将作为常量用于药代动力学计算的分布容积调整</p>
        <mdui-text-field
          :value="weightDisplay"
          label="体重 (kg)"
          type="number"
          variant="outlined"
          @input="onWeightInput"
          @change="saveWeight"
        ></mdui-text-field>
      </div>
    </mdui-card>

    <mdui-card variant="filled" class="settings-card">
      <div class="settings-content">
        <h3 class="card-title">数据存储</h3>
        <p class="card-desc">数据默认使用浏览器持久化存储（OPFS），关闭浏览器后数据不会丢失。点击下方按钮可将当前所有数据同步保存到指定文件夹。</p>
        <div class="action-buttons">
          <mdui-button variant="filled" @click="syncToFolder">
            <mdui-icon slot="icon" name="folder_open"></mdui-icon>
            选择并同步到本地文件夹
          </mdui-button>
          <mdui-button v-if="savedDirName && !dirPermissionGranted" variant="outlined" @click="reauthorizeDir">
            <mdui-icon slot="icon" name="lock_open"></mdui-icon>
            重新授权同步
          </mdui-button>
        </div>
        <p v-if="savedDirName" class="dir-info">
          同步目录: {{ savedDirName }}/data/
          <span v-if="dirPermissionGranted" class="dir-status">● 已授权</span>
          <span v-else class="dir-status dir-status--revoked">● 需重新授权</span>
        </p>
      </div>
    </mdui-card>

    <mdui-card variant="filled" class="settings-card">
      <div class="settings-content">
        <h3 class="card-title">数据管理</h3>
        <div class="action-buttons">
          <mdui-button variant="tonal" @click="exportData">
            <mdui-icon slot="icon" name="download"></mdui-icon>
            导出数据
          </mdui-button>
          <mdui-button variant="tonal" @click="triggerImport">
            <mdui-icon slot="icon" name="upload"></mdui-icon>
            导入数据
          </mdui-button>
          <input
            ref="importInput"
            type="file"
            accept="application/json"
            class="hidden-input"
            @change="importData"
          />
        </div>
      </div>
    </mdui-card>

    <mdui-card variant="filled" class="settings-card">
      <div class="settings-content">
        <h3 class="card-title">关于</h3>
        <p class="about-text">Valence - 开源药物管理与血药浓度追踪</p>
        <p class="about-text">v0.1.0</p>
        <p class="about-text">所有数据存储在本地，注重隐私保护</p>
      </div>
    </mdui-card>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import * as store from '../wasm/engineStore'

const DEFAULT_WEIGHT = 60

const weight = ref(DEFAULT_WEIGHT)
const weightDisplay = ref(String(DEFAULT_WEIGHT))
const importInput = ref(null)
const savedDirName = ref('')
const dirPermissionGranted = ref(false)

onMounted(async () => {
  const w = await store.getWeight()
  if (w > 0) {
    weight.value = w
    weightDisplay.value = String(w)
  }

  const name = await store.getExternalDirName()
  if (name) {
    savedDirName.value = name
    dirPermissionGranted.value = await store.checkExternalDirPermission()
  }
})

function onWeightInput(e) {
  const val = e.target.value
  weightDisplay.value = val
  const num = parseFloat(val)
  if (!isNaN(num) && num > 0) {
    weight.value = num
  }
}

async function saveWeight() {
  const current = parseFloat(weightDisplay.value)
  if (!isNaN(current) && current > 0) {
    weight.value = current
    await store.setWeight(current)
  }
}

async function syncToFolder() {
  try {
    const name = await store.pickDataDirectory()
    savedDirName.value = name
    dirPermissionGranted.value = true
  } catch (e) {
    console.error('[Settings] sync failed:', e)
    alert('同步失败: ' + e.message)
  }
}

async function reauthorizeDir() {
  try {
    const granted = await store.requestExternalDirPermission()
    dirPermissionGranted.value = granted
    if (granted) {
      const name = await store.getExternalDirName()
      savedDirName.value = name
    }
  } catch (e) {
    console.error('[Settings] reauthorize failed:', e)
  }
}

async function exportData() {
  try {
    const jsonStr = await store.exportAllData()
    const blob = new Blob([jsonStr], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `valence-export-${new Date().toISOString().slice(0, 10)}.json`
    a.click()
    URL.revokeObjectURL(url)
  } catch (e) {
    console.error('[Settings] export failed:', e)
  }
}

function triggerImport() {
  importInput.value?.click()
}

async function importData(event) {
  const file = event.target.files?.[0]
  if (!file) return
  const reader = new FileReader()
  reader.onload = async (e) => {
    try {
      const text = e.target.result
      await store.importAllData(text)

      const w = await store.getWeight()
      if (w > 0) {
        weight.value = w
        weightDisplay.value = String(w)
      }
    } catch (err) {
      console.error('Import failed:', err)
    }
  }
  reader.readAsText(event)
  event.target.value = ''
}
</script>

<style scoped>
.settings-page {
  max-width: 600px;
  margin: 0 auto;
}

.page-title {
  font-size: 24px;
  font-weight: 600;
  margin-bottom: 16px;
}

.settings-card {
  padding: 16px;
  margin-bottom: 12px;
}

.settings-content {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.card-title {
  font-size: 16px;
  font-weight: 600;
}

.card-desc {
  font-size: 13px;
  opacity: 0.7;
  line-height: 1.5;
}

.action-buttons {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.dir-info {
  font-size: 13px;
  color: var(--mdui-color-primary);
  opacity: 0.8;
}

.dir-status {
  margin-left: 6px;
  font-size: 12px;
}

.dir-status--revoked {
  color: var(--mdui-color-error, #f44336);
  opacity: 1;
}

.hidden-input {
  display: none;
}

.about-text {
  font-size: 13px;
  opacity: 0.6;
  line-height: 1.6;
}
</style>
