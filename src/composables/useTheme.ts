import { ref, onMounted } from 'vue'
import { setTheme, setColorScheme } from 'mdui'
import { useSettings } from './useSettings'

export type ThemeMode = 'light' | 'dark' | 'auto'

const STORAGE_MODE = 'mizauka-theme-mode'
const STORAGE_COLOR = 'mizauka-theme-color'

export const PRESET_COLORS = [
  { name: '默认蓝', hex: '#4285F4' },
  { name: '青绿', hex: '#009688' },
  { name: '紫罗兰', hex: '#7C4DFF' },
  { name: '玫瑰', hex: '#E91E63' },
  { name: '琥珀', hex: '#FF9800' },
  { name: '石墨', hex: '#607D8B' },
]

/** m3e --md-sys-color-{key} → mdui --mdui-color-{key} 映射 */
const TOKEN_KEYS = [
  'background', 'error', 'error-container', 'inverse-on-surface', 'inverse-primary',
  'inverse-surface', 'on-background', 'on-error', 'on-error-container', 'on-primary',
  'on-primary-container', 'on-primary-fixed', 'on-primary-fixed-variant', 'on-secondary',
  'on-secondary-container', 'on-secondary-fixed', 'on-secondary-fixed-variant', 'on-surface',
  'on-surface-variant', 'on-tertiary', 'on-tertiary-container', 'on-tertiary-fixed',
  'on-tertiary-fixed-variant', 'outline', 'outline-variant', 'primary', 'primary-container',
  'primary-fixed', 'primary-fixed-dim', 'scrim', 'secondary', 'secondary-container',
  'secondary-fixed', 'secondary-fixed-dim', 'shadow', 'surface', 'surface-bright',
  'surface-container', 'surface-container-high', 'surface-container-highest',
  'surface-container-low', 'surface-container-lowest', 'surface-dim', 'surface-tint',
  'surface-variant', 'tertiary', 'tertiary-container', 'tertiary-fixed', 'tertiary-fixed-dim',
]

function hexToRgb(hex: string): string {
  const h = hex.replace('#', '')
  return `${parseInt(h.slice(0, 2), 16)}, ${parseInt(h.slice(2, 4), 16)}, ${parseInt(h.slice(4, 6), 16)}`
}

// ─── 预缓存机制 ───

function getThemeElement(): any {
  return document.querySelector('m3e-theme')
}

/** 等待 m3e-theme 完成渲染（2 帧） */
function waitForPaint(): Promise<void> {
  return new Promise<void>(resolve => {
    requestAnimationFrame(() => requestAnimationFrame(() => resolve()))
  })
}

/**
 * 以指定 scheme 渲染 m3e-theme，抓取所有 --md-sys-color-* 令牌，
 * 写入 :root 的 --mdui-color-{key}-{scheme}。
 */
async function captureScheme(scheme: 'light' | 'dark') {
  const el = getThemeElement()
  if (!el) return
  el.scheme = scheme
  el.requestUpdate?.()
  await waitForPaint()

  const cs = getComputedStyle(el)
  const root = document.documentElement.style
  for (const key of TOKEN_KEYS) {
    const val = cs.getPropertyValue(`--md-sys-color-${key}`).trim()
    if (!val) continue
    const rgb = val.startsWith('#') ? hexToRgb(val) : val
    root.setProperty(`--mdui-color-${key}-${scheme}`, rgb)
  }
}

/** 并发抓取保护：新抓取开始时，旧抓取自动中止 */
let captureGeneration = 0

/** 记录每个 scheme 最后一次抓取时的种子色，用于判断是否需要重新抓取 */
let capturedSeedForLight = ''
let capturedSeedForDark = ''

/** 抓取当前有效 scheme（不修改 m3e-theme.scheme，零闪烁） */
async function captureCurrentScheme() {
  const el = getThemeElement()
  if (!el) return
  const effective = resolveEffectiveDark() ? 'dark' : 'light'
  // 等待当前渲染完成（不切换 scheme，无闪烁）
  await waitForPaint()
  const cs = getComputedStyle(el)
  const root = document.documentElement.style
  for (const key of TOKEN_KEYS) {
    const val = cs.getPropertyValue(`--md-sys-color-${key}`).trim()
    if (!val) continue
    const rgb = val.startsWith('#') ? hexToRgb(val) : val
    root.setProperty(`--mdui-color-${key}-${effective}`, rgb)
  }
  if (effective === 'light') capturedSeedForLight = seedColor.value
  else capturedSeedForDark = seedColor.value
}

async function captureBothSchemes() {
  const generation = ++captureGeneration
  const savedMode = themeMode.value
  await captureScheme('light')
  if (generation !== captureGeneration) return // 被新抓取中止
  await captureScheme('dark')
  if (generation !== captureGeneration) return
  // 恢复原主题模式
  const el = getThemeElement()
  if (el) el.scheme = schemeAttr(savedMode)
  setTheme(savedMode)
  syncDataTheme()
}

// ─── 响应式状态 ───

