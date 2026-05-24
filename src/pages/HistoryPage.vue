<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { vLit } from "../composables/useLitProps";
import { useSplitPane } from "../composables/useSplitPane";
import { useCalibrationForm } from "../composables/useCalibrationForm";
import history from "../compose/history.vue";
import calibration from "../compose/calibration.vue";
import calibrationSettings from "../compose/calibrationSettings.vue";
import addPage from "./addPage.vue";
import planPage from "./planPage.vue";

const { width, paneRef, minLeftPaneWidth, minRightPaneWidth, splitProps } =
  useSplitPane({
    rightFraction: 0.25,
    rightMin: 170,
    initValue: 50,
  });

// 跟踪 bottom-app-bar 的 hide 属性：有 hide → 隐藏，无 → 显示
const barVisible = ref(true);

// Calibration form
const calForm = useCalibrationForm();

onMounted(() => {
  const bar = document.querySelector("mdui-navigation-bar");
  if (!bar) return;
  // 初始状态
  barVisible.value = !bar.hasAttribute("hide");
  // 监听 hide 属性变化
  const observer = new MutationObserver(() => {
    barVisible.value = !bar.hasAttribute("hide");
  });
  observer.observe(bar, { attributes: true, attributeFilter: ["hide"] });
  onUnmounted(() => observer.disconnect());
});
</script>

