<script setup lang="ts">
import { onMounted } from 'vue'
import { useDrugLibrary } from '../composables/useDrugLibrary'

const { categories, loading, load, searchQuery, setSearch, selectDrug, selectedDrugId } = useDrugLibrary()

onMounted(() => {
  load()
})

defineExpose({ load })
</script>

<template>
  <div style="padding: 8px">
    <m3e-search-bar clearable style="margin-bottom: 8px;">
      <m3e-icon name="search" slot="leading"></m3e-icon>
      <input
        slot="input"
        placeholder="搜索药物..."
        :value="searchQuery"
        @input="setSearch(($event.target as HTMLInputElement).value)"
      />
    </m3e-search-bar>

    <div v-if="loading" style="display:flex;align-items:center;justify-content:center;padding:32px">
      <mdui-circular-progress indeterminate></mdui-circular-progress>
    </div>

    <m3e-tree v-else>
      <m3e-tree-item
        v-for="cat in categories"
        :key="cat.source"
        open
      >
        <span slot="label">{{ cat.label }} ({{ cat.drugs.length }})</span>
        <m3e-tree-item
          v-for="drug in cat.drugs"
          :key="drug.drug_id"
          :selected="selectedDrugId === drug.drug_id"
          @click="selectDrug(drug.drug_id)"
        >
          <span slot="label">{{ drug.name }}</span>
        </m3e-tree-item>
      </m3e-tree-item>
    </m3e-tree>
  </div>
</template>
