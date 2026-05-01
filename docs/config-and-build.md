# 配置文件与构建系统

## package.json

**文件位置**：`package.json`

### 基本信息
- 名称：`the-valence-project`
- 版本：`0.0.0`
- 类型：`module`（ES 模块）

### 脚本

| 命令 | 说明 |
|------|------|
| `pnpm dev` | 启动 Vite 开发服务器 |
| `pnpm build` | 生产构建 |
| `pnpm preview` | 预览生产构建 |
| `pnpm wasm` | 编译 Rust WASM（需先安装 wasm-pack） |

---

## GitHub Pages 部署

项目通过 GitHub Actions 自动构建部署。配置位于 `.github/workflows/deploy.yml`。

### 手动部署步骤

1. Fork/Push 仓库到 GitHub
2. 在仓库 Settings → Pages → Source 选择 **GitHub Actions**
3. 每次 push 到 main/master 分支时将自动构建部署

### 部署原理

| 配置项 | 说明 |
|------|------|
| `vite.config.js` → `base: './'` | 所有资源使用相对路径，适配 `user.github.io/repo/` 子目录 |
| `router/index.js` → `createWebHashHistory()` | Hash 路由（`#/page`），不需要服务端路由支持 |
| `engineStore.js` → `assetUrl()` | 使用 `new URL(path, document.baseURI)` 动态解析 WASM/数据文件路径 |

### WASM 构建命令

```bash
cd wasm-core
wasm-pack build --target web --out-dir ../public/wasm
```

### 前端依赖

| 包名 | 版本 | 用途 |
|------|------|------|
| `vue` | ^3.5.32 | 前端框架 |
| `vue-router` | ^4.5.0 | 路由 |
| `chart.js` | ^4.4.0 | 图表库 |
| `chartjs-adapter-date-fns` | ^3.0.0 | Chart.js 日期适配器 |
| `date-fns` | ^4.1.0 | 日期工具库 |
| `mdui` | ^2.1.4 | Material Design 3 UI 组件库 |
| `@fontsource/material-icons` | ^5.2.7 | Material Icons 字体 |
| `@fontsource/material-icons-outlined` | ^5.2.6 | Material Icons Outlined 字体 |

### 开发依赖

| 包名 | 版本 | 用途 |
|------|------|------|
| `vite` | ^8.0.10 | 构建工具 |
| `@vitejs/plugin-vue` | ^6.0.6 | Vite Vue 插件 |

---

## vite.config.js

**文件位置**：`vite.config.js`

### 插件
- `@vitejs/plugin-vue` — Vue SFC 编译

### 路径别名
- `@` → `src/`

### 优化配置
- `optimizeDeps.exclude: []` — 不排除任何预构建依赖

### 开发服务器
- `fs.allow: ['..']` — 允许访问上级目录文件

### 构建配置
- `target: 'esnext'` — 使用最新 ES 标准（WASM 需要）

### Worker 配置
- `format: 'es'` — Worker 使用 ES 模块格式

---

## Cargo.toml（Rust WASM 项目）

**文件位置**：`wasm-core/Cargo.toml`

### 基本信息
- 名称：`wasm-core`
- Rust Edition：2024
- 库类型：`cdylib`（动态库，供 WASM 使用）+ `rlib`（Rust 库）

### 依赖

| crate | 版本 | 用途 |
|-------|------|------|
| `wasm-bindgen` | 0.2 | Rust ↔ JS 互操作 |
| `js-sys` | 0.3 | JavaScript 全局对象绑定 |
| `serde` | 1.0（derive） | 序列化/反序列化 |
| `serde-wasm-bindgen` | 0.6 | wasm-bindgen 与 serde 桥接 |
| `serde_json` | 1.0 | JSON 处理 |

---

## index.html

**文件位置**：`index.html`

### 关键配置
- `lang="zh-CN"` — 中文语言
- `viewport: width=device-width, maximum-scale=1.0, user-scalable=no` — 移动端视口
- 入口脚本：`<script type="module" src="/src/main.js">`

---

## main.js — 应用入口

**文件位置**：`src/main.js`

```javascript
import { createApp } from 'vue'
import 'mdui/mdui.css'
import 'mdui'
import '@fontsource/material-icons-outlined/400.css'
import '@fontsource/material-icons/400.css'
import './style.css'
import App from './App.vue'
import router from './router'

createApp(App).use(router).mount('#app')
```

