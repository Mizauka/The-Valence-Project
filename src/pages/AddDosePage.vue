<template>
  <div class="page add-dose-page">
    <template v-if="step === 1">
      <div class="step-header">
        <h2 class="page-title">选择药物</h2>
        <p class="step-desc">搜索并选择你要记录的药物</p>
      </div>

      <div class="search-box">
        <mdui-text-field
          :value="searchQuery"
          label="搜索药物"
          variant="outlined"
          icon="search"
          clearable
          placeholder="输入药物名称..."
          @input="onSearchInput"
          @clear="onSearchClear"
        ></mdui-text-field>
      </div>

      <mdui-tabs :value="activeSource" @change="onSourceChange" class="source-tabs">
        <mdui-tab value="all">全部</mdui-tab>
        <mdui-tab value="hrt">HRT</mdui-tab>
        <mdui-tab value="journal">Journal</mdui-tab>
        <mdui-tab value="custom">自定义</mdui-tab>
      </mdui-tabs>

      <div class="drug-list" v-if="displayDrugs.length > 0">
        <mdui-card clickable
          v-for="drug in displayDrugs"
          :key="drug.drug_id"
          variant="outlined"
          class="drug-card"
          @click="selectDrug(drug)"
        >
          <div class="drug-card-content">
            <div class="drug-card-main">
              <span class="drug-card-name">{{ drug.name }}</span>
              <span class="drug-card-model">{{ modelLabel(drug.model_type) }}</span>
            </div>
            <div class="drug-card-meta">
              <span v-if="drug.source === 'hrt'" class="source-tag hrt">HRT</span>
              <span v-else-if="drug.source === 'journal'" class="source-tag journal">Journal</span>
              <span v-else class="source-tag custom">自定义</span>
              <span v-if="drug.parameters.equivalence_factor" class="eq-tag">
                等效={{ drug.parameters.equivalence_factor }}
              </span>
              <span class="hl-tag">t½={{ drug.parameters.half_life }}h</span>
            </div>
          </div>
        </mdui-card>
        <div v-if="hasMore" class="load-more" @click="loadMore">
          加载更多 ({{ displayedCount }}/{{ sourceFilteredDrugs.length }})
        </div>
      </div>

      <div class="empty-state" v-else>
        <mdui-icon name="search_off"></mdui-icon>
        <p>{{ searchQuery ? '未找到匹配药物' : (allDrugs.length === 0 ? '加载中...' : '该分类下暂无药物') }}</p>
      </div>
    </template>

    <template v-if="step === 2">
      <div class="step-header">
        <mdui-button-icon icon="arrow_back" @click="step = 1"></mdui-button-icon>
        <div>
          <h2 class="page-title">记录剂量</h2>
          <p class="step-desc">{{ selectedDrug?.name }}</p>
        </div>
      </div>

      <mdui-card variant="outlined" class="form-card">
        <div class="form-content">
          <div class="selected-drug-banner">
            <mdui-icon name="medication" class="banner-icon"></mdui-icon>
            <div class="banner-info">
              <span class="banner-name">{{ selectedDrug?.name }}</span>
              <span class="banner-detail">
                {{ modelLabel(selectedDrug?.model_type) }}
                <template v-if="selectedDrug?.parameters?.equivalence_factor">
                  · 等效系数={{ selectedDrug.parameters.equivalence_factor }}
                </template>
                · t½={{ selectedDrug?.parameters?.half_life }}h
              </span>
            </div>
          </div>

          <mdui-text-field
            :value="doseAmount"
            :label="'剂量 (' + currentDoseUnit + ')'"
            type="number"
            variant="outlined"
            @input="onDoseInput"
          ></mdui-text-field>

          <mdui-select
            :value="route"
            label="给药方式"
            variant="outlined"
            @change="onRouteChange"
          >
            <mdui-menu-item
              v-for="r in availableRoutes"
              :key="r.route"
              :value="r.route"
            >{{ routeLabel(r.route) }}</mdui-menu-item>
          </mdui-select>

          <mdui-text-field
            :value="timestamp"
            label="给药时间"
            type="datetime-local"
            variant="outlined"
            @input="onTimestampInput"
          ></mdui-text-field>

          <mdui-button variant="filled" full-width @click="saveDose" :disabled="!canSave">
            确认记录
          </mdui-button>
        </div>
      </mdui-card>
    </template>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import * as store from '../wasm/engineStore'

