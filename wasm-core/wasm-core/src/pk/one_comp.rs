use std::collections::HashMap;

fn param(map: &HashMap<String, f64>, key: &str, default: f64) -> f64 {
    map.get(key).copied().unwrap_or(default)
}

pub struct OneCompartment {
    pub ka: f64,
    pub ke: f64,
    pub f: f64,
}

impl OneCompartment {
    pub fn from_params(params: &HashMap<String, f64>) -> Self {
        let half_life = param(params, "half_life", 0.0);
        let ka = param(params, "ka", 0.32);
        let bioavailability = param(params, "bioavailability", 1.0);
        let ke = if half_life > 0.0 { 0.693 / half_life } else { param(params, "k_clear", 0.41) };
        OneCompartment {
            ka,
            ke,
            f: bioavailability,
        }
    }

    pub fn amount(&self, tau: f64, dose_mg: f64) -> f64 {
        if tau < 0.0 || dose_mg <= 0.0 {
            return 0.0;
        }
        let ka = self.ka;
        let ke = self.ke;
        let f = self.f;

        if (ka - ke).abs() < 1e-9 {
            return dose_mg * f * ka * tau * (-ke * tau).exp();
        }
        dose_mg * f * ka / (ka - ke) * ((-ke * tau).exp() - (-ka * tau).exp())
    }
}
