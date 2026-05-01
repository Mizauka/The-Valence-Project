# Vue 页面文档

## 路由配置

| 路径 | 名称 | 组件 | 说明 |
|------|------|------|------|
| `/` | home | `HomePage.vue` | 首页：浓度曲线图表 |
| `/add-dose` | add-dose | `AddDosePage.vue` | 添加给药记录 |
| `/history` | history | `HistoryPage.vue` | 给药历史 |
| `/drug-library` | drug-library | `DrugLibraryPage.vue` | 药物库管理 |
| `/calibration` | calibration | `CalibrationPage.vue` | 模型校准 |
| `/settings` | settings | `SettingsPage.vue` | 设置 |

---

## App.vue — 根组件

**文件位置**：`src/App.vue`

### 模板结构

```
mdui-layout
├── mdui-navigation-drawer（桌面端侧边栏）
│   └── mdui-list（导航项列表）
├── mdui-layout-main
│   ├── mdui-top-app-bar（顶栏 + 菜单按钮）
│   ├── div.page-container → <router-view />
│   └── mdui-bottom-app-bar（移动端底部导航栏）
```

### 导航项

| 路由 | 图标 | 标签 |
|------|------|------|
| `/` | `home` | 首页 |
| `/add-dose` | `add_circle` | 记录 |
| `/history` | `history` | 历史 |
| `/drug-library` | `list_alt` | 药物库 |
| `/calibration` | `tune` | 校准 |
| `/settings` | `settings` | 设置 |

### 响应式布局
- **桌面端**：侧边 `mdui-navigation-drawer` + 顶部菜单按钮
- **移动端**：底部 `mdui-navigation-bar`，通过 `window.innerWidth < 768` 判断

### 状态变量

| 变量 | 说明 |
|------|------|
| `drawerOpen` | 侧边栏开关状态 |
| `isDesktop` | 桌面端判定（width >= 768） |
| `currentRoute` | 当前路由路径 |
| `navItems` | 导航项配置数组 |

---

## HomePage.vue — 首页图表

**文件位置**：`src/pages/HomePage.vue`

### 功能概述
- 展示所有药物血药浓度随时间变化的曲线
- 横向时间范围滑块（mdui-range-slider）
- 竖向 Y 轴数值滑块（mdui-slider，CSS 旋转）
- 图例交互：点击切换曲线显示/隐藏
- 导出 CSV / JSON 数据

### 模板结构

```
page-header（标题 + 导出按钮）
chart-wrapper（flex 行）
├── y-slider-container（竖向滑块 + 顶部/底部标签）
│   ├── yMaxDisplay（当前 Y 轴最大值）
│   ├── mdui-slider（旋转 90°）
│   └── "0"
└── chart-container（canvas + 占位符）
time-range-control（时间范围滑块 + 起止标签）
summary-card（给药记录计数）
```

### 核心状态

| 变量 | 类型 | 说明 |
|------|------|------|
| `chartInstance` | `Chart \| null` | Chart.js 实例 |
| `cachedSimResults` | `SimulationOutput[]` | 缓存的仿真结果 |
| `currentYMax` | `ref<number>` | 当前 Y 轴最大值 |
| `ySliderMin/Max/Step` | `ref<number>` | Y 滑块范围参数 |
| `currentRangeStart/End` | `ref<number>` | 时间范围起止（毫秒） |
| `curveVisibility` | `ref<Object>` | 曲线可见性状态映射 |

### 关键函数

| 函数 | 说明 |
|------|------|
| `renderChart()` | 运行仿真 → 构建 Chart.js 数据集 → 创建图表 |
| `autoScaleUnit(rawValues)` | 根据裸值自动选择显示单位（mg/L / µg/mL / ng/mL / pg/mL） |
| `calcNiceYMax(maxVal)` | 将浓度最大值向上取整到 50 的倍数 |
| `onYSliderInput(e)` | Y 滑块拖动 → 更新 chart y.max |
| `onRangeInput/Change(e)` | 时间范围滑块 → 更新 chart x.min/x.max |
| `syncYSliderSize()` | 动态计算 slider 高度适配容器 |
| `formatXLabel(ms)` | 毫秒时间戳 → `MM/DD HH:mm` 格式 |
| `exportCSV()` / `exportJSON()` | 导出图表数据 |

### Chart.js 配置要点

- **图例**：自定义 `generateLabels` 在每条曲线名后追加单位（如 `E2 (ng/mL)`）
- **图例点击**：切换 `dataset.hidden`，同步 `curveVisibility`
- **X 轴**：`linear` 类型，毫秒时间戳
- **Y 轴**：`beginAtZero: true`，max 由滑块控制
- **Tooltip**：根据单位类型不同精度显示（pg 整数、mg 3 位小数等）