const router = useRouter()

const step = ref(1)
const searchQuery = ref('')
const activeSource = ref('all')
const allDrugs = ref([])
const selectedDrug = ref(null)
const doseAmount = ref('')
const route = ref('oral')
const timestamp = ref('')
const displayedCount = ref(80)

const canSave = computed(() => {
  const val = parseFloat(doseAmount.value)
  return !isNaN(val) && val > 0
})

const availableRoutes = computed(() => {
  if (!selectedDrug.value) return [{ route: 'oral', unit: 'mg' }]
  return selectedDrug.value.routes || [{ route: 'oral', unit: selectedDrug.value.dose_unit || 'mg' }]
})

const currentDoseUnit = computed(() => {
  const matched = availableRoutes.value.find(r => r.route === route.value)
  return matched ? matched.unit : (selectedDrug.value?.dose_unit || 'mg')
})

const routeLabels = {
  oral: '口服',
  sublingual: '舌下',
  buccal: '颊黏膜',
  insufflated: '鼻吸',
  rectal: '直肠',
  injection: '注射',
  transdermal: '透皮',
  gel: '凝胶',
  smoked: '吸入(烟)',
  inhaled: '吸入',
}

function routeLabel(r) {
  return routeLabels[r] || r
}

function onSearchInput(e) {
  searchQuery.value = e.target.value
  displayedCount.value = 80
}

function onSearchClear() {
  searchQuery.value = ''
  displayedCount.value = 80
}

function onSourceChange(e) {
  activeSource.value = e.target.value
  displayedCount.value = 80
}

function onDoseInput(e) {
  doseAmount.value = e.target.value
}

function onRouteChange(e) {
  route.value = e.target.value
}

function onTimestampInput(e) {
  timestamp.value = e.target.value
}

function getCurrentTimestamp() {
  const now = new Date()
  const offset = now.getTimezoneOffset()
  const local = new Date(now.getTime() - offset * 60000)
  return local.toISOString().slice(0, 16)
}

const sourceFilteredDrugs = computed(() => {
  if (activeSource.value === 'all') return allDrugs.value
  return allDrugs.value.filter(d => d.source === activeSource.value)
})

const filteredDrugs = computed(() => {
  const q = searchQuery.value.trim().toLowerCase()
  const sourceList = sourceFilteredDrugs.value
  if (!q) return sourceList
  return sourceList.filter(d =>
    d.name.toLowerCase().includes(q) ||
    d.drug_id.toLowerCase().includes(q)
  )
})

const displayDrugs = computed(() => {
  return filteredDrugs.value.slice(0, displayedCount.value)
})

const hasMore = computed(() => {
  return displayedCount.value < filteredDrugs.value.length
})

function loadMore() {
  displayedCount.value = Math.min(displayedCount.value + 80, filteredDrugs.value.length)
}

onMounted(async () => {
  allDrugs.value = await store.getAllDrugsWithSource()
})

async function selectDrug(drug) {
  selectedDrug.value = drug
  timestamp.value = getCurrentTimestamp()
  doseAmount.value = ''
  const routes = drug.routes || [{ route: 'oral', unit: drug.dose_unit || 'mg' }]
  route.value = routes[0]?.route || 'oral'
  step.value = 2

  await store.addDrug({
    drug_id: drug.drug_id,
    name: drug.name,
    model_type: drug.model_type,
    dose_unit: drug.dose_unit || 'mg',
    routes: drug.routes || [{ route: 'oral', unit: drug.dose_unit || 'mg' }],
    parameters: JSON.parse(JSON.stringify(drug.parameters)),
  })
}

