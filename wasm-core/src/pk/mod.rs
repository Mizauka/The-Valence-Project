pub mod one_comp;
pub mod two_comp;
pub mod e2;
pub mod ester;
pub mod cpa;

pub const VD_PER_KG: f64 = 2.0;
pub const KCLEAR: f64 = 0.41;
pub const KCLEAR_INJECTION: f64 = 0.041;

pub struct EsterMW {
    pub e2: f64,
    pub eb: f64,
    pub ev: f64,
    pub ec: f64,
    pub en: f64,
    #[allow(dead_code)]
    pub cpa: f64,
}

impl EsterMW {
    pub fn new() -> Self {
        EsterMW {
            e2: 272.38,
            eb: 376.50,
            ev: 356.50,
            ec: 396.58,
            en: 384.56,
            cpa: 416.94,
        }
    }

    pub fn to_e2_factor(&self, name: &str) -> f64 {
        match name {
            "Estradiol" | "E2" => 1.0,
            "Estradiol Benzoate" | "EB" => self.e2 / self.eb,
            "Estradiol Valerate" | "EV" => self.e2 / self.ev,
            "Estradiol Cypionate" | "EC" => self.e2 / self.ec,
            "Estradiol Enanthate" | "EN" => self.e2 / self.en,
            _ => 1.0,
        }
    }

    pub fn ester_key(&self, name: &str) -> &'static str {
        match name {
            "Estradiol Benzoate" | "EB" => "EB",
            "Estradiol Valerate" | "EV" => "EV",
            "Estradiol Cypionate" | "EC" => "EC",
            "Estradiol Enanthate" | "EN" => "EN",
            "Estradiol" | "E2" => "E2",
            "Cyproterone Acetate" | "CPA" => "CPA",
            _ => "",
        }
    }
}
