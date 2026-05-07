<template>
  <div class="sliding-page" :data-step="panels.step">

    <mdui-button-icon icon="arrow_back" @click="handleBack" />

    <div class="sliding-viewport">
      <div class="sliding-track"
        :style="{ width: panels.trackWidthPercent + '%', transform: `translateX(-${panels.offsetPercent}%)` }">

        <!-- Panel 0: Model -->
        <div class="sliding-panel">
          <div class="sliding-panel-inner">
            <h3>校准模型</h3>
            <p class="step-desc">选择后验校准算法以优化血药浓度预测</p>
            <mdui-segmented-button-group :value="calibModel" selects="single" @change="onModelChange">
              <mdui-segmented-button value="ratio">比率插值</mdui-segmented-button>
              <mdui-segmented-button value="ou-kalman">OU-Kalman</mdui-segmented-button>
            </mdui-segmented-button-group>
            <div class="cal-model-info">
              <p v-if="calibModel === 'ratio'">基于实测值与预测值的比率构建分段线性插值器，适用于少量观测数据。</p>
              <p v-else>基于 Ornstein-Uhlenbeck 过程的卡尔曼滤波器 + RTS 反向平滑器，适用于有规律的采血检测。</p>
            </div>

            <div class="step-header">
              <h3>校准记录</h3>
              <mdui-button variant="filled" size="small" @click="panels.advance(1)"><mdui-icon slot="icon"
                  name="add" />添加</mdui-button>
            </div>
            <mdui-list v-if="labResults.length" class="cal-lab-list-native">
              <mdui-list-item v-for="(l, i) in labResults" :key="i" clickable @click="openEdit(i)">
                <div class="cal-lab-item-row">
                  <span v-if="l.group_id" class="cal-lab-group">{{ l.group_id }}</span>
                  <span class="cal-lab-val">{{ l.conc_value }} {{ l.unit }}</span>
                  <span class="cal-lab-time">{{ fmt(l.time_h) }}</span>
                </div>
                <mdui-icon slot="end" name="edit" class="cal-lab-edit-icon" @click.stop="openEdit(i)" />
                <mdui-icon slot="end" name="delete" class="cal-lab-del-icon" @click.stop="removeLab(i)" />
              </mdui-list-item>
            </mdui-list>
            <div v-else class="empty-state">
              <p>暂无校准记录</p>
              <p class="empty-sub">点击「添加」选择物质组</p>
            </div>
          </div>
        </div>

        <!-- Panel 1: Drug Select -->
        <div class="sliding-panel">
          <div class="sliding-panel-inner">
            <h3>选择物质组</h3>
            <p class="step-desc">选择需要校准的目标药物组</p>
            <mdui-text-field :value="drugSQ" label="搜索药物" variant="outlined" icon="search" clearable
              placeholder="输入药物名称..." @input="e => drugSQ = e.target.value" @clear="drugSQ = ''" />
            <mdui-list v-if="filteredGroups.length" class="cal-drug-list-native">
              <mdui-list-item v-for="g in filteredGroups" :key="g.id" clickable @click="selectGroup(g)">
                <span class="cal-drug-name">{{ g.name }}</span>
                <span slot="description" class="cal-drug-model">group</span>
              </mdui-list-item>
            </mdui-list>
            <div v-else class="empty-state"><p>未找到匹配药物</p></div>
          </div>
        </div>

        <!-- Panel 2: Lab Form -->
        <div class="sliding-panel">
          <div class="sliding-panel-inner">
            <h3>添加血检数据</h3>
            <div class="cal-selected-banner" v-if="pendingGroup">
              <mdui-icon name="medication" />
              <div><span class="cal-banner-name">{{ pendingGroup.name }}</span><span class="cal-banner-id">group_id: {{
                pendingGroup.id }}</span></div>
            </div>
            <p v-else class="empty-state">未选择物质组</p>
            <div v-if="pendingGroup" class="form-content">
              <mdui-text-field :value="labCV" label="血药浓度值" type="number" variant="outlined"
                @input="e => labCV = e.target.value" />
              <mdui-select :value="labUnit" label="单位" variant="outlined" @change="e => labUnit = e.target.value">
                <mdui-menu-item value="pg/ml">pg/mL</mdui-menu-item>
                <mdui-menu-item value="pmol/l">pmol/L</mdui-menu-item>
              </mdui-select>
              <mdui-text-field :value="labTS" label="采样时间" type="datetime-local" variant="outlined"
                @input="e => labTS = e.target.value" />
              <mdui-button variant="filled" full-width :disabled="!canAddLab" @click="addLab"><mdui-icon slot="icon"
                  name="check" />确认添加</mdui-button>
            </div>
          </div>
        </div>

      </div>
    </div>

    <!-- Edit Dialog -->
    <mdui-dialog :open="editDialogOpen" headline="编辑校准记录" @closed="editDialogOpen = false">
      <div class="edit-dialog-form" v-if="editingIndex !== null && labResults[editingIndex]">
        <mdui-text-field :value="editCV" label="血药浓度值" type="number" variant="outlined"
          @input="e => editCV = e.target.value" />
        <mdui-select :value="editUnit" label="单位" variant="outlined" @change="e => editUnit = e.target.value">
          <mdui-menu-item value="pg/ml">pg/mL</mdui-menu-item>
          <mdui-menu-item value="pmol/l">pmol/L</mdui-menu-item>
        </mdui-select>
        <mdui-text-field :value="editTS" label="采样时间" type="datetime-local" variant="outlined"
          @input="e => editTS = e.target.value" />
      </div>
      <mdui-button slot="action" variant="text" @click="editDialogOpen = false">取消</mdui-button>
      <mdui-button slot="action" variant="tonal" @click="saveEdit">保存</mdui-button>
    </mdui-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import * as store from '../wasm/engineStore'
