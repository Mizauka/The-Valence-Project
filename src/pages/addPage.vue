<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { getAllDrugsWithSource, addDose } from '../services/engineStore'
import { useDoseHistory } from '../composables/useDoseHistory'
import type { DrugRecord } from '../services/types'

const router = useRouter()
const { load: refreshHistory } = useDoseHistory()

// ─── State ────────────────────────────────────────────────────────
const searchQuery = ref('')
const activeSource = ref<'all' | 'hrt' | 'journal' | 'custom'>('all')
const allDrugs = ref<DrugRecord[]>([])
const selectedDrug = ref<DrugRecord | null>(null)
const doseAmount = ref('')
const route = ref('oral')
const timestamp = ref(new Date().toISOString().slice(0, 16)) // "YYYY-MM-DDTHH:mm"
const saving = ref(false)
const displayedCount = ref(80)

// ─── Computed ─────────────────────────────────────────────────────

const canSave = computed(() => {
  const v = parseFloat(doseAmount.value)
  return !isNaN(v) && v > 0 && selectedDrug.value !== null
})

const availableRoutes = computed(() => {
  if (!selectedDrug.value) return [{ route: 'oral', unit: 'mg' }]
  return selectedDrug.value.routes || [{ route: 'oral', unit: selectedDrug.value.dose_unit || 'mg' }]
})

const currentDoseUnit = computed(() => {
  const matched = availableRoutes.value.find(r => r.route === route.value)
  return matched?.unit || selectedDrug.value?.dose_unit || 'mg'
})

const filteredDrugs = computed(() => {
  const q = searchQuery.value.toLowerCase().trim()
  let list = allDrugs.value
  if (activeSource.value !== 'all') {
    list = list.filter(d => d.source === activeSource.value)
  }
  if (!q) return list
  return list.filter(d => d.name.toLowerCase().includes(q) || d.drug_id.toLowerCase().includes(q))
})

const displayDrugs = computed(() => filteredDrugs.value.slice(0, displayedCount.value))
const hasMore = computed(() => displayedCount.value < filteredDrugs.value.length)

// ─── Lifecycle ────────────────────────────────────────────────────

onMounted(async () => {
  allDrugs.value = await getAllDrugsWithSource()
})

// ─── Methods ──────────────────────────────────────────────────────

function selectDrug(drug: DrugRecord) {
  selectedDrug.value = drug
  doseAmount.value = ''
  const routes = drug.routes || [{ route: 'oral', unit: drug.dose_unit || 'mg' }]
  route.value = routes[0]?.route || 'oral'
}

function loadMore() {
  displayedCount.value = Math.min(displayedCount.value + 80, filteredDrugs.value.length)
}

