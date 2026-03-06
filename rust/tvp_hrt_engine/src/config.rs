use serde::{Deserialize, Serialize};

// 注意：本 crate 的 HRT 三层配置（L1/L2/L3）后续会落地到 assets/database/hrt/...
// 目前先定义最小 schema 以支撑 CLI/回归测试。

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Range {
    pub min: f64,
    pub max: f64,
    pub units: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HrtL1Compound {
    pub schema_version: u32,
    pub id: String,
    pub name: String,

    /// 仅用于单位换算：mg * 1e9 / Vd_ml => pg/mL
    pub distribution_volume_l_per_kg: Option<f64>,

    /// Oyama: E2 vs CPA 使用不同 Vd/kg
    pub notes: Option<String>,

    #[serde(default)]
    pub legacy: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HrtL2Route {
    pub schema_version: u32,
    pub id: String,
    pub name: String,

    #[serde(default)]
    pub legacy: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum HrtModel {
    /// Oyama: oneCompAmount(t) = Dose*F*k1/(k1-k3)*(e^{-k3 t}-e^{-k1 t}); ka≈ke 时为 Dose*F*k1*t*e^{-k3 t}
    OneCompFirstOrder {
        k1_per_hour: f64,
        k3_per_hour: f64,
        f: f64,
    },

    /// Oyama: _analytic3C(t; dose, F, k1, k2, k3)
    ThreeCompAnalytical {
        k1_per_hour: f64,
        k2_per_hour: f64,
        k3_per_hour: f64,
        f: f64,
    },

    /// Oyama: injection 双库并联（快/慢）三室解析解
    InjectionTwoDepot3C {
        frac_fast: f64,
        fast: Box<HrtModel>,
        slow: Box<HrtModel>,
    },

    /// Oyama: patch 零阶输入（佩戴期 rate_mg_per_hour 输入），移除后按 k3 衰减。
    ZeroOrderPatch {
        rate_mg_per_hour: f64,
        k3_per_hour: f64,
        wear_hours: f64,
    },

    /// Oyama: sublingual 双通路（快：黏膜，慢：吞咽→口服），两支路可分别选 OneComp/ThreeComp
    DualBranch {
        frac_fast: f64,
        fast: Box<HrtModel>,
        slow: Box<HrtModel>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HrtL3DoseModel {
    pub schema_version: u32,

    pub compound_id: String,
    pub compound_name: String,
    pub route_id: String,
    pub route_name: String,

    pub dose_units: String,

    /// 该 compound 在该 route 下的 PK 模型（严格跟随 Oyama 的逻辑形状）
    pub model: HrtModel,

    /// 可选：典型持续时间，供 CLI 验证/展示
    pub total: Option<Range>,

    #[serde(default)]
    pub legacy: serde_json::Value,
}