async function saveDose() {
  if (!selectedDrug.value) return
  const amount = parseFloat(doseAmount.value)
  if (isNaN(amount) || amount <= 0) return

  const doseUnit = currentDoseUnit.value
  let amountMG = amount
  if (doseUnit === 'µg') amountMG = amount / 1000
  else if (doseUnit === 'ng') amountMG = amount / 1000000
  else if (doseUnit === 'pg') amountMG = amount / 1000000000
  else if (doseUnit === 'mL') amountMG = amount

  await store.addDose({
    dose_id: crypto.randomUUID(),
    drug_id: selectedDrug.value.drug_id,
    dose_amount: amountMG,
    timestamp: new Date(timestamp.value).getTime() / 1000 / 3600,
    route_of_administration: route.value,
  })

  router.push({ name: 'home' })
}

function modelLabel(modelType) {
  const map = {
    one_compartment: '一室',
    two_compartment: '二室',
    multi_compartment: '多室',
  }
  return map[modelType] || modelType
}
</script>

<style scoped>
.add-dose-page {
  margin: 0 auto;
  height: calc(100vh - 140px);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.step-header {
  display: flex;
  align-items: center;
  margin-bottom: 12px;
  flex-shrink: 0;
}

.step-header .page-title {
  font-size: 24px;
  font-weight: 600;
  margin: 0;
  line-height: 1.2;
}

.step-desc {
  font-size: 13px;
  opacity: 0.6;
  margin: 0;
}

.search-box {
  margin-bottom: 12px;
  flex-shrink: 0;
}

.source-tabs {
  flex-shrink: 0;
  margin-bottom: 8px;
}

.drug-list {
  flex: 1;
  overflow-y: auto;
}

.drug-card {
  padding: 12px 16px;
  margin-bottom: 8px;
  cursor: pointer;
  transition: background 0.15s;
}

.drug-card-content {
  display: flex;
  flex-direction: column;
}

.drug-card-main {
  display: flex;
  align-items: center;
}

.drug-card-name {
  font-weight: 600;
  font-size: 15px;
}

.drug-card-model {
  font-size: 11px;
  opacity: 0.5;
  background: rgba(0, 0, 0, 0.06);
  padding: 1px 6px;
  border-radius: 4px;
}

.drug-card-meta {
  display: flex;
  font-size: 11px;
  opacity: 0.6;
  align-items: center;
  flex-wrap: wrap;
}

.source-tag {
  padding: 1px 6px;
  border-radius: 4px;
  font-size: 10px;
  font-weight: 600;
  opacity: 1;
}

.source-tag.hrt {
  background: rgba(120, 0, 200, 0.12);
  color: #6a1b9a;
}

.source-tag.journal {
  background: rgba(0, 100, 200, 0.12);
  color: #1565c0;
}

.source-tag.custom {
  background: rgba(0, 80, 180, 0.12);
  color: #0d47a1;
}

.eq-tag {
  color: #1976d2;
  opacity: 1;
  font-weight: 600;
}

.hl-tag {
  opacity: 0.5;
}

.load-more {
  text-align: center;
  padding: 12px;
  font-size: 13px;
  opacity: 0.5;
  cursor: pointer;
}

.load-more:hover {
  opacity: 0.8;
}

.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  opacity: 0.4;
  padding: 48px 0;
}

.empty-state p {
  font-size: 14px;
}

.form-card {
  padding: 16px;
}

.form-content {
  display: flex;
  flex-direction: column;
}

.selected-drug-banner {
  display: flex;
  align-items: center;
  padding: 12px;
  background: var(--mdui-color-primary-container);
  border-radius: 12px;
}

.banner-icon {
  font-size: 28px;
  color: var(--mdui-color-primary);
}

.banner-info {
  display: flex;
  flex-direction: column;
}

.banner-name {
  font-weight: 600;
  font-size: 16px;
  color: var(--mdui-color-on-primary-container);
}

.banner-detail {
  font-size: 12px;
  opacity: 0.7;
  color: var(--mdui-color-on-primary-container);
}
</style>
