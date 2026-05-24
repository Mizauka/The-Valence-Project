<script setup lang="ts">
import { onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { vLit } from '../composables/useLitProps'
import { useBreakpoint } from '../composables/useBreakpoint'
import { usePlan } from '../composables/usePlan'
import { useDoseHistory } from '../composables/useDoseHistory'
import { addDose } from '../services/engineStore'
import planEditor from './planEditor.vue'

const { isPhone } = useBreakpoint()
const {
  groupedPlans, selectedPlanId,
  selectPlan, startEdit, deletePlan,
  getDoseRecordsFromPlan, plans,
} = usePlan()
const { load: refreshHistory } = useDoseHistory()

defineExpose({ reload: () => {} })

// Apply a plan: add all its doses as actual records
async function applyPlan(planId: string) {
  const plan = plans.value.find(p => p.id === planId)
  if (!plan) return
  const records = getDoseRecordsFromPlan(plan)
  for (const rec of records) {
    await addDose({
      dose_id: crypto.randomUUID(),
      drug_id: rec.drug_id,
      dose_amount: rec.dose_amount,
      timestamp: rec.timestamp,
      route_of_administration: rec.route,
    })
  }
  await refreshHistory()
  const router = useRouter()
  router.push({ name: 'home' })
}
</script>

<template>
  <div v-if="!groupedPlans.length" style="padding:32px;text-align:center;color:rgb(var(--mdui-color-on-surface-variant))">
    <p style="font-size:0.875rem">暂无用药方案</p>
    <p style="font-size:0.75rem;margin-top:4px">点击 + 创建用药方案模板</p>
  </div>

  <mdui-list v-else>
    <template v-for="g in groupedPlans" :key="g.date">
      <mdui-list-subheader>{{ g.date }}</mdui-list-subheader>
      <mdui-list-item
        v-for="plan in g.plans"
        :key="plan.id"
        icon="clinical_notes"
        :description="plan.description || `${plan.doses.length} 种药物`"
        end-icon="edit"
        :active="selectedPlanId === plan.id"
        rounded
        @click="selectPlan(plan.id)"
      >
        <m3e-bottom-sheet-trigger for="planeditor" v-if="isPhone"
          @click="startEdit(plan)"
        >
          {{ plan.name || '未命名方案' }}
        </m3e-bottom-sheet-trigger>
        <template v-if="!isPhone">
          <span @click="startEdit(plan)" style="cursor:pointer;">{{ plan.name || '未命名方案' }}</span>
        </template>
        <mdui-button-icon
          slot="end-icon"
          icon="play_arrow"
          @click.stop="applyPlan(plan.id)"
          title="应用此方案"
        ></mdui-button-icon>
      </mdui-list-item>
    </template>
  </mdui-list>

  <m3e-bottom-sheet
    v-lit="{
      handle: true,
      detents: ['fit', 'full', 'full'],
      detent: 1,
      hideable: true,
    }"
    id="planeditor"
    modal
  >
    <planEditor />
  </m3e-bottom-sheet>
</template>
