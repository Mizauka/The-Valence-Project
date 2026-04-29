<template>
  <div class="page history-page">
    <h2 class="page-title">给药记录</h2>

    <div v-if="loading" class="loading-state">
      <mdui-circular-progress></mdui-circular-progress>
      <p>加载中...</p>
    </div>

    <div v-else-if="doses.length === 0" class="empty-state">
      <mdui-icon name="history"></mdui-icon>
      <p>暂无给药记录</p>
      <mdui-button variant="tonal" @click="goAddDose">记录剂量</mdui-button>
    </div>

    <div v-else class="dose-list">
      <mdui-card
        v-for="group in groupedDoses"
        :key="group.date"
        variant="outlined"
        class="date-group"
      >
        <div class="date-header">{{ group.date }}</div>
        <mdui-list>
          <mdui-list-item
            v-for="dose in group.items"
            :key="dose.dose_id"
            class="dose-item"
          >
            <div class="dose-row">
              <div class="dose-left">
                <mdui-icon name="medication" class="dose-icon"></mdui-icon>
                <div class="dose-info">
                  <span class="dose-drug-name">{{ dose.drugName }}</span>
                  <span class="dose-meta">
                    {{ formatDose(dose) }} · {{ routeLabel(dose.route_of_administration) }} · {{ dose.timeStr }}
                  </span>
                </div>
              </div>
              <mdui-button-icon icon="delete" @click="confirmDelete(dose)"></mdui-button-icon>
            </div>
          </mdui-list-item>
        </mdui-list>
      </mdui-card>
    </div>

    <mdui-dialog
      :open="deleteDialogOpen"
      headline="确认删除"
      @closed="deleteDialogOpen = false"
    >
      确定要删除这条给药记录吗？此操作不可撤销。
      <mdui-button slot="action" variant="text" @click="deleteDialogOpen = false">取消</mdui-button>
      <mdui-button slot="action" variant="tonal" @click="doDelete">删除</mdui-button>
    </mdui-dialog>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import * as store from '../wasm/engineStore'

const router = useRouter()

const doses = ref([])
const loading = ref(true)
const deleteDialogOpen = ref(false)
const pendingDelete = ref(null)

const routeLabels = {
  oral: '口服',
  injection: '注射',
  sublingual: '舌下',
  buccal: '颊黏膜',
  insufflated: '鼻吸',
  transdermal: '透皮',
  gel: '凝胶',
  patch: '贴片',
  rectal: '直肠',
  smoked: '吸入(烟)',
  inhaled: '吸入',
  inhalation: '吸入',
}

function routeLabel(key) {
  return routeLabels[key] || key
}

function formatDose(dose) {
  const val = dose.display_amount ?? dose.dose_amount
  const unit = dose.display_unit || 'mg'
  if (Number.isInteger(val) || Math.abs(val) >= 10) return `${val} ${unit}`
  if (Math.abs(val) >= 1) return `${val.toFixed(1)} ${unit}`
  return `${val.toFixed(2)} ${unit}`
}

function formatTimestamp(ts) {
  const d = new Date(ts * 1000)
  const date = d.toLocaleDateString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit' })
  const time = d.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' })
  return { date, time }
}

const groupedDoses = computed(() => {
  const groups = new Map()
  const sorted = [...doses.value].sort((a, b) => b.timestamp - a.timestamp)

  for (const dose of sorted) {
    const { date, time: timeStr } = formatTimestamp(dose.timestamp)
    dose.timeStr = timeStr

    if (!groups.has(date)) {
      groups.set(date, { date, items: [] })
    }
    groups.get(date).items.push(dose)
  }

  return [...groups.values()]
})

async function loadData() {
  loading.value = true
  try {
    doses.value = await store.getAllDoses()
  } catch (e) {
    console.error('[HistoryPage] loadData failed:', e)
  } finally {
    loading.value = false
  }
}

function confirmDelete(dose) {
  pendingDelete.value = dose
  deleteDialogOpen.value = true
}

async function doDelete() {
  if (!pendingDelete.value) return
  try {
    await store.removeDose(pendingDelete.value.dose_id)
    doses.value = doses.value.filter(d => d.dose_id !== pendingDelete.value.dose_id)
  } catch (e) {
    console.error('[HistoryPage] deleteDose failed:', e)
  }
  deleteDialogOpen.value = false
  pendingDelete.value = null
}

function goAddDose() {
  router.push({ name: 'add-dose' })
}

onMounted(loadData)
</script>

<style scoped>
.history-page {
  max-width: 600px;
  margin: 0 auto;
}

.page-title {
  font-size: 24px;
  font-weight: 600;
  margin-bottom: 16px;
}

.loading-state,
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 64px 0;
  opacity: 0.6;
}

.empty-state p {
  font-size: 14px;
}

.dose-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.date-group {
  overflow: hidden;
}

.date-header {
  padding: 10px 16px;
  font-size: 13px;
  font-weight: 600;
  opacity: 0.6;
  background: var(--mdui-color-surface-container);
}

.dose-item {
  --mdui-comp-list-item-padding: 8px 16px;
}

.dose-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
}

.dose-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.dose-icon {
  font-size: 20px;
  color: var(--mdui-color-primary);
}

.dose-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.dose-drug-name {
  font-weight: 600;
  font-size: 14px;
}

.dose-meta {
  font-size: 12px;
  opacity: 0.6;
}
</style>
