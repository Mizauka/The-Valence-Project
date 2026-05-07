<template>
  <div class="sliding-page" :data-step="panels.step">
    <div class="sliding-viewport">
      <mdui-button-icon icon="arrow_back" @click="handleBack" />

      <div class="sliding-track"
        :style="{ width: panels.trackWidthPercent + '%', transform: `translateX(-${panels.offsetPercent}%)` }">

        <!-- Panel 0: Dose List -->
        <div class="sliding-panel"><div class="sliding-panel-inner">
          <div v-if="loading" class="loading-state">
            <mdui-circular-progress></mdui-circular-progress><p>加载中...</p>
          </div>
          <div v-else-if="doses.length === 0" class="empty-state">
            <mdui-icon name="history"></mdui-icon><p>暂无给药记录</p>
            <mdui-button variant="tonal" @click="goAddDose">记录剂量</mdui-button>
          </div>
          <div v-else class="dose-list-custom">
            <div v-for="group in groupedDoses" :key="group.date" class="date-group">
              <div class="date-header">{{ group.date }}</div>
              <mdui-list>
                <mdui-list-item v-for="dose in group.items" :key="dose.dose_id" class="dose-item-custom">
                  <div class="dose-row">
                    <div class="dose-left">
                      <mdui-icon name="medication" class="dose-icon"></mdui-icon>
                      <div class="dose-info">
                        <span class="dose-drug-name">{{ dose.drugName }}</span>
                        <span class="dose-meta">{{ formatDose(dose) }} · {{ routeLabel(dose.route_of_administration) }} · {{ dose.timeStr }}</span>
                      </div>
                    </div>
                    <div class="dose-actions">
                      <mdui-button-icon icon="edit" @click.stop="openEdit(dose)"></mdui-button-icon>
                      <mdui-button-icon icon="delete" @click.stop="confirmDelete(dose)"></mdui-button-icon>
                    </div>
                  </div>
                </mdui-list-item>
              </mdui-list>
            </div>
          </div>
        </div></div>

        <!-- Panel 1: Edit Dose -->
        <div class="sliding-panel"><div class="sliding-panel-inner">
            <template v-if="!panels.showPlaceholder">
          <div class="step-header">
            <p class="step-desc">{{ editingDrug?.name || '编辑剂量' }}</p>
          </div>
          <DoseForm
            v-if="editingDrug"
            :drug="editingDrug"
            :doseAmount="editAmount"
            :route="editRoute"
            :timestamp="editTimestamp"
            :routes="editRoutes"
            :currentDoseUnit="editDoseUnit"
            :canSave="editCanSave"
            saveLabel="保存修改"
            @update:doseAmount="v => editAmount = v"
            @update:route="v => editRoute = v"
            @update:timestamp="v => editTimestamp = v"
            @save="saveEdit"
          />
          </template>
          <template v-else>
            <div class="panel-placeholder">
              <mdui-icon :name="panels.placeholderIcon"></mdui-icon>
              <p>{{ panels.placeholderText }}</p>
            </div>
          </template>
        </div></div>

      </div>
    </div>

    <mdui-dialog :open="deleteDialogOpen" headline="确认删除" @closed="deleteDialogOpen = false">
      确定要删除这条给药记录吗？此操作不可撤销。
      <mdui-button slot="action" variant="text" @click="deleteDialogOpen = false">取消</mdui-button>
      <mdui-button slot="action" variant="tonal" @click="doDelete">删除</mdui-button>
    </mdui-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import * as store from '../wasm/engineStore'
import { useSlidingPanels } from '../composables/useSlidingPanels'
import { formatDose as fmtDose, routeLabel } from '../utils/format'
import DoseForm from '../components/DoseForm.vue'

function formatDose(dose: any) { return fmtDose(dose.display_amount ?? dose.dose_amount, dose.display_unit || 'mg') }

const router = useRouter()
const panels = useSlidingPanels(2, { icon: 'edit_note', text: '在左侧选择记录后在此编辑' })
function handleBack() { if (!panels.back()) router.push({ name: 'home' }) }

onMounted(async () => {
  loadData()
})

const doses = ref<any[]>([]); const loading = ref(true)
const deleteDialogOpen = ref(false); const pendingDelete = ref<any>(null)

const editingDose = ref<any>(null); const editingDrug = ref<any>(null)
const editAmount = ref(''); const editRoute = ref('oral'); const editTimestamp = ref('')
const editRoutes = computed(() => [{ route: 'oral', unit: 'mg' }])
const editDoseUnit = computed(() => 'mg')
const editCanSave = computed(() => parseFloat(editAmount.value) > 0 && editTimestamp.value.length > 0)

function formatTimestamp(ts: number) {
  const d = new Date(ts * 1000)
  return {
    date: d.toLocaleDateString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit' }),
    time: d.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' }),
  }
}

const groupedDoses = computed(() => {
  const groups = new Map()
  const sorted = [...doses.value].sort((a, b) => b.timestamp - a.timestamp)
  for (const dose of sorted) {
    const { date, time } = formatTimestamp(dose.timestamp)
    dose.timeStr = time
    if (!groups.has(date)) groups.set(date, { date, items: [] })
    groups.get(date).items.push(dose)
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

function openEdit(dose: any) {
  editingDose.value = dose
  editingDrug.value = { name: dose.drugName, drug_id: dose.drug_id, model_type: 'one_compartment', parameters: {} }
  const ts = new Date(dose.timestamp * 1000)
  const local = new Date(ts.getTime() - ts.getTimezoneOffset() * 60000)
  editAmount.value = String(dose.display_amount ?? dose.dose_amount)
  editRoute.value = dose.route_of_administration || 'oral'
  editTimestamp.value = local.toISOString().slice(0, 16)
  panels.markSelection()
  panels.advance(0)
}

async function saveEdit() {
  if (!editingDose.value || !editAmount.value || !editTimestamp.value) return
  const amount = parseFloat(editAmount.value)
  if (isNaN(amount) || amount <= 0) return
  const newTs = new Date(editTimestamp.value).getTime() / 1000 / 3600
  await store.removeDose(editingDose.value.dose_id)
  await store.addDose({
    dose_id: editingDose.value.dose_id,
    drug_id: editingDose.value.drug_id,
    dose_amount: amount,
    timestamp: newTs,
    route_of_administration: editRoute.value,
  })
  doses.value = await store.getAllDoses()
  panels.reset()
}

function goAddDose() { router.push({ name: 'add-dose' }) }
</script>
