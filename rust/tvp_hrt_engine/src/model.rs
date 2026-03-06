use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum Route {
    #[serde(rename = "injection")]
    Injection,
    #[serde(rename = "patchApply")]
    PatchApply,
    #[serde(rename = "patchRemove")]
    PatchRemove,
    #[serde(rename = "gel")]
    Gel,
    #[serde(rename = "oral")]
    Oral,
    #[serde(rename = "sublingual")]
    Sublingual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum Ester {
    #[serde(rename = "E2")]
    E2,
    #[serde(rename = "EB")]
    EB,
    #[serde(rename = "EV")]
    EV,
    #[serde(rename = "EC")]
    EC,
    #[serde(rename = "EN")]
    EN,
    #[serde(rename = "CPA")]
    CPA,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum ExtraKey {
    #[serde(rename = "concentrationMGmL")]
    ConcentrationMGmL,
    #[serde(rename = "areaCM2")]
    AreaCM2,
    #[serde(rename = "releaseRateUGPerDay")]
    ReleaseRateUgPerDay,
    #[serde(rename = "sublingualTheta")]
    SublingualTheta,
    #[serde(rename = "sublingualTier")]
    SublingualTier,
    #[serde(rename = "gelSite")]
    GelSite,

    // --- Injection overrides (config-driven; defaults match Oyama logic.ts) ---
    #[serde(rename = "injectionFracFast")]
    InjectionFracFast,
    #[serde(rename = "injectionK1FastPerHour")]
    InjectionK1FastPerHour,
    #[serde(rename = "injectionK1SlowPerHour")]
    InjectionK1SlowPerHour,
    #[serde(rename = "injectionK2PerHour")]
    InjectionK2PerHour,
    #[serde(rename = "injectionK3PerHour")]
    InjectionK3PerHour,
    #[serde(rename = "injectionFormationFraction")]
    InjectionFormationFraction,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DoseEvent {
    pub id: String,
    pub route: Route,
    #[serde(rename = "timeH")]
    pub time_h: f64,
    #[serde(rename = "doseMG")]
    pub dose_mg: f64,
    pub ester: Ester,
    #[serde(default)]
    pub extras: HashMap<ExtraKey, f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SimulationResult {
    #[serde(rename = "timeH")]
    pub time_h: Vec<f64>,
    #[serde(rename = "concPGmL")]
    pub conc_pg_ml: Vec<f64>,
    #[serde(rename = "concPGmL_E2")]
    pub conc_pg_ml_e2: Vec<f64>,
    #[serde(rename = "concPGmL_CPA")]
    pub conc_ng_ml_cpa: Vec<f64>,
    pub auc: f64,
}
