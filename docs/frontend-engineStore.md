# 前端引擎封装 — engineStore.js

## 文件位置
`src/wasm/engineStore.js`

## 概述
`engineStore` 是前端与 Rust WASM 引擎之间的桥接层，负责：
1. WASM 模块的初始化与缓存
2. OPFS/IndexedDB/LocalStorage 数据持久化
3. 药物元数据缓存（给药方式、剂量单位）
4. 外部目录同步（File System Access API）
5. 剂量单位转换（存储 mg ↔ 显示 µg/ng/pg/mL）

---

## 模块级变量

| 变量 | 类型 | 说明 |
|------|------|------|
| `_engine` | `ValenceEngine \| null` | WASM 引擎单例 |
| `_initPromise` | `Promise \| null` | 初始化 Promise（防重入） |
| `_opfsRoot` | `FileSystemDirectoryHandle \| null` | OPFS 根目录句柄 |
| `_externalDirHandle` | `FileSystemDirectoryHandle \| null` | 用户选择的外部目录句柄 |
| `_drugMetaCache` | `Object` | 药物元数据缓存（dose_unit, routes） |

---

## IndexedDB 辅助函数

用于持久化 FileSystemDirectoryHandle（浏览器关闭后恢复）。

| 函数 | 签名 | 说明 |
|------|------|------|
| `openIDB()` | `() → Promise<IDBDatabase>` | 打开/创建 `valence_handles` 数据库 |
| `idbPut(key, value)` | `(string, any) → Promise` | 存储句柄 |
| `idbGet(key)` | `(string) → Promise<any>` | 读取句柄 |

**数据库结构**：
- 数据库名：`valence_handles`
- 对象存储：`handles`
- 键：`"externalDir"`

---

## OPFS 文件操作

### `getOPFS() → Promise<FileSystemDirectoryHandle | null>`
获取 OPFS 根目录下的 `valence/` 子目录（自动创建）。

### `opfsWrite(fileName, content)`
写入文件到 OPFS。失败时回退到 `localStorage`。

### `opfsRead(fileName) → Promise<string | null>`
从 OPFS 读取文件。失败时回退到 `localStorage`。

### 回退机制
当 OPFS 不可用时（如部分浏览器不支持），降级使用 `localStorage`：
- `writeFallback(key, value)` — `localStorage.setItem('valence_' + key, value)`
- `readFallback(key)` — `localStorage.getItem('valence_' + key)`

---

## 核心函数

### `getEngine() → Promise<ValenceEngine>`
获取 WASM 引擎单例。首次调用触发初始化流程。

### `_init()` — 初始化流程

```
1. 获取 WASM JS 胶水代码（带 cache-bust 时间戳）
2. 动态 import Blob URL
3. 调用 wasm-bindgen default 初始化
4. 实例化 ValenceEngine
5. 从 IndexedDB 恢复外部目录句柄
6. 加载 OPFS 中的体重设置
7. 加载预置药物（hrt_drugs.json + journal_drugs.json）
8. 尝试从外部目录加载数据
9. 回退从 OPFS 加载自定义药物和剂量
10. 保存到 OPFS（同步外部数据）
```

### `jsonDrugToRecord(d) → DrugRecord`
将 JSON 药物配置转换为 Rust 期望的 DrugRecord 格式。
- 提取 `parameters` 嵌套字段到顶层
- 构建 `routes` 数组
- 设置默认值（bioavailability = 1, vd = 1）

### `resolveDrugId(engine, ev) → string | null`
从剂量事件中解析药物 ID：
1. 优先从 `extras.drug_id` 读取
2. 回退按 `ester`（药物名）在已注册药物中查找
3. 先按 `drug_id` 匹配，再按 `name` 匹配

---

## 持久化函数

### `saveAll(engine)`
保存所有数据到 OPFS：
- `custom_drugs.json` — 自定义药物（排除 hrt_/journal_ 前缀）
- `doses.json` — 剂量记录（含 weight 和 events 数组）
- `weight` — 体重值

如果有外部目录句柄，同步调用 `syncToExternalDir()`。

### `syncToExternalDir(engine, customDrugs, payload)`
将数据同步写入外部目录的 `data/` 子目录：
- `data/custom_drugs.json`
- `data/doses.json`

### `pickDataDirectory() → Promise<string>`
触发浏览器目录选择对话框，将选中目录的句柄存入 IndexedDB，加载其中的数据，然后同步回 OPFS。

---

## 数据加载函数

### `loadPresetDrugs(engine)`
从 `public/data/` 加载 HRT 和 Journal 药物配置，缓存元数据到 `_drugMetaCache`。

### `loadCustomDrugsFromOPFS(engine)`
从 OPFS 加载自定义药物。

### `loadDosesFromOPFS(engine)`
从 OPFS 加载剂量记录的 events 数组。

### `loadFromExternalDir(engine) → Promise<boolean>`
从外部目录加载 `doses.json` 和 `custom_drugs.json`。

---

## 导出函数（供 Vue 页面调用）

| 函数 | 说明 |
|------|------|
| `getAllDrugsWithSource()` | 获取所有药物，附加 `source`（hrt/journal/custom）和 `dose_unit`/`routes` 元数据 |
| `getCustomDrugs()` | 获取自定义药物 |
| `getPresetDrugs(source)` | 按来源获取预置药物 |
| `addDrug(drugData)` | 添加药物并保存 |
| `deleteDrug(drugId)` | 删除药物并保存 |
| `addDose(doseData)` | 添加剂量并保存 |
| `removeDose(doseId)` | 删除剂量并保存 |
| `getAllDoses()` | 获取所有剂量（含显示单位转换） |
| `exportAllData()` | 导出完整数据 JSON |
| `getWeight()` / `setWeight(kg)` | 体重读写 |
| `getExternalDirName()` | 获取外部目录名称 |
| `checkExternalDirPermission()` | 检查外部目录权限 |
| `requestExternalDirPermission()` | 请求重新授权 |

---

## 单位转换逻辑

### 存储层
所有剂量统一以 **mg** 存储在 Rust/WASM 中。

### 显示层（`getAllDoses()`）
根据药物元数据中的 `dose_unit` 转换为显示单位：
- `µg` → `amount × 1000`
- `ng` → `amount × 1,000,000`
- `pg` → `amount × 1,000,000,000`
- `mg` → 不变
- `mL` → 不变

### 输入层（AddDosePage）
用户输入反向转换回 mg：
- `µg` → `amount / 1000`
- `ng` → `amount / 1,000,000`
- 以此类推
