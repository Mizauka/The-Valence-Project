use std::collections::HashMap;

fn param(map: &HashMap<String, f64>, key: &str, default: f64) -> f64 {
    map.get(key).copied().unwrap_or(default)
}

pub struct TwoCompartment {
    f: f64,
    ka: f64,
    alpha: f64,
    beta: f64,
    k21: f64,
}

impl TwoCompartment {
    pub fn from_params(params: &HashMap<String, f64>) -> Self {
        let half_life = param(params, "half_life", 0.0);
        let bioavailability = param(params, "bioavailability", 1.0);
        let ka = param(params, "ka", 0.6);
        let k12 = param(params, "k12", 0.0);
        let k21 = param(params, "k21", 0.04);
        let beta = if half_life > 0.0 { 0.693 / half_life } else { 0.01579 };
        let ke = beta;
        let alpha = ke + k12 + k21;
        TwoCompartment {
            f: bioavailability,
            ka,
            alpha,
            beta,
            k21,
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
}