### 初始化顺序
1. 导入 MDUI CSS
2. 导入 MDUI JS（注册 Web Components）
3. 导入 Material Icons 字体
4. 导入全局样式
5. 创建 Vue 应用 → 挂载路由 → 挂载到 `#app`

---

## 药物配置 JSON 文件

### hrt_drugs.json

**文件位置**：`public/data/hrt_drugs.json`

存储 HRT（激素替代治疗）相关的预置药物参数。

### journal_drugs.json

**文件位置**：`public/data/journal_drugs.json`

存储 Journal（精神活性物质等）相关的预置药物参数。

### 药物 JSON 结构（统一格式）

```json
{
  "drug_id": "hrt_ev",
  "name": "Estradiol Valerate",
  "model_type": "multi_compartment",
  "depot_model": true,
  "group_id": "E2",
  "molecular_weight": 356.50,
  "display_unit": "pg/mL",
  "dose_unit": "mg",
  "routes": [
    { "route": "injection", "unit": "mg" },
    { "route": "oral", "unit": "mg" }
  ],
  "parameters": {
    "half_life": -1,
    "volume_of_distribution": 2.0,
    "equivalence_factor": 0.764,
    "ka": 0,
    "bioavailability": 0,
    "frac_fast": 0.40,
    "k1_fast": 0.0216,
    "k1_slow": 0.0138,
    "hydrolysis_k2": 0.070,
    "formation_frac": 0.0623,
    "k_clear": 0.041,
    "oral_ka": 0.05,
    "oral_bioavailability": 0.03
  }
}
```

**核心字段（所有药物共有）**：

| 字段 | 类型 | 说明 |
|------|------|------|
| `drug_id` | `string` | 唯一标识，HRT 以 `hrt_` 开头，Journal 以 `journal_` 开头 |
| `name` | `string` | 药物显示名称 |
| `model_type` | `string` | `one_compartment` / `two_compartment` / `multi_compartment` |
| `depot_model` | `bool` | 非 depot 药物必须显式设为 `false` |
| `group_id` | `string` | 合并曲线标识，空字符串 = 独立曲线 |
| `molecular_weight` | `number` | 分子量 (g/mol) |
| `display_unit` | `string` | 前端浓度显示单位（如 `"pg/mL"`） |
| `dose_unit` | `string` | 剂量单位 |
| `routes` | `array` | 支持的给药方式列表 |

**parameters 常见键（模型特有，按 model_type 选用）**：

| 键 | 适用模型 | 说明 |
|------|---------|------|
| `half_life` | 全部 | 半衰期 (h)，depot 设为 -1 |
| `volume_of_distribution` | 全部 | 分布容积 (L/kg) |
| `ka` | one/two_comp | 吸收速率常数 (1/h) |
| `bioavailability` | one/two_comp | 生物利用度 F |
| `k_clear` | 全部 | 清除速率常数 |
| `k12`, `k21` | two_comp | 二室速率常数 |
| `equivalence_factor` | multi/depot | 等效转化因子 |
| `frac_fast` | depot | 快速释放比例 |
| `k1_fast`, `k1_slow` | depot | 快速/慢速释放速率 |
| `hydrolysis_k2` | depot | 酯键水解速率 |
| `formation_frac` | depot | 代谢形成分数 |
| `oral_ka` | depot | 口服吸收速率 |
| `oral_bioavailability` | depot | 口服生物利用度 |

---

## .gitignore

**文件位置**：`.gitignore`

### 忽略内容

| 分类 | 规则 | 说明 |
|------|------|------|
| 日志 | `logs`, `*.log`, `pnpm-debug.log*` | 调试日志 |
| 构建产物 | `node_modules`, `dist`, `dist-ssr` | 依赖和前端构建输出 |
| WASM 产物 | `wasm-core/target`, `wasm-core/pkg`, `public/wasm_core.*` | Rust 编译中间产物 |
| Rust 锁文件 | `wasm-core/Cargo.lock` | 库项目不提交 lock 文件 |
| 参考资料 | `reference` | 外部参考项目 |
| 编辑器 | `.vscode/*`, `.idea`, `.DS_Store` | IDE 配置（保留 extensions.json） |
