<template>
  <div class="page drug-library-page">
    <h2 class="page-title">药物库</h2>

    <div class="source-tabs">
      <mdui-chip elevated :selected="activeSource === 'hrt'" @click="loadSource('hrt')">HRT 药物</mdui-chip>
      <mdui-chip elevated :selected="activeSource === 'journal'" @click="loadSource('journal')">Journal 药物</mdui-chip>
      <mdui-chip elevated :selected="activeSource === 'custom'" @click="loadSource('custom')">自定义</mdui-chip>
    </div>

    <div class="drug-list" v-if="displayDrugs.length > 0">
      <mdui-card
        v-for="drug in displayDrugs"
        :key="drug.drug_id"
        variant="outlined"
        class="drug-item"
      >
        <div class="drug-item-content">
          <div class="drug-main">
            <span class="drug-name">{{ drug.name }}</span>
            <span class="drug-model">{{ modelLabel(drug.model_type) }}</span>
          </div>
          <div class="drug-params">
            <span>t½={{ drug.parameters.half_life }}h</span>
            <span>Vd={{ drug.parameters.volume_of_distribution }} L/kg</span>
            <span>CL={{ drug.parameters.k_clear || drug.parameters.clearance }} L/h/kg</span>
            <span v-if="drug.parameters.equivalence_factor" class="eq-factor">
              等效系数={{ drug.parameters.equivalence_factor }}
            </span>
            <span v-if="drug.parameters.group_id" class="group-tag">
              {{ drug.parameters.group_id }}
            </span>
          </div>
          <div class="drug-actions">
            <mdui-button-icon
              icon="add_circle"
              @click="goToDose(drug)"
            ></mdui-button-icon>
            <mdui-button-icon
              v-if="activeSource === 'custom'"
              icon="delete"
              @click="deleteDrug(drug.drug_id)"
            ></mdui-button-icon>
          </div>
        </div>
      </mdui-card>
    </div>

    <div class="empty-hint" v-else>
      <mdui-icon name="medication"></mdui-icon>
      <p>{{ activeSource === 'custom' ? '暂无自定义药物，点击右下角添加' : '加载中...' }}</p>
    </div>

    <mdui-card variant="filled" class="form-card" v-if="showForm && activeSource === 'custom'">
      <div class="form-content">
        <h3 class="form-title">添加药物</h3>
        <mdui-text-field
          v-model="drug.name"
          label="药物名称"
          variant="outlined"
        ></mdui-text-field>

        <mdui-select
          v-model="drug.model_type"
          label="房室模型"
          variant="outlined"
        >
          <mdui-menu-item value="one_compartment">一室模型</mdui-menu-item>
          <mdui-menu-item value="two_compartment">二室模型</mdui-menu-item>
          <mdui-menu-item value="multi_compartment">多室模型</mdui-menu-item>
        </mdui-select>

        <mdui-text-field
          v-model.number="drug.half_life"
          label="半衰期 (h)"
          type="number"
          variant="outlined"
        ></mdui-text-field>

        <mdui-text-field
          v-model.number="drug.volume_of_distribution"
          label="分布容积 (L/kg)"
          type="number"
          variant="outlined"
        ></mdui-text-field>

        <mdui-text-field
          v-model.number="drug.clearance"
          label="清除率 (L/h/kg)"
          type="number"
          variant="outlined"
        ></mdui-text-field>

        <mdui-text-field
          v-model.number="drug.ka"
          label="吸收速率常数 Ka (1/h)"
          type="number"
          variant="outlined"
        ></mdui-text-field>

        <mdui-text-field
          v-model.number="drug.bioavailability"
          label="生物利用度 F"
          type="number"
          variant="outlined"
        ></mdui-text-field>

        <div class="form-actions">
          <mdui-button variant="tonal" @click="showForm = false">取消</mdui-button>
          <mdui-button variant="filled" @click="saveDrug">保存药物</mdui-button>
        </div>
      </div>
    </mdui-card>

    <mdui-fab icon="add" class="fab" @click="showForm = true" v-if="!showForm && activeSource === 'custom'"></mdui-fab>
  </div>
</template>

<script setup>
import { reactive, ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import * as store from '../wasm/engineStore'

const router = useRouter()

const customDrugs = ref([])
const presetDrugs = ref([])
const activeSource = ref('hrt')
let showForm = ref(false)

const displayDrugs = computed(() => {
  if (activeSource.value === 'custom') return customDrugs.value
  return presetDrugs.value
})

const drug = reactive({
  name: '',
  model_type: 'one_compartment',
  half_life: 0,
  volume_of_distribution: 0,
  clearance: 0,
  ka: 0,
  bioavailability: 0,
})

onMounted(async () => {
  customDrugs.value = await store.getCustomDrugs()
  await loadSource('hrt')
})

async function loadSource(source) {
  activeSource.value = source
  showForm.value = false
  if (source === 'custom') {
    customDrugs.value = await store.getCustomDrugs()
    return
  }

  presetDrugs.value = await store.getPresetDrugs(source)
}

async function goToDose(drug) {
  await store.addDrug({
    drug_id: drug.drug_id,
    name: drug.name,
    model_type: drug.model_type,
    parameters: drug.parameters,
  })
  router.push({ name: 'add-dose' })
}

async function saveDrug() {
  await store.addDrug({
    drug_id: crypto.randomUUID(),
    name: drug.name,
    model_type: drug.model_type,
    parameters: {
      half_life: drug.half_life,
      volume_of_distribution: drug.volume_of_distribution,
      clearance: drug.clearance,
      ka: drug.ka,
      bioavailability: drug.bioavailability,
    },
  })
  customDrugs.value = await store.getCustomDrugs()
  showForm = false
  drug.name = ''
  drug.half_life = 0
  drug.volume_of_distribution = 0
  drug.clearance = 0
  drug.ka = 0
  drug.bioavailability = 0
}

async function deleteDrug(drugId) {
  await store.deleteDrug(drugId)
  customDrugs.value = await store.getCustomDrugs()
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
.drug-library-page {
  margin: 0 auto;
  padding-bottom: 80px;
}

.page-title {
  font-size: 24px;
  font-weight: 600;
  margin-bottom: 16px;
}

.source-tabs {
  display: flex;
  gap: 8px;
  margin-bottom: 16px;
  flex-wrap: wrap;
}

.drug-item {
  padding: 12px 16px;
  margin-bottom: 8px;
}

.drug-item-content {
  display: flex;
  align-items: center;
  gap: 12px;
}

.drug-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}

.drug-name {
  font-weight: 600;
  font-size: 16px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.drug-model {
  font-size: 12px;
  opacity: 0.6;
}

.drug-params {
  display: flex;
  gap: 8px;
  font-size: 12px;
  opacity: 0.7;
  flex-wrap: wrap;
}

.eq-factor {
  color: var(--mdui-color-primary);
  opacity: 1 !important;
  font-weight: 600;
}

.group-tag {
  color: var(--mdui-color-secondary);
  opacity: 1 !important;
}

.drug-actions {
  flex-shrink: 0;
  display: flex;
  gap: 4px;
}

.empty-hint {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 48px 0;
  opacity: 0.5;
}

.form-card {
  padding: 16px;
  margin-top: 16px;
}

.form-title {
  font-size: 18px;
  font-weight: 600;
  margin-bottom: 12px;
}

.form-content {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.fab {
  position: fixed;
  bottom: 90px;
  right: 24px;
  z-index: 50;
}

@media (min-width: 768px) {
  .fab {
    bottom: 24px;
  }
}
</style>
