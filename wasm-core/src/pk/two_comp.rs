pub struct TwoCompartment {
    f: f64,
    ka: f64,
    alpha: f64,
    beta: f64,
    k21: f64,
    pub v1_per_kg: f64,
}

impl TwoCompartment {
    pub fn new(f: f64, ka: f64, alpha: f64, beta: f64, k21: f64, v1_per_kg: f64) -> Self {
        TwoCompartment { f, ka, alpha, beta, k21, v1_per_kg }
    }

    pub fn from_params(half_life: f64, v1_per_kg: f64, ka: f64, bioavailability: f64, k12: f64, k21: f64) -> Self {
        let beta = if half_life > 0.0 { 0.693 / half_life } else { 0.01579 };
        let ke = beta;
        let alpha = ke + k12 + k21;
        TwoCompartment {
            f: if bioavailability > 0.0 { bioavailability } else { 1.0 },
            ka: if ka > 0.0 { ka } else { 0.6 },
            alpha,
            beta,
            k21: if k21 > 0.0 { k21 } else { 0.04 },
            v1_per_kg: if v1_per_kg > 0.0 { v1_per_kg } else { 2.666 },
        }
    }

    pub fn amount(&self, tau: f64, dose_mg: f64) -> f64 {
        if tau < 0.0 || dose_mg <= 0.0 {
            return 0.0;
        }
        let eps = 1e-8;

        if (self.alpha - self.ka).abs() < eps
            || (self.beta - self.ka).abs() < eps
            || (self.alpha - self.beta).abs() < eps
        {
            if (self.ka - self.beta).abs() < eps {
                return (dose_mg * self.f * self.ka * tau * (-self.beta * tau).exp()).max(0.0);
            }
            return (dose_mg * self.f * self.ka / (self.ka - self.beta)
                * ((-self.beta * tau).exp() - (-self.ka * tau).exp()))
            .max(0.0);
        }

        let a = (self.k21 - self.ka) / ((self.alpha - self.ka) * (self.beta - self.ka));
        let b = (self.k21 - self.alpha) / ((self.ka - self.alpha) * (self.beta - self.alpha));
        let c = (self.k21 - self.beta) / ((self.ka - self.beta) * (self.alpha - self.beta));

        let val = dose_mg * self.f * self.ka
            * (a * (-self.ka * tau).exp()
                + b * (-self.alpha * tau).exp()
                + c * (-self.beta * tau).exp());
        val.max(0.0)
    }

    pub fn concentration(&self, tau: f64, dose_mg: f64, body_weight_kg: f64) -> f64 {
        let vd_ml = self.v1_per_kg * body_weight_kg * 1000.0;
        if vd_ml <= 0.0 {
            return 0.0;
        }
        self.amount(tau, dose_mg) / vd_ml
    }
}
