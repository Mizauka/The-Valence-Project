use crate::pk::{KCLEAR, KCLEAR_INJECTION};
use std::collections::HashMap;

struct DepotParams {
    frac_fast: HashMap<&'static str, f64>,
    k1_fast: HashMap<&'static str, f64>,
    k1_slow: HashMap<&'static str, f64>,
}

impl DepotParams {
    fn new() -> Self {
        let mut frac_fast = HashMap::new();
        frac_fast.insert("EB", 0.90);
        frac_fast.insert("EV", 0.40);
        frac_fast.insert("EC", 0.229164549);
        frac_fast.insert("EN", 0.05);
        frac_fast.insert("E2", 1.0);

        let mut k1_fast = HashMap::new();
        k1_fast.insert("EB", 0.144);
        k1_fast.insert("EV", 0.0216);
        k1_fast.insert("EC", 0.005035046);
        k1_fast.insert("EN", 0.0010);
        k1_fast.insert("E2", 0.5);

        let mut k1_slow = HashMap::new();
        k1_slow.insert("EB", 0.114);
        k1_slow.insert("EV", 0.0138);
        k1_slow.insert("EC", 0.004510574);
        k1_slow.insert("EN", 0.0050);
        k1_slow.insert("E2", 0.0);

        DepotParams { frac_fast, k1_fast, k1_slow }
    }

    fn get(&self, key: &str) -> (f64, f64, f64) {
        (
            *self.frac_fast.get(key).unwrap_or(&0.5),
            *self.k1_fast.get(key).unwrap_or(&0.1),
            *self.k1_slow.get(key).unwrap_or(&0.01),
        )
    }
}

struct FormationFraction {
    map: HashMap<&'static str, f64>,
}

impl FormationFraction {
    fn new() -> Self {
        let mut map = HashMap::new();
        map.insert("EB", 0.1092);
        map.insert("EV", 0.0623);
        map.insert("EC", 0.1173);
        map.insert("EN", 0.12);
        map.insert("E2", 1.0);
        FormationFraction { map }
    }

    fn get(&self, key: &str) -> f64 {
        *self.map.get(key).unwrap_or(&0.08)
    }
}

struct HydrolysisK2 {
    map: HashMap<&'static str, f64>,
}

impl HydrolysisK2 {
    fn new() -> Self {
        let mut map = HashMap::new();
        map.insert("EB", 0.090);
        map.insert("EV", 0.070);
        map.insert("EC", 0.045);
        map.insert("EN", 0.015);
        map.insert("E2", 0.0);
        HydrolysisK2 { map }
    }

    fn get(&self, key: &str) -> f64 {
        *self.map.get(key).unwrap_or(&0.0)
    }
}

struct OralEsterParams {
    k_abs_ev: f64,
    k_abs_e2: f64,
    bioavailability: f64,
}

impl OralEsterParams {
    fn new() -> Self {
        OralEsterParams {
            k_abs_ev: 0.05,
            k_abs_e2: 0.32,
            bioavailability: 0.03,
        }
    }
}

pub struct EsterMultiCompartment {
    depot: DepotParams,
    formation: FormationFraction,
    hydrolysis_k2: HydrolysisK2,
    oral: OralEsterParams,
}

impl EsterMultiCompartment {
    pub fn new() -> Self {
        EsterMultiCompartment {
            depot: DepotParams::new(),
            formation: FormationFraction::new(),
            hydrolysis_k2: HydrolysisK2::new(),
            oral: OralEsterParams::new(),
        }
    }

    fn analytic_3c(&self, tau: f64, dose_mg: f64, f: f64, k1: f64, k2: f64, k3: f64) -> f64 {
        if k1 <= 0.0 || dose_mg <= 0.0 || tau < 0.0 {
            return 0.0;
        }
        let k1_k2 = k1 - k2;
        let k1_k3 = k1 - k3;
        let k2_k3 = k2 - k3;

        if k1_k2.abs() < 1e-9 || k1_k3.abs() < 1e-9 || k2_k3.abs() < 1e-9 {
            return 0.0;
        }

        let term1 = (-k1 * tau).exp() / (k1_k2 * k1_k3);
        let term2 = (-k2 * tau).exp() / (-k1_k2 * k2_k3);
        let term3 = (-k3 * tau).exp() / (k1_k3 * k2_k3);

        dose_mg * f * k1 * k2 * (term1 + term2 + term3)
    }

    fn one_comp(&self, tau: f64, dose_mg: f64, ka: f64, ke: f64, f: f64) -> f64 {
        if tau < 0.0 || dose_mg <= 0.0 {
            return 0.0;
        }
        if (ka - ke).abs() < 1e-9 {
            return dose_mg * f * ka * tau * (-ke * tau).exp();
        }
        dose_mg * f * ka / (ka - ke) * ((-ke * tau).exp() - (-ka * tau).exp())
    }

    pub fn injection_concentration(&self, tau: f64, dose_mg: f64, ester_key: &str, to_e2_factor: f64) -> f64 {
        let (frac_fast, k1_fast, k1_slow) = self.depot.get(ester_key);
        let k2 = self.hydrolysis_k2.get(ester_key);
        let formation_frac = self.formation.get(ester_key);
        let f = formation_frac * to_e2_factor;
        let k3 = KCLEAR_INJECTION;

        let dose_fast = dose_mg * frac_fast;
        let dose_slow = dose_mg * (1.0 - frac_fast);

        self.analytic_3c(tau, dose_fast, f, k1_fast, k2, k3)
            + self.analytic_3c(tau, dose_slow, f, k1_slow, k2, k3)
    }

    pub fn oral_concentration(&self, tau: f64, dose_mg: f64, ester_key: &str, to_e2_factor: f64) -> f64 {
        let ka = if ester_key == "EV" { self.oral.k_abs_ev } else { self.oral.k_abs_e2 };
        let ke = KCLEAR;
        let f = self.oral.bioavailability * to_e2_factor;
        self.one_comp(tau, dose_mg, ka, ke, f)
    }
}
