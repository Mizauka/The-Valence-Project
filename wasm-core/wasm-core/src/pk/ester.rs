use std::collections::HashMap;

fn param(map: &HashMap<String, f64>, key: &str, default: f64) -> f64 {
    map.get(key).copied().unwrap_or(default)
}

fn one_comp_amount(tau: f64, dose_mg: f64, ka: f64, ke: f64, f: f64) -> f64 {
    if tau < 0.0 || dose_mg <= 0.0 {
        return 0.0;
    }
    if (ka - ke).abs() < 1e-9 {
        return dose_mg * f * ka * tau * (-ke * tau).exp();
    }
    dose_mg * f * ka / (ka - ke) * ((-ke * tau).exp() - (-ka * tau).exp())
}

fn analytic_3c(tau: f64, dose_mg: f64, f: f64, k1: f64, k2: f64, k3: f64) -> f64 {
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

pub fn depot_injection_amount(
    tau: f64,
    dose_mg: f64,
    parameters: &HashMap<String, f64>,
    molar_factor: f64,
) -> f64 {
    let frac_fast = param(parameters, "frac_fast", 0.5);
    let k1_fast = param(parameters, "k1_fast", 0.1);
    let k1_slow = param(parameters, "k1_slow", 0.01);
    let k2 = param(parameters, "hydrolysis_k2", 0.0);
    let formation_frac = param(parameters, "formation_frac", 1.0);
    let k3 = param(parameters, "depot_clearance", 0.041);

    let f = formation_frac * molar_factor;

    let dose_fast = dose_mg * frac_fast;
    let dose_slow = dose_mg * (1.0 - frac_fast);

    analytic_3c(tau, dose_fast, f, k1_fast, k2, k3)
        + analytic_3c(tau, dose_slow, f, k1_slow, k2, k3)
}

pub fn depot_oral_amount(
    tau: f64,
    dose_mg: f64,
    parameters: &HashMap<String, f64>,
    molar_factor: f64,
) -> f64 {
    let ka = param(parameters, "oral_ka", 0.32);
    let ke = param(parameters, "oral_ke", 0.41);
    let oral_bio = param(parameters, "oral_bioavailability", 0.03);
    let f = oral_bio * molar_factor;
    one_comp_amount(tau, dose_mg, ka, ke, f)
}