import { useSlidingPanels } from '../composables/useSlidingPanels'

const router = useRouter()
const panels = useSlidingPanels(3)

function handleBack() { if (!panels.back()) router.push({ name: 'home' }) }

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
  const seen = new Set<string>()
  const gs: any[] = []
  for (const d of all) {
    const gid = d.group_id || d.drug_id
    if (!seen.has(gid)) {
      seen.add(gid)
      gs.push({ id: gid, name: d.group_id ? `${d.group_id} (${d.name})` : d.name })
    }
  }
  drugGroups.value = gs
})

async function onModelChange(e: any) { calibModel.value = e.target.value; await store.setCalibrationModel(calibModel.value) }

function selectGroup(g: any) {
  pendingGroup.value = g
  panels.advance(2)
}

async function addLab() {
  const v = parseFloat(labCV.value)
  if (!v || !labTS.value || !pendingGroup.value) return
  await store.addLabResult({
    id: crypto.randomUUID(),
    time_h: new Date(labTS.value).getTime() / 3600000,
    conc_value: v,
    unit: labUnit.value,
    group_id: pendingGroup.value.id,
  })
  labResults.value = await store.getLabResults() || []
  labCV.value = ''; labTS.value = ''; pendingGroup.value = null; drugSQ.value = ''
  panels.reset()
}

async function removeLab(i: number) {
  labResults.value.splice(i, 1)
  await store.clearLabResults()
  for (const l of labResults.value) await store.addLabResult(l)
}

function openEdit(i: number) {
  const l = labResults.value[i]
  if (!l) return
  editingIndex.value = i
  editCV.value = String(l.conc_value)
  editUnit.value = l.unit || 'pg/ml'
  editTS.value = new Date(l.time_h * 3600000 - new Date().getTimezoneOffset() * 60000).toISOString().slice(0, 16)
  editDialogOpen.value = true
}

async function saveEdit() {
  const i = editingIndex.value
  if (i === null || !labResults.value[i]) return
  const v = parseFloat(editCV.value)
  if (!v || !editTS.value) return
  labResults.value[i] = {
    ...labResults.value[i],
    conc_value: v,
    unit: editUnit.value,
    time_h: new Date(editTS.value).getTime() / 3600000,
  }
  await store.clearLabResults()
  for (const l of labResults.value) await store.addLabResult(l)
  labResults.value = await store.getLabResults() || []
  editDialogOpen.value = false
}

function fmt(th: number) { return new Date(th * 3600000).toLocaleString() }
</script>
