# 药历管理系统前端 (Mizauka)

基于 Vue 3 + Material Design 3 的药历管理与浓度曲线追踪系统的前端。

## 技术栈

| 类别 | 技术 |
|------|------|
| 框架 | Vue 3.5 (Composition API + `<script setup>`) |
| 语言 | TypeScript 6.0 |
| 构建 | Vite 8 |
| UI | m3e (Material 3 Expressive) + mdui v2 |
| 图标 | Material Symbols + Material Icons |
| 路由 | Vue Router 5 (History 模式) |
| 包管理 | pnpm |

## 项目结构

```
src/
├── main.ts                    # 入口：挂载 Vue、注册 mdui/m3e、SPA 回退恢复
├── App.vue                    # 根布局：m3e-theme + mdui-layout + navrail + drawer
├── style.css                  # 全局样式：字体、主题动画、圆形揭幕
├── router/
│   └── index.ts               # 路由定义（含 GitHub Pages base 适配）
├── composables/               # 通用逻辑
│   ├── useTheme.ts            # 主题管理（明暗/种子色/双套令牌预缓存）
│   ├── useSettings.ts         # 持久化设置（localStorage）
│   ├── useBreakpoint.ts       # 响应式断点（phone < 640, tablet < 1200, desktop ≥ 1200）
│   ├── useLitProps.ts         # v-lit 指令（Vue → Lit 属性桥接）
│   ├── useNavRailMode.ts      # 导航栏扩展/折叠状态
│   ├── useSplitPane.ts        # 分栏面板配置生成
│   ├── useParentWidth.ts      # ResizeObserver 宽度追踪
│   └── routes.ts              # 共享路由元数据（图标、标题）
├── pages/                     # 页面组件
│   ├── homePage.vue           # 首页：浓度曲线 + 用药记录
│   ├── historyPage.vue        # 药历核对：记录列表 + 曲线校准 + 用药方案（移动端 tabs）
│   ├── libPage.vue            # 物质列表：药物树 + 编辑器
│   ├── planPage.vue           # 用药方案：方案列表 + 编辑器
│   ├── addPage.vue            # 添加记录
│   └── settingsPage.vue       # 设置：主题 / 动画性能
└── compose/                   # 可复用组件
    ├── app/                   # 全局布局组件
    │   ├── navrail.vue        # 左侧导航栏（桌面/平板）
    │   ├── topappbar.vue      # 顶部应用栏
    │   └── bottombar.vue      # 底部导航栏（手机）
    ├── history.vue            # 用药记录列表
    ├── calibration.vue        # 曲线校准面板
    ├── chart.vue / charttitle.vue  # 浓度曲线图
    ├── drugTree.vue / drugEditor.vue  # 物质树 / 编辑器
    └── ...
```

## 页面路由

| 路径 | 名称 | 标题 | 图标 |
|------|------|------|------|
| `/` | home | 浓度曲线 | dataset |
| `/history` | history | 药历核对 | watch_later |
| `/library` | library | 物质列表 | medication |
| `/plan` | plan | 用药方案 | clinical_notes |
| `/add` | add | 添加记录 | add |
| `/settings` | settings | 设置 | settings |

> `add`、`settings`、`plan` 不在底部/侧边导航栏中显示，通过 FAB 和设置按钮访问。

## 响应式布局

| 断点 | 宽度 | 布局 |
|------|------|------|
| Phone | < 640px | 顶部栏 + 底部导航 + 移动端 tabs/FAB |
| Tablet | 640–1200px | 侧边导航栏 + 分栏面板 |
| Desktop | ≥ 1200px | 侧边导航栏（可展开）+ 分栏面板 + 右侧设置抽屉 |

## 主题系统

- **明暗模式**：浅色 / 深色 / 跟随系统
- **主题色**：6 种预设 + 自定义取色器
- **双套令牌预缓存**：light / dark 两套 mdui 颜色令牌分别抓取并缓存到 `:root`，切换模式时不执行 JS 重计算
- **圆形揭幕动画**：基于 View Transitions API 的 clip-path 圆形展开
- **持久化**：主题设置保存到 localStorage

## 开发

```bash
# 安装依赖
pnpm install

# 启动开发服务器
pnpm dev

# 类型检查
pnpm type-check

# 生产构建
pnpm build

# 预览生产构建
pnpm preview
```

### 环境要求

- Node.js ≥ 20.19 或 ≥ 22.12
- pnpm（推荐通过 `corepack enable` 启用）

## 部署

项目通过 GitHub Actions 自动部署到 GitHub Pages：

1. 推送代码到 `main` 分支
2. GitHub Actions 自动执行构建并部署
3. 在仓库 **Settings → Pages** 中选择 **GitHub Actions** 作为源

### 自定义域名 / 不同仓库名

若使用自定义域名或仓库名不是 `reValenceGUI`，需修改以下文件中的 base 路径：
- `vite.config.ts` 的 `base` 选项
- `src/router/index.ts` 的 `BASE` 常量
- `public/404.html` 的 `location.replace` 目标

## 许可证

MIT


### Compile and Hot-Reload for Development

```sh
pnpm dev
```

### Type-Check, Compile and Minify for Production

```sh
pnpm build
```