<template>
  <m3e-split-pane
    ref="paneRef"
    v-lit="splitProps"
    style="height: 100%"
    v-if="820 < width"
  >
    <div
      slot="start"
      style="position: relative; height: 100%; overflow-x: hidden; min-width: 0"
    >
      <div
        :style="{
          position: 'absolute',
          top: '0',
          left: '0',
          right: '0',
          bottom: '0',
          minWidth: minLeftPaneWidth,
        }"
      >
        <div style="padding: 8px">
          <m3e-search-bar clearable>
            <m3e-icon name="search" slot="leading"></m3e-icon>
            <input slot="input" placeholder="Search..." /> </m3e-search-bar
          ><history />
        </div>
      </div>
    </div>
    <div
      slot="end"
      style="position: relative; height: 100%; overflow-x: hidden; min-width: 0"
    >
      <div
        :style="{
          position: 'absolute',
          top: '0',
          left: '0',
          right: '0',
          bottom: '0',
          minWidth: minRightPaneWidth,
        }"
      >
        <div style="padding: 8px">
          <m3e-search-bar clearable>
            <m3e-icon name="search" slot="leading"></m3e-icon>
            <input slot="input" placeholder="Search..." />
          </m3e-search-bar>
          <calibrationSettings />
          <calibration />
        </div>
      </div>
      <m3e-fab
        variant="primary"
        size="medium"
        style="position: absolute; right: 16px; bottom: 96px"
      >
        <m3e-bottom-sheet-trigger for="calibration-sheet">
          <m3e-icon name="add"></m3e-icon>
        </m3e-bottom-sheet-trigger>
      </m3e-fab>
    </div>
  </m3e-split-pane>
  <div v-else style="width: 100%; overflow: auto; position: relative">
    <mdui-tabs value="history-panel" placement="top">
      <mdui-tab value="history-panel">用药记录</mdui-tab>
      <mdui-tab value="calibration-panel">曲线校准</mdui-tab>
      <mdui-tab value="plan-panel">用药方案</mdui-tab>

      <!-- 用药记录 -->
      <mdui-tab-panel slot="panel" value="history-panel" style="padding: 8px">
        <div>
          <m3e-search-bar clearable style="margin-top: 8px">
            <m3e-icon name="search" slot="leading"></m3e-icon>
            <input slot="input" placeholder="Search..." />
          </m3e-search-bar>
          <history />
          <m3e-fab
            variant="primary"
            size="medium"
            :style="{
              position: 'fixed',
              right: '16px',
              bottom: barVisible ? '96px' : '16px',
              transition: 'bottom 0.3s',
            }"
          >
            <m3e-fab-menu-trigger for="main-fab-menu">
              <m3e-icon name="add"></m3e-icon>
            </m3e-fab-menu-trigger>
          </m3e-fab>

          <m3e-fab-menu id="main-fab-menu" variant="primary">
            <m3e-fab-menu-item>
              <m3e-icon slot="icon" name="medication"></m3e-icon>门诊常规方案
            </m3e-fab-menu-item>
            <m3e-fab-menu-item
              ><m3e-icon slot="icon" name="note_add"></m3e-icon>
              <m3e-bottom-sheet-trigger for="add-sheet">
                自定义用药
              </m3e-bottom-sheet-trigger>
            </m3e-fab-menu-item>
          </m3e-fab-menu>
        </div>
      </mdui-tab-panel>

      <!-- 曲线校准 -->
      <mdui-tab-panel
        slot="panel"
        value="calibration-panel"
        style="padding: 8px"
      >
        <div>
          <m3e-search-bar clearable style="margin-top: 8px">
            <m3e-icon name="search" slot="leading"></m3e-icon>
            <input slot="input" placeholder="Search..." />
          </m3e-search-bar>
          <calibrationSettings />
          <calibration />
        </div>
        <!-- 校准 FAB -->
        <m3e-fab
          variant="primary"
          size="medium"
          :style="{
            position: 'fixed',
            right: '16px',
            bottom: barVisible ? '96px' : '16px',
            transition: 'bottom 0.3s',
          }"
        >
          <m3e-bottom-sheet-trigger for="calibration-sheet">
            <m3e-icon name="add"></m3e-icon>
          </m3e-bottom-sheet-trigger>
        </m3e-fab>
      </mdui-tab-panel>

      <!-- 用药方案 -->
      <mdui-tab-panel slot="panel" value="plan-panel" style="padding: 8px">
        <div>
          <planPage />
        </div>
        <!-- 方案 FAB -->
        <m3e-fab
          variant="primary"
          size="medium"
          :style="{
            position: 'fixed',
            right: '16px',
            bottom: barVisible ? '96px' : '16px',
            transition: 'bottom 0.3s',
          }"
        >
          <m3e-bottom-sheet-trigger for="plan-sheet">
            <m3e-icon name="add"></m3e-icon>
          </m3e-bottom-sheet-trigger>
        </m3e-fab>
      </mdui-tab-panel>
    </mdui-tabs>

    <!-- 自定义用药 Bottom Sheet（全屏） -->
    <m3e-bottom-sheet
      id="add-sheet"
      modal
      v-lit="{ handle: true, detents: ['full'], detent: 0 }"
      style="--m3e-bottom-sheet-border-radius: 0"
    >
      <div style="padding: 16px; height: 100%; overflow: auto">
        <addPage />
      </div>
    </m3e-bottom-sheet>

    <!-- 校准记录 Bottom Sheet -->
    <m3e-bottom-sheet
      id="calibration-sheet"
      modal
      v-lit="{ handle: true, detents: ['full'], detent: 0 }"
    >
      <div style="padding: 16px; overflow: auto; max-height: 80vh;">
        <h3 style="margin: 0 0 16px 0">添加校准记录</h3>
        <div style="display: flex; flex-direction: column; gap: 12px;">
          <m3e-form-field>
            <label slot="label">采血时间 (小时)</label>
            <input
              type="number"
              step="any"
              min="0"
              placeholder="例如 48"
              :value="calForm.timeH.value"
              @input="calForm.timeH.value = ($event.target as HTMLInputElement).value"
            />
            <span slot="supporting-text">距离第一次用药的小时数</span>
          </m3e-form-field>

          <m3e-form-field>
            <label slot="label">实测浓度</label>
            <input
              type="number"
              step="any"
              min="0"
              placeholder="例如 150"
              :value="calForm.concValue.value"
              @input="calForm.concValue.value = ($event.target as HTMLInputElement).value"
            />
          </m3e-form-field>

          <m3e-form-field>
            <label slot="label">浓度单位</label>
            <select
              :value="calForm.unit.value"
              @change="calForm.unit.value = ($event.target as HTMLSelectElement).value"
              style="width:100%;padding:8px;border-radius:var(--mdui-shape-corner-small);border:1px solid rgb(var(--mdui-color-outline));background:rgb(var(--mdui-color-surface));color:rgb(var(--mdui-color-on-surface))"
            >
              <option value="pg/mL">pg/mL</option>
              <option value="ng/mL">ng/mL</option>
              <option value="µg/mL">µg/mL</option>
              <option value="mg/L">mg/L</option>
              <option value="pmol/L">pmol/L</option>
            </select>
          </m3e-form-field>

          <m3e-form-field>
            <label slot="label">药物分组 (可选)</label>
            <input
              type="text"
              placeholder="例如 E2, CPA 或留空"
              :value="calForm.groupId.value"
              @input="calForm.groupId.value = ($event.target as HTMLInputElement).value"
            />
            <span slot="supporting-text">对应模拟结果中的 drug_name，留空则应用于全部</span>
          </m3e-form-field>

          <m3e-button
            variant="filled"
            :disabled="!calForm.canSave.value"
            @click="calForm.save()"
            style="width: 100%;"
          >
            <m3e-icon slot="icon" name="save"></m3e-icon>保存校准数据
          </m3e-button>
        </div>
      </div>
    </m3e-bottom-sheet>

    <!-- 用药方案 Bottom Sheet -->
    <m3e-bottom-sheet
      id="plan-sheet"
      modal
      v-lit="{ handle: true, detents: ['full'], detent: 0 }"
    >
      <div style="padding: 16px">
        <h3 style="margin: 0 0 8px 0">新建用药方案</h3>
        <addPage />
      </div>
    </m3e-bottom-sheet>

    <!-- 底部占位，防止被 FAB / bottom-app-bar 遮挡 -->
    <div style="height: 160px"></div>
  </div>
</template>