const themeMode = ref<ThemeMode>('auto')
const effectiveDark = ref(false)
const seedColor = ref('#4285F4')

let mediaQuery: MediaQueryList | null = null

function resolveEffectiveDark(): boolean {
  return themeMode.value === 'auto'
    ? window.matchMedia('(prefers-color-scheme: dark)').matches
    : themeMode.value === 'dark'
}

/** m3e-theme 不接受字符串 'auto'，需转为 null 表示跟随系统 */
function schemeAttr(mode: ThemeMode): 'light' | 'dark' | null {
  return mode === 'auto' ? null : mode
}

/** 将 themeMode 同步到 <html data-theme> */
function syncDataTheme() {
  document.documentElement.dataset.theme = resolveEffectiveDark() ? 'dark' : 'light'
}

// ─── 核心操作（同步，无重计算） ───

/** 切换主题色（只更新当前 scheme，不闪烁；另一 scheme 延后到切模式时懒抓取） */
function applyColor(hex: string) {
  seedColor.value = hex
  setColorScheme(hex)
  const el = getThemeElement()
  if (el) { el.color = hex; el.requestUpdate?.() }
  localStorage.setItem(STORAGE_COLOR, hex)
  // 只抓当前有效 scheme，不切 scheme → 零闪烁
  captureCurrentScheme()
  // 标记另一 scheme 过期，下次切模式时再抓
  const effective = resolveEffectiveDark() ? 'dark' : 'light'
  if (effective === 'light') capturedSeedForDark = ''
  else capturedSeedForLight = ''
}

/** 切换明暗模式 */
function applyTheme(mode: ThemeMode) {
  setTheme(mode)
  const el = getThemeElement()
  if (el) el.scheme = schemeAttr(mode)
  effectiveDark.value = resolveEffectiveDark()
  syncDataTheme()
  localStorage.setItem(STORAGE_MODE, mode)
  // 只抓当前有效 scheme（不闪烁），另一 scheme 标记过期懒抓取
  const effective = resolveEffectiveDark() ? 'dark' : 'light'
  const captured = effective === 'light' ? capturedSeedForLight : capturedSeedForDark
  if (captured !== seedColor.value) {
    captureCurrentScheme()
    if (effective === 'light') capturedSeedForDark = ''
    else capturedSeedForLight = ''
  }
}

// ─── 初始化 ───

function loadAll() {
  themeMode.value = (localStorage.getItem(STORAGE_MODE) as ThemeMode) ?? 'auto'
  seedColor.value = localStorage.getItem(STORAGE_COLOR) ?? '#4285F4'

  // 先应用最终主题，让首帧即正确
  const el = getThemeElement()
  if (el) {
    el.color = seedColor.value
    el.scheme = schemeAttr(themeMode.value)
    el.requestUpdate?.()
  }
  setColorScheme(seedColor.value)
  setTheme(themeMode.value)
  syncDataTheme()

  // 只抓当前 scheme（不闪烁），另一 scheme 首次切模式时懒抓取
  captureCurrentScheme()
}

let _initialized = false

function setThemeMode(mode: ThemeMode) { themeMode.value = mode; applyTheme(mode) }
function setSeedColor(hex: string) { applyColor(hex) }

// ─── 圆形揭幕动画 ───

/** 以 (x, y) 为圆心圆形揭幕切换 */
function circularReveal(callback: () => void, x: number, y: number) {
  const r = Math.hypot(
    Math.max(x, window.innerWidth - x),
    Math.max(y, window.innerHeight - y),
  )
  document.documentElement.style.setProperty('--reveal-x', `${x}px`)
  document.documentElement.style.setProperty('--reveal-y', `${y}px`)
  document.documentElement.style.setProperty('--reveal-r', `${r}px`)

  if ('startViewTransition' in document) {
    ;(document as any).startViewTransition(() => callback())
  } else {
    callback()
  }
}

function setThemeModeAt(mode: ThemeMode, event: MouseEvent) {
  const { settings } = useSettings();
  if (settings.value.circularReveal) {
    circularReveal(() => setThemeMode(mode), event.clientX, event.clientY);
  } else {
    setThemeMode(mode);
  }
}

function setSeedColorAt(hex: string, event: MouseEvent) {
  const { settings } = useSettings();
  if (settings.value.circularReveal) {
    circularReveal(() => setSeedColor(hex), event.clientX, event.clientY);
  } else {
    setSeedColor(hex);
  }
}

// ─── 导出 ───

export function useTheme() {
  if (!_initialized) {
    _initialized = true
    onMounted(() => {
      loadAll()
      mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
      const handler = () => {
        effectiveDark.value = resolveEffectiveDark()
        syncDataTheme()
      }
      mediaQuery.addEventListener('change', handler)
    })
  }
  return {
    themeMode: themeMode as Readonly<typeof themeMode>,
    effectiveDark: effectiveDark as Readonly<typeof effectiveDark>,
    seedColor: seedColor as Readonly<typeof seedColor>,
    setThemeMode,
    setSeedColor,
    setThemeModeAt,
    setSeedColorAt,
  }
}
