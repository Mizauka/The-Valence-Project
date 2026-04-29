use crate::pk::two_comp::TwoCompartment;

pub struct CPATwoCompartment {
    model: TwoCompartment,
}

impl CPATwoCompartment {
    pub fn new() -> Self {
        CPATwoCompartment {
            model: TwoCompartment::new(0.88, 0.60, 0.20, 0.01579, 0.04, 2.666),
        }
    }

    pub fn concentration(&self, tau: f64, dose_mg: f64) -> f64 {
        self.model.amount(tau, dose_mg)
    }

    pub fn v1_per_kg(&self) -> f64 {
        self.model.v1_per_kg
    }
}
