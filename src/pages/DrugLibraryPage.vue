<template>
  <div class="sliding-page" :data-step="panels.step">
    <div class="sliding-viewport">
      <mdui-button-icon icon="arrow_back" @click="handleBack" />

      <div class="sliding-track"
        :style="{ width: panels.trackWidthPercent + '%', transform: `translateX(-${panels.offsetPercent}%)` }">
        <div class="sliding-panel">
          <div class="sliding-panel-inner">
            <DrugBrowser :searchQuery="searchQuery" :activeSource="activeSource" :displayDrugs="displayDrugs"
              :totalCount="sourceFilteredDrugs.length" :hasMore="hasMore" :showCustomFab="activeSource === 'custom'"
              @update:searchQuery="q => { searchQuery = q; displayedCount = 80 }"
              @update:activeSource="s => { activeSource = s; displayedCount = 80 }" @select="onSelectDrug"
              @loadMore="loadMore" @createCustom="onCreateCustom" />
          </div>
        </div>
        <div class="sliding-panel">
          <div class="sliding-panel-inner">
            <template v-if="!panels.showPlaceholder">
            <mdui-card variant="" class="form-card" v-if="detailDrug">
              <div class="form-content">
                <div class="selected-drug-banner">
                  <mdui-icon name="medication" class="banner-icon"></mdui-icon>
                  <div class="banner-info">
                    <span class="banner-name">{{ detailDrug.name }}</span>
                    <span class="banner-detail">{{ modelLabel(detailDrug.model_type) }} · {{ detailDrug.drug_id
                      }}</span>
                  </div>
                </div>
                <mdui-button-icon v-if="detailDrug?.source === 'custom'" icon="edit" style="margin-left:auto;flex-shrink:0" @click="enableEdit"></mdui-button-icon>
              </div>
              <template v-if="!editingDrug">
                <mdui-list><mdui-list-item v-for="(v, k) in drugParams" :key="k"><span class="param-key">{{ k }}</span><span slot="description" class="param-val">{{ v }}</span></mdui-list-item></mdui-list>
              </template>
              <template v-else>
                <mdui-text-field :value="editName" label="药物名称" variant="outlined" @input="(e: any) => editName = e.target.value" />
                <mdui-text-field :value="editHL" label="半衰期 (h)" type="number" variant="outlined" @input="(e: any) => editHL = e.target.value" />
                <mdui-text-field :value="editVD" label="分布容积 (L/kg)" type="number" variant="outlined" @input="(e: any) => editVD = e.target.value" />
                <mdui-text-field :value="editKa" label="吸收速率 ka (1/h)" type="number" variant="outlined" @input="(e: any) => editKa = e.target.value" />
                <mdui-text-field :value="editF" label="生物利用度 F" type="number" variant="outlined" @input="(e: any) => editF = e.target.value" />
                <div style="display:flex;gap:8px">
                  <mdui-button variant="text" @click="cancelEdit">取消</mdui-button>
                  <mdui-button variant="filled" :disabled="!editCanSave" @click="saveDrugEdit">保存</mdui-button>
                </div>
              </template>
            </mdui-card>
            <mdui-card variant="" class="form-card" v-if="showCustomForm">
              <div class="form-content">
                <mdui-text-field :value="customName" label="药物名称" variant="outlined"
                  @input="(e: any) => customName = e.target.value" />
                <mdui-select :value="customModel" label="房室模型" variant="outlined"
                  @change="(e: any) => customModel = e.target.value"><mdui-menu-item
                    value="one_compartment">一室模型</mdui-menu-item><mdui-menu-item
                    value="two_compartment">二室模型</mdui-menu-item></mdui-select>
                <mdui-text-field :value="customHL" label="半衰期 (h)" type="number" variant="outlined"
                  @input="(e: any) => customHL = e.target.value" />
                <mdui-text-field :value="customVD" label="分布容积 (L/kg)" type="number" variant="outlined"
                  @input="(e: any) => customVD = e.target.value" />
                <mdui-text-field :value="customKa" label="吸收速率 ka (1/h)" type="number" variant="outlined"
                  @input="(e: any) => customKa = e.target.value" />
                <mdui-text-field :value="customF" label="生物利用度 F" type="number" variant="outlined"
                  @input="(e: any) => customF = e.target.value" />
                <mdui-button variant="filled" full-width :disabled="!canCreateCustom"
                  @click="saveCustomDrug">创建药物</mdui-button>
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
import { modelLabel } from '../utils/format'
import type { Drug } from '../types/drug'
import DrugBrowser from '../components/DrugBrowser.vue'

const router = useRouter()
const panels = useSlidingPanels(2, { icon: 'medication', text: '在左侧选择药物后查看详情' })

