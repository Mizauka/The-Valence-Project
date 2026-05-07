<template>
  <div class="sliding-page" :data-step="panels.step">
    <div class="sliding-viewport">
      <div class="sliding-track"
        :style="{ width: panels.trackWidthPercent + '%', transform: `translateX(-${panels.offsetPercent}%)` }">
        <div class="sliding-panel">
          <div class="sliding-panel-inner">

            <mdui-list variant="filled" class="settings-card">
              <div class="settings-content">
                <h3>体重设置</h3>
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
            </mdui-list>

            <mdui-list variant="filled" class="settings-card">
              <div class="settings-content">
                <h3>数据存储</h3>
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
            </mdui-list>

            <mdui-list variant="filled" class="settings-card">
              <div class="settings-content">
                <h3>数据管理</h3>
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
            </mdui-list>

            <mdui-list variant="filled" class="settings-card">
              <div class="settings-content">
                <h3>关于</h3>
                <p class="about-text">Valence - 开源药物管理与血药浓度追踪</p>
                <p class="about-text">v0.1.0</p>
                <p class="about-text">所有数据存储在本地，注重隐私保护</p>
              </div>
            </mdui-list>

          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import * as store from '../wasm/engineStore'
import { useSlidingPanels } from '../composables/useSlidingPanels'

const panels = useSlidingPanels(1)

const DEFAULT_WEIGHT = 60

const weight = ref(DEFAULT_WEIGHT)
const weightDisplay = ref(String(DEFAULT_WEIGHT))
const importInput = ref<HTMLInputElement | null>(null)
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

function onWeightInput(e: any) {
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
  } catch (e: any) {
    console.error('[Settings] sync failed:', e)
    alert('同步失败: ' + (e?.message || e))
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
  } catch (e: any) {
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

async function importData(event: any) {
  const file = event.target.files?.[0]
  if (!file) return
  const reader = new FileReader()
  reader.onload = async (e: any) => {
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
