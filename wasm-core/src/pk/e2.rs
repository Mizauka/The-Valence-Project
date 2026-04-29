use crate::pk::one_comp::OneCompartment;
use crate::pk::KCLEAR;

pub struct E2OneCompartment {
    model: OneCompartment,
}

impl E2OneCompartment {
    pub fn new() -> Self {
        E2OneCompartment {
            model: OneCompartment::new(0.32, KCLEAR, 0.03, 2.0),
        }
    }

    pub fn concentration(&self, tau: f64, dose_mg: f64, to_e2_factor: f64) -> f64 {
        let f = self.model.f * to_e2_factor;
        if tau < 0.0 || dose_mg <= 0.0 {
            return 0.0;
        }
        let ka = self.model.ka;
        let ke = self.model.ke;

        if (ka - ke).abs() < 1e-9 {
            return dose_mg * f * ka * tau * (-ke * tau).exp();
        }
        dose_mg * f * ka / (ka - ke) * ((-ke * tau).exp() - (-ka * tau).exp())
    }
}
