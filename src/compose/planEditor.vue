<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { usePlan } from '../composables/usePlan'
import type { PlanDose } from '../composables/usePlan'
import { getAllDrugsWithSource } from '../services/engineStore'
import type { DrugRecord } from '../services/types'

const {
  editingPlan, selectedPlan,
  cancelEdit, savePlan,
  addDoseToPlan, removeDoseFromPlan,
} = usePlan()

const allDrugs = ref<DrugRecord[]>([])
const drugSearch = ref('')
const selectedDrugForDose = ref<DrugRecord | null>(null)
const newDoseAmount = ref('')
const newDoseUnit = ref('mg')
const newDoseRoute = ref('oral')
const newDoseInterval = ref('24')
const newDoseDuration = ref('7')

onMounted(async () => {
  allDrugs.value = await getAllDrugsWithSource()
})

const filteredDrugs = computed(() => {
  const q = drugSearch.value.toLowerCase().trim()
  if (!q) return allDrugs.value
  return allDrugs.value.filter(d => d.name.toLowerCase().includes(q))
})

function selectDrugForDose(drug: DrugRecord) {
  selectedDrugForDose.value = drug
  newDoseUnit.value = drug.dose_unit || 'mg'
  const routes = drug.routes || [{ route: 'oral', unit: drug.dose_unit || 'mg' }]
  newDoseRoute.value = routes[0]?.route || 'oral'
}

function addDose() {
  const amount = parseFloat(newDoseAmount.value)
  if (!selectedDrugForDose.value || isNaN(amount) || amount <= 0) return
  const dose: PlanDose = {
    drug_id: selectedDrugForDose.value.drug_id,
    drugName: selectedDrugForDose.value.name,
    dose_amount: amount,
    dose_unit: newDoseUnit.value,
    route: newDoseRoute.value,
    interval_h: parseFloat(newDoseInterval.value) || 24,
    duration_d: parseFloat(newDoseDuration.value) || 7,
  }
  addDoseToPlan(dose)
  newDoseAmount.value = ''
  selectedDrugForDose.value = null
  drugSearch.value = ''
}

async function handleSave() {
  if (!editingPlan.value) return
  if (!editingPlan.value.name.trim()) {
    editingPlan.value.name = '未命名方案'
  }
  await savePlan()
}
</script>

<template>
  <div style="padding: 16px; max-height: 80vh; overflow: auto;">
    <div v-if="editingPlan" style="display:flex;flex-direction:column;gap:12px;">
      <m3e-heading variant="title" size="medium">
        {{ selectedPlan ? '编辑方案' : '新建方案' }}
      </m3e-heading>
      <m3e-form-field>
        <label slot="label">方案名称</label>
        <input type="text" placeholder="例如：门诊常规方案"
          :value="editingPlan.name"
          @input="editingPlan.name = ($event.target as HTMLInputElement).value" />
      </m3e-form-field>
      <m3e-form-field>
        <label slot="label">描述 (可选)</label>
        <input type="text" placeholder="例如：雌二醇 每天4mg"
          :value="editingPlan.description"
          @input="editingPlan.description = ($event.target as HTMLInputElement).value" />
      </m3e-form-field>

      <!-- Current doses -->
      <div v-if="editingPlan.doses.length" style="margin-top:8px;">
        <div style="font-weight:500;margin-bottom:8px;">已添加药物 ({{ editingPlan.doses.length }})</div>
        <m3e-card v-for="(dose, idx) in editingPlan.doses" :key="idx" variant="filled" style="margin-bottom:8px;padding:8px;">
          <div style="display:flex;justify-content:space-between;align-items:center;">
            <div>
              <div style="font-weight:500;">{{ dose.drugName }}</div>
              <div style="font-size:0.75rem;color:rgb(var(--mdui-color-on-surface-variant))">
                {{ dose.dose_amount }}{{ dose.dose_unit }} &middot; {{ dose.route }} &middot; 每{{ dose.interval_h }}h &middot; {{ dose.duration_d }}天
              </div>
            </div>
            <mdui-button-icon icon="delete" @click="removeDoseFromPlan(idx)"></mdui-button-icon>
          </div>
        </m3e-card>
      </div>

      <!-- Add drug -->
      <div style="border-top:1px solid rgb(var(--mdui-color-outline-variant));padding-top:12px;">
        <div style="font-weight:500;margin-bottom:8px;">添加药物</div>
        <m3e-search-bar clearable style="margin-bottom:8px;">
          <m3e-icon name="search" slot="leading"></m3e-icon>
          <input slot="input" placeholder="搜索药物..."
            :value="drugSearch"
            @input="drugSearch = ($event.target as HTMLInputElement).value" />
        </m3e-search-bar>
        <div v-if="drugSearch && filteredDrugs.length" style="max-height:150px;overflow:auto;margin-bottom:8px;">
          <mdui-list-item v-for="drug in filteredDrugs.slice(0, 20)" :key="drug.drug_id"
            icon="medication" :active="selectedDrugForDose?.drug_id === drug.drug_id" rounded
            @click="selectDrugForDose(drug)">
            {{ drug.name }}
          </mdui-list-item>
        </div>
        <div v-if="selectedDrugForDose" style="display:flex;flex-direction:column;gap:8px;">
          <div style="display:grid;grid-template-columns:1fr 1fr;gap:8px;">
            <m3e-form-field>
              <label slot="label">剂量 ({{ newDoseUnit }})</label>
              <input type="number" step="any" min="0" placeholder="剂量"
                :value="newDoseAmount"
                @input="newDoseAmount = ($event.target as HTMLInputElement).value" />
            </m3e-form-field>
            <m3e-form-field>
              <label slot="label">途径</label>
              <select :value="newDoseRoute"
                @change="newDoseRoute = ($event.target as HTMLSelectElement).value"
                style="width:100%;padding:8px;border-radius:var(--mdui-shape-corner-small);border:1px solid rgb(var(--mdui-color-outline));background:rgb(var(--mdui-color-surface));color:rgb(var(--mdui-color-on-surface))">
                <option v-for="r in (selectedDrugForDose.routes || [{route:'oral',unit:'mg'}])" :key="r.route" :value="r.route">
                  {{ r.route }}
                </option>
              </select>
            </m3e-form-field>
          </div>
          <div style="display:grid;grid-template-columns:1fr 1fr;gap:8px;">
            <m3e-form-field>
              <label slot="label">间隔 (小时)</label>
              <input type="number" step="any" min="0.5"
                :value="newDoseInterval"
                @input="newDoseInterval = ($event.target as HTMLInputElement).value" />
            </m3e-form-field>
            <m3e-form-field>
              <label slot="label">持续 (天)</label>
              <input type="number" step="any" min="1"
                :value="newDoseDuration"
                @input="newDoseDuration = ($event.target as HTMLInputElement).value" />
            </m3e-form-field>
          </div>
          <m3e-button variant="filled" @click="addDose" :disabled="!newDoseAmount">
            <m3e-icon slot="icon" name="add"></m3e-icon>添加此药物
          </m3e-button>
        </div>
      </div>

      <div style="display:flex;gap:8px;margin-top:16px;">
        <m3e-button variant="outlined" @click="cancelEdit" style="flex:1;">取消</m3e-button>
        <m3e-button variant="filled" @click="handleSave" style="flex:1;">保存方案</m3e-button>
      </div>
    </div>

    <div v-else style="text-align:center;padding:32px;color:rgb(var(--mdui-color-on-surface-variant))">
      <p>选择方案进行编辑，或创建新方案</p>
    </div>
  </div>
</template>
