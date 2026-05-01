# Rust WASM 核心 — lib.rs

## 文件位置
`wasm-core/src/lib.rs`

## 概述
定义所有与前端交互的数据结构和 `ValenceEngine` 核心类。所有药代动力学（PK）参数统一放在 `parameters: HashMap<String, f64>` 中，实现纯数据驱动。

---

## 数据结构

### RouteInfo
给药途径信息。

| 字段 | 类型 | 说明 |
|------|------|------|
| `route` | `String` | 给药方式标识（如 "oral"、"injection"） |
| `unit` | `String` | 该给药方式的剂量单位（如 "mg"、"µg"） |

### DrugRecord
药物注册记录——仅保留标识和类型字段，PK 参数全部在 `parameters` 中。

| 字段 | 类型 | 说明 |
|------|------|------|
| `drug_id` | `String` | 药物唯一标识 |
| `name` | `String` | 药物名称 |
| `model_type` | `String` | 房室模型类型：`"one_compartment"` / `"two_compartment"` / `"multi_compartment"` |
| `group_id` | `String` | 合并曲线标识，空字符串 = 独立曲线 |
| `dose_unit` | `String` | 剂量单位 |
| `routes` | `Vec<RouteInfo>` | 可用给药途径（`#[wasm_bindgen(skip)]`） |
| `display_unit` | `String` | 前端显示浓度单位（如 "pg/mL"） |
| `molecular_weight` | `f64` | 分子量 (g/mol)，用于自动计算 molar_factor |
| `depot_model` | `bool` | 是否使用储库注射/口服双路径模型 |
| `parameters` | `HashMap<String, f64>` | 所有 PK 参数（`#[wasm_bindgen(skip)]`+`params` getter 暴露给 JS） |

JavaScript 通过 `drug.params` getter 获取 parameters（转换为 JS Object）。

### DoseRecord / SimulationOutput
不变。

### ValenceEngine 内部状态

```rust
pub struct ValenceEngine {
    drugs: HashMap<String, DrugRecord>,
    doses: Vec<DoseRecord>,
    weight_kg: f64,
}
```

简洁——无硬编码模型实例。

---

## 核心方法

### `compute_molar_factor(drug) → f64`
计算酯类到母体化合物的摩尔转化系数。

优先级：
1. `parameters["equivalence_factor"] > 0` → 直接使用
2. 遍历已注册药物，查找同 `group_id` 且 `depot_model=false` 的父药物 → `parent.mw / drug.mw`
3. 回退 → 1.0

### `resolve_group_vd(items) → f64`
获取该 group 的分布容积。优先取同组非 depot 父药物的 `parameters["volume_of_distribution"]`。

### `resolve_group_id(drug) → String`
`group_id` 非空则用 `group_id`，否则用 `drug_id`（独立分组）。

---

## 仿真流程

### `runSimulation()`
1. 收集所有 `DoseWithDrug`
2. 按 `resolve_group_id()` 分组到 `HashMap<String, Vec<DoseWithDrug>>`
3. 对每组调用 `simulate_group()`

### `simulate_group(items)`
1. 取代表药物的 `group_name` 和 `display_unit`
2. 判断是否有口服/注射，选择步长
3. 计算 Vd：`resolve_group_vd() * weight * 1000`
4. 逐采样点累加 `route_amount()`

### `route_amount(drug, tau, dose_mg, route, molar_factor) → f64`
模型分发（纯数据驱动）：

```
depot_model == true:
  ├── route == "injection" → depot_injection_amount(params, molar_factor)
  └── route != "injection" → depot_oral_amount(params, molar_factor)

depot_model == false:
  ├── "one_compartment" | "multi_compartment" → OneCompartment::from_params(params)
  │   amount = model.amount(tau, dose_mg * equivalence_factor)
  ├── "two_compartment" → TwoCompartment::from_params(params)
  │   amount = model.amount(tau, dose_mg)
  └── _ → 0
```
