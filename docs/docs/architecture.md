# 架构文档

## 整体架构

```
main.ts → App.vue (根布局)
              ├── m3e-theme (主题上下文)
              ├── mdui-layout
              │   ├── navrail (左侧导航)
              │   ├── topappbar (顶部栏)
              │   ├── router-view → pages/ (页面内容)
              │   └── mdui-navigation-drawer (右侧设置抽屉)
              └── bottombar (底部栏，手机)
```

## Web Component 兼容层

项目同时使用两套 Lit 组件库（`@m3e/web` + `mdui`），与 Vue 模板语法存在兼容问题。

### v-lit 指令

Vue 的 `:prop="value"` 绑定只设置 HTML attribute，Lit 组件需要通过 `requestUpdate()` 触发重渲染。`v-lit` 指令统一处理：

```vue
<!-- 属性值通过 v-lit 传递 -->
<m3e-bottom-sheet v-lit="{ handle: true, detents: ['full'], detent: 0 }" />

<!-- 复杂的非字符串值用 v-lit -->
<m3e-split-pane v-lit="splitProps" />
```

详见 `src/composables/useLitProps.ts`。

### 已知限制

1. Vue 组件不能放在 Web Component 的 Shadow DOM 内（如 `<m3e-tab-panel>` 内部）。需在外面用 `<div>` 包裹
2. Lit 组件初始化时可能抛出 `firstUpdated` 空值错误（非致命）
3. `toggle` 属性与 Vue 的 `:selected` 绑定冲突，需配合 `@input.prevent` 使用

## 主题系统

### 令牌桥接

m3e 生成 `--md-sys-color-*` 令牌，mdui 使用 `--mdui-color-*-light/dark` 令牌。

```
m3e-theme (种子色 + scheme) → --md-sys-color-primary
                                    ↓ captureScheme()
                              --mdui-color-primary-light
                              --mdui-color-primary-dark
```

### 初始化流程

```
loadAll()
  ├─ 读取 localStorage (mode + seedColor)
  ├─ 设置 m3e-theme.color / scheme → 首帧即正确
  ├─ setTheme() → mdui 跟随
  └─ captureCurrentScheme() → 异步抓当前 scheme 令牌（零闪烁）
      另一 scheme 首次切模式时懒抓取
```

### 并发抓取保护

连续快速换色时，旧 `captureBothSchemes()` 通过 `captureGeneration` 自动中止，避免过期结果覆盖新值。

## 响应式断点

| 断点 | 宽度 | 组件 |
|------|------|------|
| Phone | `< 640px` | `isPhone` |
| Tablet | `640–1200px` | `isTablet` |
| Desktop | `≥ 1200px` | `isDesktop` |

- **Phone**：顶部栏 + 底部导航 + 全宽内容
- **Tablet**：紧凑侧边导航 + 分栏面板
- **Desktop**：可展开侧边导航 + 分栏面板 + 右侧固定设置抽屉

## 抽屉状态管理

设置抽屉使用 `drawerOpen` ref 控制：

- **桌面端**：默认展开，非模态，contained
- **移动端**：通过导航栏设置按钮触发
- 遮罩层关闭和 ESC 由 mdui `close-on-overlay-click` / `close-on-esc` 处理
- 不再使用 `@closed` 事件同步（会引起状态混乱）

## GitHub Pages 部署

SPA 在 GitHub Pages 上的路由回退方案：

```
用户访问 /settings → 404 → 404.html
  → sessionStorage 存原始路径
  → 重定向到 /reValenceGUI/
  → main.ts 读取 sessionStorage
  → router.replace(原始路径)
```

详见 `public/404.html` 和 `src/main.ts`。