async function save() {
  if (!canSave.value || !selectedDrug.value) return
  saving.value = true
  try {
    const amount = parseFloat(doseAmount.value)
    const unit = currentDoseUnit.value
    let amountMG = amount
    if (unit === 'µg') amountMG = amount / 1000
    else if (unit === 'ng') amountMG = amount / 1_000_000
    else if (unit === 'pg') amountMG = amount / 1_000_000_000

    const timeH = new Date(timestamp.value).getTime() / 3600_000

    await addDose({
      dose_id: crypto.randomUUID(),
      drug_id: selectedDrug.value.drug_id,
      dose_amount: amountMG,
      timestamp: timeH,
      route_of_administration: route.value,
    })

    // Refresh shared dose history before navigating
    await refreshHistory()

    // Navigate back
    if (router.options.history.state?.back) {
      router.back()
    } else {
      router.push({ name: 'home' })
    }
  } catch (e) {
    console.error('Failed to save dose:', e)
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <div style="padding: 16px; max-width: 480px; margin: 0 auto">
    <!-- Drug Search -->
    <m3e-search-bar clearable style="margin-bottom: 12px;">
      <m3e-icon name="search" slot="leading"></m3e-icon>
      <input
        slot="input"
        placeholder="搜索药物..."
        :value="searchQuery"
        @input="searchQuery = ($event.target as HTMLInputElement).value; displayedCount = 80"
      />
    </m3e-search-bar>

    <!-- Source Tabs -->
    <m3e-segmented-button style="margin-bottom: 12px;">
      <m3e-button-segment :checked="activeSource === 'all'" @click="activeSource = 'all'; displayedCount = 80">全部</m3e-button-segment>
      <m3e-button-segment :checked="activeSource === 'hrt'" @click="activeSource = 'hrt'; displayedCount = 80">HRT</m3e-button-segment>
      <m3e-button-segment :checked="activeSource === 'journal'" @click="activeSource = 'journal'; displayedCount = 80">Journal</m3e-button-segment>
      <m3e-button-segment :checked="activeSource === 'custom'" @click="activeSource = 'custom'; displayedCount = 80">自定义</m3e-button-segment>
    </m3e-segmented-button>

    <!-- Drug List -->
    <mdui-list style="max-height: 200px; overflow: auto; margin-bottom: 12px;">
      <mdui-list-item
        v-for="drug in displayDrugs"
        :key="drug.drug_id"
        :icon="selectedDrug?.drug_id === drug.drug_id ? 'check' : 'medication'"
        :active="selectedDrug?.drug_id === drug.drug_id"
        rounded
        @click="selectDrug(drug)"
      >
        {{ drug.name }}
        <span slot="description">{{ drug.dose_unit || 'mg' }}</span>
      </mdui-list-item>
      <mdui-list-item v-if="hasMore" @click="loadMore" style="text-align:center;color:rgb(var(--mdui-color-primary))">
        加载更多...
      </mdui-list-item>
    </mdui-list>

    <!-- Dose Form -->
    <div v-if="selectedDrug" style="display: flex; flex-direction: column; gap: 12px;">
      <mdui-card variant="filled" style="padding: 12px;">
        <div style="font-weight: 500; margin-bottom: 8px;">{{ selectedDrug.name }}</div>

        <!-- Dose Amount -->
        <m3e-form-field style="margin-bottom: 12px;">
          <label slot="label">剂量</label>
          <input
            type="number"
            step="any"
            min="0"
            placeholder="输入剂量"
            :value="doseAmount"
            @input="doseAmount = ($event.target as HTMLInputElement).value"
            style="width: 100%;"
          />
          <span slot="supporting-text">单位: {{ currentDoseUnit }}</span>
        </m3e-form-field>

        <!-- Route -->
        <m3e-form-field style="margin-bottom: 12px;">
          <label slot="label">给药途径</label>
          <select
            :value="route"
            @change="route = ($event.target as HTMLSelectElement).value"
            style="width: 100%; padding: 8px; border-radius: var(--mdui-shape-corner-small); border: 1px solid rgb(var(--mdui-color-outline)); background: rgb(var(--mdui-color-surface)); color: rgb(var(--mdui-color-on-surface));"
          >
            <option v-for="r in availableRoutes" :key="r.route" :value="r.route">
              {{ r.route }} ({{ r.unit }})
            </option>
          </select>
        </m3e-form-field>

        <!-- Timestamp -->
        <m3e-form-field style="margin-bottom: 12px;">
          <label slot="label">用药时间</label>
          <input
            type="datetime-local"
            :value="timestamp"
            @input="timestamp = ($event.target as HTMLInputElement).value"
            style="width: 100%;"
          />
        </m3e-form-field>
      </mdui-card>

      <!-- Save Button -->
      <m3e-button
        variant="filled"
        :disabled="!canSave || saving"
        @click="save"
        style="width: 100%;"
      >
        <m3e-icon slot="icon" :name="saving ? 'hourglass' : 'save'"></m3e-icon>
        {{ saving ? '保存中...' : '保存记录' }}
      </m3e-button>
    </div>

    <!-- No drug selected hint -->
    <div
      v-else
      style="text-align: center; padding: 32px; color: rgb(var(--mdui-color-on-surface-variant));"
    >
      <p style="font-size: 0.875rem;">请先选择一种药物</p>
    </div>
  </div>
</template>
