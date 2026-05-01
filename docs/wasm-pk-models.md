# Rust PK 模块 — 药代动力学模型

## 文件位置
`wasm-core/src/pk/`

## 模块结构

```
pk/
├── mod.rs       # 模块入口
├── one_comp.rs  # 一室模型
├── two_comp.rs  # 二室模型
└── ester.rs     # Depot 储库注射/口服函数
```

> **已删除**：`e2.rs`（硬编码 E2）、`cpa.rs`（硬编码 CPA）——所有参数移入 JSON `parameters`。

---

## one_comp.rs — 一室模型

```rust
pub struct OneCompartment {
    pub ka: f64,
    pub ke: f64,
    pub f: f64,
}
```

### `from_params(params: &HashMap<String, f64>)`
从 parameters 字典读取：
- `half_life`：半衰期 (h)，> 0 则 `ke = 0.693 / half_life`
- `ka`：吸收速率常数（默认 0.32）
- `bioavailability`：生物利用度 F（默认 1.0）
- `k_clear`：当 `half_life <= 0` 时的回退清除速率（默认 0.41）

### `amount(tau, dose_mg) → f64`
一室口服模型标准公式。

---

## two_comp.rs — 二室模型

### `from_params(params: &HashMap<String, f64>)`
- `half_life` → `beta = 0.693 / half_life`（默认 0.01579）
- `bioavailability` → F（默认 1.0）
- `ka`（默认 0.6）
- `k12`（默认 0.0）
- `k21`（默认 0.04）
- `alpha = ke + k12 + k21`

### `amount(tau, dose_mg) → f64`
二室口服模型三指数解析解。

---

## ester.rs — Depot 储库模型

纯函数，所有参数从 `parameters` HashMap 读取。

### `depot_injection_amount(tau, dose_mg, parameters, molar_factor) → f64`

储库注射三室连串模型 (depot → ester → parent → cleared)：

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `frac_fast` | 0.5 | 快速释放比例 |
| `k1_fast` | 0.1 | 快速释放速率 |
| `k1_slow` | 0.01 | 缓慢释放速率 |
| `hydrolysis_k2` | 0.0 | 酯键水解速率 |
| `formation_frac` | 1.0 | 代谢形成分数 |
| `k_clear` | 0.041 | 清除速率 |

`F = formation_frac × molar_factor`
`amount = analytic_3c(dose_fast, F, k1_fast, k2, k3) + analytic_3c(dose_slow, F, k1_slow, k2, k3)`

### `depot_oral_amount(tau, dose_mg, parameters, molar_factor) → f64`

Depot 类药物的口服路径：

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `oral_ka` | 0.32 | 口服吸收速率 |
| `k_clear` | 0.41 | 清除速率 |
| `oral_bioavailability` | 0.03 | 口服生物利用度 |

`F = oral_bioavailability × molar_factor`
使用一室口服模型。

---

## half_life = -1 说明

Depot 药物（depot_model=true）的 `parameters.half_life = -1`，因为 depot 模型完全由 `frac_fast/k1_fast/k1_slow/k2/k_clear` 驱动，不进入 `OneCompartment::from_params()`。`-1` 是明确的语义标记（区别于 `0` 的歧义）。
