# Valence 项目文档

## 项目概述

Valence 是一个开源药物管理与血药浓度追踪应用，基于药代动力学（Pharmacokinetics, PK）模型计算并可视化药物在体内的浓度变化曲线。支持多种房室模型（一室、二室、多室），涵盖 HRT（激素替代治疗）药物和 Journal 药物（精神活性物质等）。

**技术栈**：
- **前端**：Vue 3 + Vite + Chart.js + MDUI 2
- **计算引擎**：Rust → WebAssembly (wasm-pack)
- **数据持久化**：OPFS（Origin Private File System）+ IndexedDB + File System Access API

## 项目结构

```
The-Valence-Project/
├── index.html                  # 应用入口 HTML
├── package.json                # Node.js 依赖与脚本
├── vite.config.js              # Vite 构建配置
├── pnpm-lock.yaml              # 依赖锁定文件
├── .gitignore                  # Git 忽略规则
├── public/
│   ├── data/
│   │   ├── hrt_drugs.json      # HRT 药物参数配置
│   │   └── journal_drugs.json  # Journal 药物参数配置
│   ├── favicon.svg             # 网站图标
│   └── icons.svg               # 图标资源
├── src/
│   ├── main.js                 # 应用入口 JS
│   ├── App.vue                 # 根组件（导航布局）
│   ├── style.css               # 全局样式
│   ├── router/
│   │   └── index.js            # Vue Router 路由配置
│   ├── wasm/
│   │   ├── engineStore.js      # WASM 引擎封装与数据持久化
│   │   └── wasmIntegration.js  # WASM 模块加载器
│   └── pages/
│       ├── HomePage.vue        # 首页：图表与浓度曲线
│       ├── AddDosePage.vue     # 添加给药记录
│       ├── HistoryPage.vue     # 给药历史记录
│       ├── DrugLibraryPage.vue # 药物库管理
│       ├── CalibrationPage.vue # 模型校准
│       └── SettingsPage.vue    # 设置页面
├── wasm-core/
│   ├── Cargo.toml              # Rust 项目配置
│   └── src/
│       ├── lib.rs              # WASM 对外接口（ValenceEngine）
│       └── pk/
│           ├── mod.rs          # PK 模块入口与常量
│           ├── one_comp.rs     # 一室模型
│           ├── two_comp.rs     # 二室模型
│           └── ester.rs        # Depot 储库模型
├── .github/workflows/
│   └── deploy.yml              # GitHub Pages 自动部署
└── docs/                       # 项目文档
```

## 构建与运行

### 环境要求

- **Node.js** >= 18
- **pnpm**（或 npm）
- **Rust**（带 wasm32-unknown-unknown target）
- **wasm-pack**（`cargo install wasm-pack`）

### 1. 编译 WASM 核心

```bash
cd wasm-core
wasm-pack build --target web --out-dir ../public/wasm
```

此命令会：
1. 编译 Rust 源码为 `.wasm` 二进制
2. 生成 JS 胶水代码（`wasm_core.js`）
3. 生成 TypeScript 类型声明（`wasm_core.d.ts`）

产物输出到 `public/wasm/`：
- `wasm_core_bg.wasm` — WASM 二进制
- `wasm_core.js` — JS 胶水代码
- `wasm_core.d.ts` — TypeScript 类型声明
- `wasm_core_bg.wasm.d.ts` — WASM 二进制类型声明

### 2. 安装前端依赖

```bash
pnpm install
```

### 3. 启动开发服务器

```bash
pnpm dev
```

默认在 `http://localhost:5173` 启动 Vite 开发服务器。

### 4. 生产构建

```bash
pnpm build
```

产物输出到 `dist/` 目录。

### 5. 部署到 GitHub Pages

项目已配置 CI/CD（`.github/workflows/deploy.yml`）：

1. Push 到 GitHub 仓库的 main/master 分支
2. 仓库 Settings → Pages → Source 选择 **GitHub Actions**
3. 每次 push 自动构建部署

无需手动操作，GitHub Actions 会自动安装 Rust + Node.js，编译 WASM，构建前端，部署到 Pages。

## 架构概览

### 数据流

```
用户操作 → Vue 组件 → engineStore.js → ValenceEngine (WASM)
                 ↓                              ↓
           OPFS/IndexedDB                 PK 模型计算
                 ↓                              ↓
           持久化存储                    SimulationOutput
                                              ↓
                                        Chart.js 渲染
```

### 核心模块

1. **ValenceEngine**（Rust WASM）：药物注册、剂量管理、仿真计算的统一入口
2. **PK 模块**（Rust）：一室/二室/多室模型的数学实现
3. **engineStore**（JS）：封装 WASM 调用，管理 OPFS/IndexedDB 持久化
4. **Vue 页面**：用户交互界面

详细文档请参阅各模块章节。
