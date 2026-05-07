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
- 展示所有药物血药浓度随时间变化的曲线（ECharts）
- **智能多物质单位归一化**：以原始数值最大的物质（锚点）为基准确定 Y 轴范围，其余物质自动换算到最佳可视单位
- **校准区间叠加**：OU-Kalman / 比率插值校准曲线 + 95%CI / 68%CI 半透明置信带
- **图例交互**：点击切换曲线显隐，CI 区间与校准曲线联动
- 导出 CSV / JSON 数据
- dataZoom 滑块进行时间范围缩放

### 核心状态

| 变量 | 类型 | 说明 |
|------|------|------|
| `chart` | `echarts.ECharts \| null` | ECharts 实例 |
| `cachedSimResults` | `SimulationOutput[]` | 缓存的仿真结果 |
| `substanceMeta` | `Array` | 每物质元数据：name, nativeUnit, displayUnit, convFactor, color, idx |
| `canvasUnit` | `string` | 锚点物质的单位（Y 轴基准） |

### 关键函数

| 函数 | 说明 |
|------|------|
| `resolveDisplayUnits(simResults, calibBands)` | 智能单位解析：找锚点 → 定 Y 轴范围（含 68%CI 上界） → 其余物质自动选取最佳显示单位 |
| `autoPickUnit(nativeMax, nativeUnit, targetMax)` | 在 pg/mL / ng/mL / µg/mL / mg/L 中选取换算后最适合目标范围的单位 |
| `renderChart()` | 运行仿真 → 解析单位 → 构建 ECharts series → 创建图表 |
| `fmtTooltip(val, unit)` | 按单位精度格式化提示框数值 |
| `exportCSV()` / `exportJSON()` | 导出图表数据 |

### ECharts 配置要点

- **Y 轴**：纯数字轴，无单位标签（单位通过图例和提示框区分）
- **图例**：每条显示 `物质名 类型 (单位)`，如 `E2 原始 (pg/mL)`、`E2 校准 (pg/mL)`
- **CI 区间**：与校准曲线同名，图例切换联动显隐
- **legendselectchanged 事件**：图例切换时动态重算 Y 轴最大值
- **dataZoom**：slider + inside 双模式时间缩放
- **tooltip**：axis 触发，显示原始值 + 校准值 + 95%/68% CI 区间

### 单位归一化算法

```
1. 遍历所有模拟结果，记录 nativeUnit + 最大原始数值
2. 锚点 = 原始数值最大的物质（如 E2: 1016 pg/mL）
3. Y轴上限 = max(锚点峰值×1.15, 锚点68%CI上界) 向上取整
4. 其余物质：autoPickUnit() 在四种单位中选取
   换算后数值落在 Y轴范围 0.5%-500% 内的最佳单位
5. 每条曲线通过 convFactor 转换到各自 displayUnit 后绑图
```


---

## AddDosePage.vue — 添加给药记录

**文件位置**：`src/pages/AddDosePage.vue`

### 双面板滑动流程（useSlidingPanels）

使用 `useSlidingPanels(2)` 组合式函数实现双面板左右滑动过渡。
- **面板 0（左侧）**：药物搜索与选择
- **面板 1（右侧）**：剂量录入表单
- 选中药物后调用 `panels.advance(0)` 滑入面板 1
- 面板 1 返回按钮调用 `panels.back()` 滑回面板 0
- 移动端（<768px）自动切换为单面板逐页模式
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

### 四面板滑动流程（useSlidingPanels(4)）

| 面板 | 内容 | advance 触发 |
|------|------|-------------|
| 0 | 校准模型选择（比率插值 / OU-Kalman） | — |
| 1 | 校准记录列表 + 添加按钮 | `panels.advance(1)` |
| 2 | 物质组选择（搜索 + mdui-list） | `panels.advance(2)` |
| 3 | 血检数据表单（浓度 + 单位 + 时间） | 提交后 `panels.reset()` |

- `advance(fromPanel)` 带来源面板参数，仅最右侧可见面板可触发前进
- 移动端自动切换为单面板逐页模式

### 校准记录管理
- 使用 `mdui-list` / `mdui-list-item` 展示记录
- 点击记录弹出 `mdui-dialog` 编辑浓度、单位、时间
- 删除操作即时生效并同步到引擎

### 校准算法

| 算法 | 说明 |
|------|------|
| 比率插值 | 基于实测值与预测值的比率构建分段线性插值器，适用于少量观测数据 |
| OU-Kalman | Ornstein-Uhlenbeck 卡尔曼滤波器 + RTS 反向平滑器，适用于有规律采血检测 |

校准结果通过 `getCalibrationBand()` 返回 95%CI / 68%CI 区间，在 HomePage 图表中叠加显示。

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

---

## useSlidingPanels — 多步滑动面板组合式函数

**文件位置**：`src/composables/useSlidingPanels.ts`

### 用途
为多步表单 / 向导流程提供统一的滑动面板交互。

### API

| 返回值 | 类型 | 说明 |
|--------|------|------|
| `step` | `Ref<number>` | 当前步骤（0-based） |
| `isMobile` | `Ref<boolean>` | 是否为移动端（<768px） |
| `maxStep` | `ComputedRef<number>` | 最大步骤（桌面端 N-2，移动端 N-1） |
| `rightmostPanel` | `ComputedRef<number>` | 当前最右侧可见面板索引 |
| `trackWidthPercent` | `ComputedRef<number>` | 轨道宽度百分比 |
| `offsetPercent` | `ComputedRef<number>` | 水平偏移百分比 |
| `advance(fromPanel?)` | `Function` | 前进一步（仅最右侧面板或 step===0 时可触发） |
| `back()` | `Function` | 后退一步 |
| `reset()` | `Function` | 回到步骤 0 |

### 桌面端 vs 移动端

| 特性 | 桌面端（≥768px） | 移动端（<768px） |
|------|-----------------|-----------------|
| 同时可见面板 | 2 个 | 1 个 |
| 滑动模式 | (0,1)→(1,2)→… | 0→1→2→… |

### CSS 要求
```css
.viewport { overflow: hidden; }
.track { display: flex; height: 100%; transition: transform .35s; }
.track > * { flex: 1; min-width: 0; }
```
