# 组件指南

## composables（通用逻辑）

### useTheme

```ts
const { themeMode, effectiveDark, seedColor,
        setThemeMode, setSeedColor, setThemeModeAt, setSeedColorAt } = useTheme()
```

| 返回值 | 类型 | 说明 |
|--------|------|------|
| `themeMode` | `Ref<'light'\|'dark'\|'auto'>` | 当前明暗模式 |
| `effectiveDark` | `Ref<boolean>` | 是否实际为深色（含 auto 解析） |
| `seedColor` | `Ref<string>` | 当前主题色 hex |
| `setThemeMode(mode)` | 函数 | 切换明暗 |
| `setSeedColor(hex)` | 函数 | 切换主题色 |
| `setThemeModeAt(mode, e)` | 函数 | 从点击位置圆形揭幕切换 |
| `setSeedColorAt(hex, e)` | 函数 | 从点击位置圆形揭幕换色 |

### useSettings

```ts
const { settings, set } = useSettings()
// settings: Ref<{ topBarShrink: boolean, circularReveal: boolean }>
// set('topBarShrink', false)
```

### useBreakpoint

```ts
const { width, isPhone, isTablet, isDesktop } = useBreakpoint()
// isPhone: < 640, isTablet: 640-1200, isDesktop: ≥ 1200
```

### useSplitPane

```ts
const { paneRef, minLeftPaneWidth, minRightPaneWidth, splitProps } = useSplitPane({
  leftFraction: 0.25,   // 左面板最小宽度比例
  leftMin: 170,         // 左面板最小 px
  rightFraction: 0.5,   // 右面板最小宽度比例
  rightMin: 340,        // 右面板最小 px
  initValue: 25,        // 初始分割百分比
})
```

返回的 `splitProps` 可直接传入 `<m3e-split-pane v-lit="splitProps">`。

### useNavRailMode

```ts
const { navRef, isNavExpanded } = useNavRailMode()
// isNavExpanded: 侧边导航是否处于展开状态
```

## compose/app（布局组件）

### navrail.vue

左侧导航栏，使用 `m3e-nav-rail`。

**Props**：`drawerOpen`, `navigator`, `navItems`  
**Emits**：`toggle-drawer`

菜单按钮控制展开/折叠，FAB 快捷跳转 `add` 页面，底部设置按钮切换抽屉。

### topappbar.vue

顶部应用栏，使用 `mdui-top-app-bar`。

**Props**：`meta` (RouteMeta), `scrollTarget` (滚动容器选择器)

手机端支持 `shrink elevate` 收缩行为，通过 `useSettings().topBarShrink` 控制。

### bottombar.vue

底部导航栏，使用 `mdui-navigation-bar`。

**Props**：`activeTab`, `navigator`, `navItems`

自动过滤 `add` 和 `plan` 页面。支持滚动隐藏（`scroll-behavior="hide"`）。

## compose（业务组件）

| 组件 | 用途 |
|------|------|
| `history.vue` | 按日期分组的用药记录列表 |
| `calibration.vue` | 曲线校准数据面板 |
| `calibrationSettings.vue` | 校准参数设置 |
| `chart.vue` | 浓度曲线图表 |
| `charttitle.vue` | 图表标题/导出栏 |
| `drugTree.vue` | 物质分类树 |
| `drugEditor.vue` | 物质编辑器 |

## FAB 菜单模式

每个页面使用统一的 FAB + bottom-sheet 模式添加记录：

```vue
<!-- FAB 触发器 -->
<m3e-fab style="position: absolute; right: 16px; bottom: 16px">
  <m3e-bottom-sheet-trigger for="my-sheet">
    <m3e-icon name="add" />
  </m3e-bottom-sheet-trigger>
</m3e-fab>

<!-- 全屏 Bottom Sheet -->
<m3e-bottom-sheet id="my-sheet" modal
  v-lit="{ handle: true, detents: ['full'], detent: 0 }">
  <div style="padding: 16px">
    <h3>标题</h3>
    <addPage />
  </div>
</m3e-bottom-sheet>
```

移动端 FAB 通过 `MutationObserver` 监听 `mdui-navigation-bar` 的 `hide` 属性，动态调整 `bottom` 值（bar 显示时 `96px`，隐藏时 `16px`）。
