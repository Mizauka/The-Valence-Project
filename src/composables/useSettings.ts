import { ref } from 'vue'

const STORAGE_KEY = 'mizauka-settings'

export interface AppSettings {
  topBarShrink: boolean    // 顶栏收缩动画
  circularReveal: boolean  // 主题切换圆环动画
}

function load(): AppSettings {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (raw) return JSON.parse(raw)
  } catch { /* ignore */ }
  return { topBarShrink: true, circularReveal: true }
}

function save(s: AppSettings) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(s))
}

const _settings = ref<AppSettings>(load())

export function useSettings() {
  function set<K extends keyof AppSettings>(key: K, value: AppSettings[K]) {
    _settings.value[key] = value
    save(_settings.value)
  }
  return { settings: _settings, set }
}
