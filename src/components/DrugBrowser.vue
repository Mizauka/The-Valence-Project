<template>
  <div class="drug-browser">
    <div class="search-box">
      <mdui-text-field
        :value="searchQuery"
        label="搜索药物"
        variant="outlined"
        icon="search"
        clearable
        placeholder="输入药物名称..."
        @input="e => $emit('update:searchQuery', e.target.value)"
        @clear="$emit('update:searchQuery', '')"
      ></mdui-text-field>
    </div>

    <mdui-tabs :value="activeSource" @change="e => $emit('update:activeSource', e.target.value)" class="source-tabs">
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
        @click="$emit('select', drug)"
      >
        <div class="drug-card-content">
          <div class="drug-card-main">
            <span class="drug-card-name">{{ drug.name }}</span>
            <span class="drug-card-model">{{ modelLabel(drug.model_type) }}</span>
          </div>
          <div class="drug-card-meta">
            <span v-if="drug.source === 'hrt'" class="source-tag source-tag-hrt">HRT</span>
            <span v-else-if="drug.source === 'journal'" class="source-tag source-tag-journal">Journal</span>
            <span v-else class="source-tag source-tag-custom">自定义</span>
            <span v-if="drug.parameters?.equivalence_factor" class="eq-tag">等效={{ drug.parameters.equivalence_factor }}</span>
            <span class="hl-tag">t½={{ drug.parameters?.half_life }}h</span>
          </div>
        </div>
      </mdui-card>
      <div v-if="hasMore" class="load-more" @click="$emit('loadMore')">
        加载更多 ({{ displayDrugs.length }}/{{ totalCount }})
      </div>
    </div>

    <div class="empty-state" v-else>
      <mdui-icon name="search_off"></mdui-icon>
      <p>{{ searchQuery ? '未找到匹配药物' : (totalCount === 0 ? '加载中...' : '该分类下暂无药物') }}</p>
    </div>

    <div v-if="showCustomFab" class="custom-fab-wrapper">
      <mdui-fab icon="add" @click="$emit('createCustom')"></mdui-fab>
    </div>
  </div>
</template>

<script setup lang="ts">
import { modelLabel } from '../utils/format'

defineProps<{
  searchQuery: string
  activeSource: string
  displayDrugs: any[]
  totalCount: number
  hasMore: boolean
  showCustomFab?: boolean
}>()

defineEmits<{
  'update:searchQuery': [value: string]
  'update:activeSource': [value: string]
  select: [drug: any]
  loadMore: []
  createCustom: []
}>()
</script>