### ResizeObserver
监听 `y-slider-container` 尺寸变化，自动调用 `syncYSliderSize()` 调整竖滑块的 CSS width。

---

## AddDosePage.vue — 添加给药记录

**文件位置**：`src/pages/AddDosePage.vue`

### 两步表单

**第一步：选择药物**
- 搜索框（mdui-text-field）：按名称或 drug_id 搜索
- 来源筛选（mdui-tabs）：全部 / HRT / Journal / 自定义
- 药物列表：显示名称、房室模型、半衰期、等效因子
- 分页加载：每次显示 80 条，滚动加载更多

**第二步：记录剂量**
- 选中药物的基本信息横幅
- 剂量输入（label 显示当前给药方式的单位）
- 给药方式选择（mdui-select）：只显示该药物支持的给药途径
- 给药时间选择（datetime-local）

### 给药方式映射

| 内部标识 | 中文标签 |
|----------|----------|
| `oral` | 口服 |
| `sublingual` | 舌下 |
| `buccal` | 颊黏膜 |
| `insufflated` | 鼻吸 |
| `rectal` | 直肠 |
| `injection` | 注射 |
| `transdermal` | 透皮 |
| `gel` | 凝胶 |
| `smoked` | 吸入(烟) |
| `inhaled` | 吸入 |

### 剂量单位转换（保存时）

```javascript
// 用户输入 → 存储（mg）
if (doseUnit === 'µg') amountMG = amount / 1000
else if (doseUnit === 'ng') amountMG = amount / 1000000
else if (doseUnit === 'pg') amountMG = amount / 1000000000
// mg 和 mL 不转换
```

### 时间戳处理
用户选择的本地时间 → `new Date().getTime() / 1000 / 3600`（从毫秒转为十进制小时）。

---

## HistoryPage.vue — 给药历史

**文件位置**：`src/pages/HistoryPage.vue`

### 功能
- 按日期分组展示所有给药记录
- 显示药物名称、剂量+单位、给药方式、时间
- 支持删除操作（带确认对话框）

### 数据结构

| 变量 | 说明 |
|------|------|
| `doses` | 从 `getAllDoses()` 获取的剂量数组（已含 display_amount/display_unit） |
| `groupedDoses` | 按日期分组的计算属性，倒序排列 |

### 关键函数
- `formatDose(dose)` — 格式化剂量显示（整数/1位小数/2位小数自适应）
- `formatTimestamp(ts)` — 时间戳 → { date, time } 对象
- `confirmDelete(dose)` — 打开删除确认对话框
- `doDelete()` — 执行删除并更新列表

---

## DrugLibraryPage.vue — 药物库

**文件位置**：`src/pages/DrugLibraryPage.vue`

### 功能
- 浏览三类药物：HRT / Journal / 自定义
- 查看药物参数（半衰期、分布容积、清除率、等效因子）
- 从药物库直接跳转到添加剂量
- 自定义药物支持新增和删除

### 新增自定义药物表单

支持的参数：
- `name` — 药物名称
- `model_type` — 一室 / 二室 / 多室
- `half_life` — 半衰期 (h)
- `volume_of_distribution` — 分布容积 (L/kg)
- `clearance` — 清除率 (L/h/kg)
- `ka` — 吸收速率常数 (1/h)
- `bioavailability` — 生物利用度 F

---

## CalibrationPage.vue — 模型校准

**文件位置**：`src/pages/CalibrationPage.vue`

### 功能
提供后验校准算法的选择界面（当前为 UI 占位，逻辑待实现）。

| 算法 | 说明 |
|------|------|
| EKF（扩展卡尔曼滤波） | 通过观测值逐步修正参数，适用于有规律采血检测 |
| OU-Kalman | 基于 Ornstein-Uhlenbeck 过程的前向后向平滑器，适用于稀疏观测 |

---

## SettingsPage.vue — 设置

**文件位置**：`src/pages/SettingsPage.vue`

### 功能区域

**体重设置**
- 输入框实时显示当前体重
- 输入时改变显示值，失焦时保存到 OPFS + WASM 引擎

**数据存储**
- 选择并同步到本地文件夹（File System Access API）
- 显示当前同步目录和权限状态
- 重新授权按钮（权限被撤销后）
- 数据默认使用 OPFS 持久化

**数据管理**
- 导出数据：从 WASM 引擎导出完整 JSON 并下载
- 导入数据：选择 JSON 文件 → 导入到引擎 → 刷新体重显示

**关于**
- 版本号和隐私说明
