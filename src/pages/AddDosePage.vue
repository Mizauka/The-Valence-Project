<template>
  <div class="sliding-page" :data-step="panels.step">
    <div class="sliding-viewport">
      <mdui-button-icon icon="arrow_back" @click="handleBack" />

      <div class="sliding-track"
        :style="{ width: panels.trackWidthPercent + '%', transform: `translateX(-${panels.offsetPercent}%)` }">

        <!-- Panel 0: Drug Select -->
        <div class="sliding-panel">
          <div class="sliding-panel-inner">
            <DrugBrowser :searchQuery="searchQuery" :activeSource="activeSource" :displayDrugs="displayDrugs"
              :totalCount="sourceFilteredDrugs.length" :hasMore="hasMore"
              @update:searchQuery="q => { searchQuery = q; displayedCount = 80 }"
              @update:activeSource="s => { activeSource = s; displayedCount = 80 }" @select="selectDrug"
              @loadMore="loadMore" />
          </div>
        </div>

        <!-- Panel 1: Dose Form -->
        <div class="sliding-panel">
          <div class="sliding-panel-inner">
            <template v-if="!panels.showPlaceholder">
            <mdui-card variant="" class="form-card">
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

                <mdui-text-field :value="doseAmount" :label="'剂量 (' + currentDoseUnit + ')'" type="number"
                  variant="outlined" @input="onDoseInput"></mdui-text-field>

                <mdui-select :value="route" label="给药方式" variant="outlined" @change="onRouteChange">
                  <mdui-menu-item v-for="r in availableRoutes" :key="r.route" :value="r.route">{{ routeLabel(r.route)
                    }}</mdui-menu-item>
                </mdui-select>

                <mdui-text-field :value="timestamp" label="给药时间" type="datetime-local" variant="outlined"
                  @input="onTimestampInput"></mdui-text-field>

                <mdui-button variant="filled" full-width @click="saveDose" :disabled="!canSave">
                  确认记录
                </mdui-button>
              </div>
            </mdui-card>
            </template>
            <template v-else>
              <div class="panel-placeholder">
                <mdui-icon :name="panels.placeholderIcon"></mdui-icon>
                <p>{{ panels.placeholderText }}</p>
              </div>
            </template>
          </div>
        </div>

      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import * as store from '../wasm/engineStore'
import { useSlidingPanels } from '../composables/useSlidingPanels'
import { routeLabel, modelLabel, getCurrentTimestamp } from '../utils/format'
import DrugBrowser from '../components/DrugBrowser.vue'

const router = useRouter()
const panels = useSlidingPanels(2, { icon: 'edit_note', text: '在左侧选择药物后在此记录剂量' })

onMounted(async () => {
  allDrugs.value = await store.getAllDrugsWithSource()
})

const searchQuery = ref(''); const activeSource = ref('all')
const allDrugs = ref<any[]>([]); const selectedDrug = ref<any>(null)
const doseAmount = ref(''); const route = ref('oral'); const timestamp = ref('')
const displayedCount = ref(80)

const canSave = computed(() => {
  const val = parseFloat(doseAmount.value); return !isNaN(val) && val > 0
})

const availableRoutes = computed(() => {
  if (!selectedDrug.value) return [{ route: 'oral', unit: 'mg' }]
  return selectedDrug.value.routes || [{ route: 'oral', unit: selectedDrug.value.dose_unit || 'mg' }]
})

const currentDoseUnit = computed(() => {
  const matched = availableRoutes.value.find((r: any) => r.route === route.value)
  return matched ? matched.unit : (selectedDrug.value?.dose_unit || 'mg')
})

function onDoseInput(e: any) { doseAmount.value = e.target.value }
function onRouteChange(e: any) { route.value = e.target.value }
function onTimestampInput(e: any) { timestamp.value = e.target.value }

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

async function selectDrug(drug: any) {
  selectedDrug.value = drug
  timestamp.value = getCurrentTimestamp()
  doseAmount.value = ''
  const routes = drug.routes || [{ route: 'oral', unit: drug.dose_unit || 'mg' }]
  route.value = routes[0]?.route || 'oral'
  panels.markSelection()
  panels.advance(0)

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

function handleBack() { if (!panels.back()) router.push({ name: 'home' }) }
</script>