function handleBack() { if (!panels.back()) router.push({ name: 'home' }) }

const searchQuery = ref(''); const activeSource = ref('all')
const allDrugs = ref<Drug[]>([]); const displayedCount = ref(80)
const detailDrug = ref<Drug | null>(null); const showCustomForm = ref(false)
const editingDrug = ref(false); const editName = ref(''); const editHL = ref(''); const editVD = ref(''); const editKa = ref(''); const editF = ref('')
const customName = ref(''); const customModel = ref('one_compartment')
const customHL = ref(''); const customVD = ref('2'); const customKa = ref('0.32'); const customF = ref('1')

onMounted(async () => {
  allDrugs.value = await store.getAllDrugsWithSource()
})

const editCanSave = computed(() => editName.value.trim() && parseFloat(editHL.value) > 0)

const canCreateCustom = computed(() => customName.value.trim() && parseFloat(customHL.value) > 0)
const sourceFilteredDrugs = computed(() => {
  if (activeSource.value === 'all') return allDrugs.value
  return allDrugs.value.filter((d: Drug) => d.source === activeSource.value)
})
const drugParams = computed(() => {
  if (!detailDrug.value?.parameters) return {}
  const p: Record<string, any> = detailDrug.value.parameters as any; const e: Record<string, string> = {}
  if (p.half_life > 0) e['半衰期'] = p.half_life + ' h'
  if (p.volume_of_distribution) e['分布容积'] = p.volume_of_distribution + ' L/kg'
  if (p.ka) e['吸收速率 ka'] = String(p.ka)
  if (p.bioavailability) e['生物利用度 F'] = String(p.bioavailability)
  if (p.equivalence_factor) e['等效因子'] = String(p.equivalence_factor)
  if (p.k_clear) e['清除速率'] = String(p.k_clear)
  return e
})
const filteredDrugs = computed(() => { const q = searchQuery.value.trim().toLowerCase(); return q ? sourceFilteredDrugs.value.filter((d: Drug) => d.name.toLowerCase().includes(q) || d.drug_id.toLowerCase().includes(q)) : sourceFilteredDrugs.value })
const displayDrugs = computed(() => filteredDrugs.value.slice(0, displayedCount.value))
const hasMore = computed(() => displayedCount.value < filteredDrugs.value.length)
function loadMore() { displayedCount.value = Math.min(displayedCount.value + 80, filteredDrugs.value.length) }

function onSelectDrug(drug: Drug) { detailDrug.value = drug; showCustomForm.value = false; panels.markSelection(); panels.advance(0) }
function onCreateCustom() { showCustomForm.value = true; detailDrug.value = null; panels.markSelection(); panels.advance(0) }
async function saveCustomDrug() {
  const hl = parseFloat(customHL.value); if (!hl || hl <= 0) return
  await store.addDrug({ drug_id: 'custom_' + customName.value.toLowerCase().replace(/\s+/g, '_'), name: customName.value, model_type: customModel.value, dose_unit: 'mg', routes: [{ route: 'oral', unit: 'mg' }], parameters: { half_life: hl, volume_of_distribution: parseFloat(customVD.value) || 2, ka: parseFloat(customKa.value) || 0.32, bioavailability: parseFloat(customF.value) || 1 } })
  allDrugs.value = await store.getAllDrugsWithSource()
  customName.value = ''; customHL.value = ''; showCustomForm.value = false; panels.reset()
}

function enableEdit() {
  if (!detailDrug.value) return
  editingDrug.value = true
  const p = detailDrug.value.parameters || {}
  editName.value = detailDrug.value.name || ''
  editHL.value = String(p.half_life || '')
  editVD.value = String(p.volume_of_distribution || p.Vd || '2')
  editKa.value = String(p.ka || '0.32')
  editF.value = String(p.bioavailability || p.F || '1')
}
function cancelEdit() { editingDrug.value = false }
async function saveDrugEdit() {
  if (!detailDrug.value || !editCanSave.value) return
  const drugId = detailDrug.value.drug_id
  await store.deleteDrug(drugId)
  await store.addDrug({ drug_id: drugId, name: editName.value, model_type: detailDrug.value.model_type, dose_unit: 'mg', routes: [{ route: 'oral', unit: 'mg' }], parameters: { half_life: parseFloat(editHL.value), volume_of_distribution: parseFloat(editVD.value) || 2, ka: parseFloat(editKa.value) || 0.32, bioavailability: parseFloat(editF.value) || 1 } })
  allDrugs.value = await store.getAllDrugsWithSource()
  detailDrug.value = allDrugs.value.find((d: Drug) => d.drug_id === drugId) || null
  editingDrug.value = false
}
</script>
