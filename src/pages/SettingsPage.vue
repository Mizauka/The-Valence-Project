<script setup lang="ts">
import { ref } from "vue";
import { useTheme, PRESET_COLORS } from "../composables/useTheme";
import { useSettings } from "../composables/useSettings";
import { useSettingsData } from "../composables/useSettingsData";

const { themeMode, seedColor, setThemeModeAt, setSeedColorAt, setSeedColor } =
  useTheme();
const { settings, set } = useSettings();

const customColor = ref(seedColor.value);
const isCustom = ref(false);
const syncEnabled = ref(false);

// Data management
const {
  weightDisplay, savedDirName, dirPermissionGranted,
  importInput, importing, exporting, syncMessage,
  onWeightInput, saveWeight, syncToFolder, reauthorizeDir,
  doExport, triggerImport, doImport,
} = useSettingsData();
</script>

<template>
  <mdui-list style="padding: 8px">
    <mdui-collapse value="theme">
      <!-- 主题 -->
      <mdui-collapse-item value="theme">
        <mdui-list-item slot="header" icon="palette" rounded
          >主题</mdui-list-item
        >
        <div
          style="
            margin-left: 2.5rem;
            padding-right: 8px;
            display: flex;
            flex-direction: column;
            gap: 16px;
            padding-bottom: 12px;
          "
        >
          <!-- 明暗模式 -->
          <div>
            <div style="font-weight: 500; margin-bottom: 4px">明暗模式</div>
            <m3e-segmented-button style="width: 100%; margin-top: 8px">
              <m3e-button-segment
                :checked="themeMode === 'auto'"
                @click="setThemeModeAt('auto', $event)"
              >
                <m3e-icon slot="icon" name="contrast"></m3e-icon>系统
              </m3e-button-segment>
              <m3e-button-segment
                :checked="themeMode === 'light'"
                @click="setThemeModeAt('light', $event)"
              >
                <m3e-icon slot="icon" name="light_mode"></m3e-icon>浅色
              </m3e-button-segment>
              <m3e-button-segment
                :checked="themeMode === 'dark'"
                @click="setThemeModeAt('dark', $event)"
              >
                <m3e-icon slot="icon" name="dark_mode"></m3e-icon>深色
              </m3e-button-segment>
            </m3e-segmented-button>
          </div>

          <!-- 主题色 -->
          <div>
            <div style="font-weight: 500; margin-bottom: 4px">主题色</div>
            <div
              style="display: flex; flex-wrap: wrap; gap: 8px; margin-top: 8px"
            >
              <div
                v-for="c in PRESET_COLORS"
                :key="c.hex"
                :style="{
                  width: '36px',
                  height: '36px',
                  borderRadius: '50%',
                  backgroundColor: c.hex,
                  border:
                    seedColor === c.hex
                      ? '3px solid var(--md-sys-color-on-surface)'
                      : '3px solid transparent',
                  cursor: 'pointer',
                }"
                :title="c.name"
                @click="
                  setSeedColorAt(c.hex, $event);
                  isCustom = false;
                "
              ></div>
              <label
                :style="{
                  width: '36px',
                  height: '36px',
                  borderRadius: '50%',
                  background:
                    'conic-gradient(red,yellow,lime,aqua,blue,magenta,red)',
                  border: isCustom
                    ? '3px solid var(--md-sys-color-on-surface)'
                    : '3px solid transparent',
                  cursor: 'pointer',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                }"
                title="自定义"
              >
                <m3e-icon
                  name="add"
                  style="
                    font-size: 18px;
                    color: #fff;
                    filter: drop-shadow(0 0 2px #000);
                  "
                ></m3e-icon>
                <input
                  type="color"
                  :value="customColor"
                  style="position: absolute; opacity: 0; width: 0; height: 0"
                  @input="
                    isCustom = true;
                    customColor = ($event.target as HTMLInputElement).value;
                    setSeedColor(customColor);
                  "
                />
              </label>
            </div>
          </div>
        </div>
      </mdui-collapse-item>

      <!-- 图像性能 -->
      <mdui-collapse-item value="performance">
        <mdui-list-item slot="header" icon="animation" rounded>
          图像性能
          <mdui-badge slot="end-icon" variant="primary">卡顿</mdui-badge>
        </mdui-list-item>
        <div
          style="
            margin-left: 2.5rem;
            padding-right: 8px;
            display: flex;
            flex-direction: column;
            gap: 16px;
            padding-bottom: 12px;
          "
        >
          <!-- 动画 -->
          <div>
            <div style="font-weight: 500; margin-bottom: 4px">动画</div>
            <div
              style="
                display: flex;
                flex-direction: column;
                gap: 4px;
                margin-top: 8px;
              "
            >
              <div
                style="
                  display: flex;
                  align-items: center;
                  justify-content: space-between;
                  padding: 8px 0;
                "
              >
                <span>顶栏收缩动画</span>
                <mdui-switch
                  :checked="settings.topBarShrink"
                  @change="
                    set(
                      'topBarShrink',
                      ($event.target as HTMLInputElement).checked,
                    )
                  "
                />
              </div>
              <div
                style="
                  display: flex;
                  align-items: center;
                  justify-content: space-between;
                  padding: 8px 0;
                "
              >
                <span>主题圆形揭幕动画</span>
                <mdui-switch
                  :checked="settings.circularReveal"
                  @change="
                    set(
                      'circularReveal',
                      ($event.target as HTMLInputElement).checked,
                    )
                  "
                />
              </div>
            </div>
          </div>

          <!-- 颜色映射 -->
          <div>
            <div style="font-weight: 500; margin-bottom: 4px">
              m3e → mdui 颜色映射
            </div>
            <div
              style="
                display: flex;
                align-items: center;
                justify-content: space-between;
                padding: 8px 0;
                margin-top: 8px;
              "
            >
              <span>启用颜色同步</span>
              <mdui-switch
                :checked="syncEnabled"
                @change="
                  syncEnabled = ($event.target as HTMLInputElement).checked
                "
              />
            </div>
            <div
              v-if="syncEnabled"
              style="
                font-size: var(--mdui-typescale-label-small-size);
                color: var(--md-sys-color-on-surface-variant);
                margin-top: 4px;
              "
            >
              同步中… 会频繁读取 m3e-theme 计算样式并写入 mdui
              令牌，关闭可减少重绘开销
            </div>
          </div>
        </div>
      </mdui-collapse-item>

      <!-- 数据管理 -->
      <mdui-collapse-item value="data">
        <mdui-list-item slot="header" icon="storage" rounded>
          数据管理
        </mdui-list-item>
        <div
          style="
            margin-left: 2.5rem;
            padding-right: 8px;
            display: flex;
            flex-direction: column;
            gap: 16px;
            padding-bottom: 12px;
          "
        >
          <!-- 体重 -->
          <div>
            <div style="font-weight: 500; margin-bottom: 4px;">体重设置</div>
            <p style="font-size:0.75rem;color:rgb(var(--mdui-color-on-surface-variant));margin-bottom:8px">
              体重将作为常量用于药代动力学计算的分布容积调整
            </p>
            <div style="display:flex;gap:8px;align-items:center;">
              <m3e-form-field style="flex:1;">
                <label slot="label">体重 (kg)</label>
                <input
                  type="number"
                  step="any"
                  min="1"
                  :value="weightDisplay"
                  @input="onWeightInput"
                  @change="saveWeight"
                />
              </m3e-form-field>
            </div>
          </div>

          <!-- 数据同步 -->
          <div>
            <div style="font-weight: 500; margin-bottom: 4px;">数据同步</div>
            <p style="font-size:0.75rem;color:rgb(var(--mdui-color-on-surface-variant));margin-bottom:8px">
              数据默认使用浏览器持久化存储（OPFS），关闭浏览器后不会丢失。可同步到本地文件夹。
            </p>
            <m3e-button variant="filled" @click="syncToFolder" style="width:100%;margin-bottom:8px;">
              <m3e-icon slot="icon" name="folder_open"></m3e-icon>
              选择并同步到本地文件夹
            </m3e-button>
            <m3e-button
              v-if="savedDirName && !dirPermissionGranted"
              variant="outlined"
              @click="reauthorizeDir"
              style="width:100%;margin-bottom:8px;"
            >
              <m3e-icon slot="icon" name="lock_open"></m3e-icon>
              重新授权同步
            </m3e-button>
            <p v-if="savedDirName" style="font-size:0.75rem;margin-top:4px;">
              同步目录: {{ savedDirName }}/data/
              <span v-if="dirPermissionGranted" style="color:rgb(var(--mdui-color-primary))">● 已授权</span>
              <span v-else style="color:rgb(var(--mdui-color-error))">● 需重新授权</span>
            </p>
            <p v-if="syncMessage" style="font-size:0.75rem;color:rgb(var(--mdui-color-primary));margin-top:4px;">
              {{ syncMessage }}
            </p>
          </div>

          <!-- 导入导出 -->
          <div>
            <div style="font-weight: 500; margin-bottom: 4px;">数据备份</div>
            <div style="display:flex;gap:8px;">
              <m3e-button variant="filled" @click="doExport" :disabled="exporting" style="flex:1;">
                <m3e-icon slot="icon" name="download"></m3e-icon>
                {{ exporting ? '导出中...' : '导出数据' }}
              </m3e-button>
              <m3e-button variant="outlined" @click="triggerImport" :disabled="importing" style="flex:1;">
                <m3e-icon slot="icon" name="upload"></m3e-icon>
                {{ importing ? '导入中...' : '导入数据' }}
              </m3e-button>
              <input
                ref="importInput"
                type="file"
                accept=".json"
                style="display:none"
                @change="doImport"
              />
            </div>
          </div>
        </div>
      </mdui-collapse-item>
    </mdui-collapse>
  </mdui-list>
  <p style="height: 120px"></p>
</template>
