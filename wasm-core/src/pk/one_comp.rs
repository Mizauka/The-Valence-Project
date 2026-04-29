use crate::pk::KCLEAR;

pub struct OneCompartment {
    pub ka: f64,
    pub ke: f64,
    pub f: f64,
    pub vd_per_kg: f64,
}

impl OneCompartment {
    pub fn new(ka: f64, ke: f64, f: f64, vd_per_kg: f64) -> Self {
        OneCompartment { ka, ke, f, vd_per_kg }
    }

    pub fn from_params(half_life: f64, vd_per_kg: f64, ka: f64, bioavailability: f64) -> Self {
        let ke = if half_life > 0.0 { 0.693 / half_life } else { KCLEAR };
        OneCompartment {
            ka: if ka > 0.0 { ka } else { 0.32 },
            ke,
            f: if bioavailability > 0.0 { bioavailability } else { 1.0 },
            vd_per_kg: if vd_per_kg > 0.0 { vd_per_kg } else { 2.0 },
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

    pub fn concentration(&self, tau: f64, dose_mg: f64, body_weight_kg: f64) -> f64 {
        let vd_ml = self.vd_per_kg * body_weight_kg * 1000.0;
        if vd_ml <= 0.0 {
            return 0.0;
        }
        self.amount(tau, dose_mg) / vd_ml
    }

    #[allow(dead_code)]
    pub fn vd_per_kg(&self) -> f64 {
        self.vd_per_kg
    }
}
