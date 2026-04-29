use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod pk;

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Clone)]
pub struct RouteInfo {
    #[wasm_bindgen(getter_with_clone)]
    pub route: String,
    #[wasm_bindgen(getter_with_clone)]
    pub unit: String,
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Clone)]
pub struct DrugRecord {
    #[wasm_bindgen(getter_with_clone)]
    pub drug_id: String,
    #[wasm_bindgen(getter_with_clone)]
    pub name: String,
    #[wasm_bindgen(getter_with_clone)]
    pub model_type: String,
    #[wasm_bindgen(getter_with_clone)]
    pub dose_unit: String,
    #[wasm_bindgen(skip)]
    pub routes: Vec<RouteInfo>,
    #[wasm_bindgen(getter_with_clone)]
    pub display_unit: String,
    pub unit_conversion_factor: f64,
    pub half_life: f64,
    pub volume_of_distribution: f64,
    pub clearance: f64,
    pub ka: f64,
    pub bioavailability: f64,
    pub k12: f64,
    pub k21: f64,
    #[wasm_bindgen(getter_with_clone)]
    pub parent_compound: String,
    pub equivalence_factor: f64,
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Clone)]
pub struct DoseRecord {
    #[wasm_bindgen(getter_with_clone)]
    pub dose_id: String,
    #[wasm_bindgen(getter_with_clone)]
    pub drug_id: String,
    pub dose_amount: f64,
    pub timestamp: f64,
    #[wasm_bindgen(getter_with_clone)]
    pub route: String,
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct SimulationOutput {
    #[wasm_bindgen(getter_with_clone)]
    pub time_h: Vec<f64>,
    #[wasm_bindgen(getter_with_clone)]
    pub concentrations: Vec<f64>,
    #[wasm_bindgen(getter_with_clone)]
    pub drug_name: String,
    #[wasm_bindgen(getter_with_clone)]
    pub display_unit: String,
}

#[derive(Clone, Copy)]
struct DoseWithDrug<'a> {
    dose: &'a DoseRecord,
    drug: &'a DrugRecord,
}

#[wasm_bindgen]
pub struct ValenceEngine {
    drugs: HashMap<String, DrugRecord>,
    doses: Vec<DoseRecord>,
    mw: pk::EsterMW,
    e2_model: pk::e2::E2OneCompartment,
    ester_model: pk::ester::EsterMultiCompartment,
    cpa_model: pk::cpa::CPATwoCompartment,
    weight_kg: f64,
}

impl ValenceEngine {
    fn is_e2_family(&self, drug: &DrugRecord) -> bool {
        if drug.parent_compound == "Estradiol" { return true; }
        if drug.drug_id.starts_with("hrt_e") && drug.drug_id != "hrt_cpa" { return true; }
        false
    }

    fn is_cpa(&self, drug: &DrugRecord) -> bool {
        drug.drug_id == "hrt_cpa"
            || drug.parent_compound == "CPA"
            || drug.name == "Cyproterone Acetate"
    }

    fn collect_doses_with_drug(&self) -> Vec<DoseWithDrug<'_>> {
        self.doses.iter()
            .filter_map(|d| self.drugs.get(&d.drug_id).map(|drug| DoseWithDrug { dose: d, drug }))
            .collect()
    }
}

#[wasm_bindgen]
impl ValenceEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        ValenceEngine {
            drugs: HashMap::new(),
            doses: Vec::new(),
            mw: pk::EsterMW::new(),
            e2_model: pk::e2::E2OneCompartment::new(),
            ester_model: pk::ester::EsterMultiCompartment::new(),
            cpa_model: pk::cpa::CPATwoCompartment::new(),
            weight_kg: 60.0,
        }
    }

    #[wasm_bindgen(js_name = setWeight)]
    pub fn set_weight(&mut self, kg: f64) {
        if kg > 0.0 { self.weight_kg = kg; }
    }

    #[wasm_bindgen(js_name = getWeight)]
    pub fn get_weight(&self) -> f64 { self.weight_kg }

    #[wasm_bindgen(js_name = registerDrug)]
    pub fn register_drug(&mut self, drug: JsValue) -> Result<(), JsValue> {
        let d: DrugRecord = serde_wasm_bindgen::from_value(drug)?;
        self.drugs.insert(d.drug_id.clone(), d);
        Ok(())
    }

    #[wasm_bindgen(js_name = registerDrugs)]
    pub fn register_drugs(&mut self, drugs: JsValue) -> Result<(), JsValue> {
        let list: Vec<DrugRecord> = serde_wasm_bindgen::from_value(drugs)?;
        for d in list { self.drugs.insert(d.drug_id.clone(), d); }
        Ok(())
    }

    #[wasm_bindgen(js_name = removeDrug)]
    pub fn remove_drug(&mut self, drug_id: &str) -> bool {
        self.drugs.remove(drug_id).is_some()
    }

    #[wasm_bindgen(js_name = getDrug)]
    pub fn get_drug(&self, drug_id: &str) -> JsValue {
        match self.drugs.get(drug_id) {
            Some(d) => serde_wasm_bindgen::to_value(d).unwrap_or(JsValue::NULL),
            None => JsValue::NULL,
        }
    }

    #[wasm_bindgen(js_name = getAllDrugs)]
    pub fn get_all_drugs(&self) -> JsValue {
        let list: Vec<&DrugRecord> = self.drugs.values().collect();
        serde_wasm_bindgen::to_value(&list).unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen(js_name = getDrugCount)]
    pub fn get_drug_count(&self) -> usize { self.drugs.len() }

    #[wasm_bindgen(js_name = addDose)]
    pub fn add_dose(&mut self, dose: JsValue) -> Result<(), JsValue> {
        let d: DoseRecord = serde_wasm_bindgen::from_value(dose)?;
        self.doses.push(d);
        Ok(())
    }

    #[wasm_bindgen(js_name = removeDose)]
    pub fn remove_dose(&mut self, dose_id: &str) -> bool {
        let before = self.doses.len();
        self.doses.retain(|d| d.dose_id != dose_id);
        self.doses.len() != before
    }

    #[wasm_bindgen(js_name = getAllDoses)]
    pub fn get_all_doses(&self) -> JsValue {
        let mut sorted = self.doses.clone();
        sorted.sort_by(|a, b| a.timestamp.partial_cmp(&b.timestamp).unwrap());
        serde_wasm_bindgen::to_value(&sorted).unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen(js_name = getDoseCount)]
    pub fn get_dose_count(&self) -> usize { self.doses.len() }

    #[wasm_bindgen(js_name = clearDoses)]
    pub fn clear_doses(&mut self) { self.doses.clear(); }

    #[wasm_bindgen(js_name = clearDrugs)]
    pub fn clear_drugs(&mut self) { self.drugs.clear(); }

    #[wasm_bindgen(js_name = exportData)]
    pub fn export_data(&self) -> JsValue {
        let drugs: Vec<&DrugRecord> = self.drugs.values().collect();
        let mut sorted_doses = self.doses.clone();
        sorted_doses.sort_by(|a, b| a.timestamp.partial_cmp(&b.timestamp).unwrap());
        let payload = serde_json::json!({
            "weight": self.weight_kg,
            "drugs": drugs,
            "doses": sorted_doses,
        });
        serde_wasm_bindgen::to_value(&payload).unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen(js_name = importData)]
    pub fn import_data(&mut self, json_str: &str) -> Result<(), JsValue> {
        let data: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| JsValue::from_str(&format!("JSON parse error: {}", e)))?;
        if let Some(w) = data.get("weight").and_then(|v| v.as_f64()) {
            if w > 0.0 { self.weight_kg = w; }
        }
        if let Some(drugs) = data.get("drugs").and_then(|v| v.as_array()) {
            for d in drugs {
                if let Ok(drug) = serde_json::from_value::<DrugRecord>(d.clone()) {
                    self.drugs.insert(drug.drug_id.clone(), drug);
                }
            }
        }
        if let Some(doses) = data.get("doses").and_then(|v| v.as_array()) {
            for d in doses {
                if let Ok(dose) = serde_json::from_value::<DoseRecord>(d.clone()) {
                    self.doses.push(dose);
                }
            }
        }
        Ok(())
    }

    #[wasm_bindgen(js_name = runSimulation)]
    pub fn run_simulation(&self) -> Vec<JsValue> {
        if self.doses.is_empty() || self.weight_kg <= 0.0 {
            return vec![];
        }

        let all_items = self.collect_doses_with_drug();

        if all_items.is_empty() {
            return vec![];
        }

        let mut e2_items: Vec<DoseWithDrug> = Vec::new();
        let mut cpa_items: Vec<DoseWithDrug> = Vec::new();
        let mut other_groups: HashMap<String, Vec<DoseWithDrug>> = HashMap::new();

        for item in &all_items {
            if self.is_e2_family(item.drug) {
                e2_items.push(*item);
            } else if self.is_cpa(item.drug) {
                cpa_items.push(*item);
            } else {
                other_groups.entry(item.drug.drug_id.clone()).or_default().push(*item);
            }
        }

        let mut results = Vec::new();

        if !e2_items.is_empty() {
            if let Some(output) = self.simulate_merged_e2(&e2_items) {
                results.push(serde_wasm_bindgen::to_value(&output).unwrap_or(JsValue::NULL));
            }
        }

        if !cpa_items.is_empty() {
            if let Some(output) = self.simulate_merged_cpa(&cpa_items) {
                results.push(serde_wasm_bindgen::to_value(&output).unwrap_or(JsValue::NULL));
            }
        }

        for (_drug_id, items) in &other_groups {
            let _rep_drug = items[0].drug;
            if let Some(output) = self.simulate_generic_group(items) {
                results.push(serde_wasm_bindgen::to_value(&output).unwrap_or(JsValue::NULL));
            }
        }

        results
    }
}

impl ValenceEngine {
    fn compute_time_range(items: &[DoseWithDrug], pad_hours_before: f64, pad_hours_after: f64) -> (f64, f64) {
        let mut times: Vec<f64> = items.iter().map(|i| i.dose.timestamp).collect();
        times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        (times[0] - pad_hours_before, *times.last().unwrap() + pad_hours_after)
    }

    fn simulate_merged_e2(&self, items: &[DoseWithDrug]) -> Option<SimulationOutput> {
        let (start_time, end_time) = Self::compute_time_range(items, 24.0, 24.0 * 14.0);

        let has_oral = items.iter().any(|i| i.dose.route != "injection");
        let max_step_h = if has_oral { 0.5 } else { 2.0 };
        let steps = ((end_time - start_time) / max_step_h).ceil() as usize;
        let steps = steps.max(1000);
        let step_size = (end_time - start_time) / (steps as f64 - 1.0);

        let vd_ml = pk::VD_PER_KG * self.weight_kg * 1000.0;

        let mut time_h = Vec::with_capacity(steps);
        let mut concentrations = Vec::with_capacity(steps);

        for i in 0..steps {
            let t = start_time + i as f64 * step_size;
            let mut total_mg = 0.0;

            for item in items {
                let tau = t - item.dose.timestamp;
                if tau < 0.0 { continue; }

                let key = self.mw.ester_key(&item.drug.name);
                let to_e2 = self.mw.to_e2_factor(&item.drug.name);

                if key == "E2" && item.dose.route != "injection" {
                    total_mg += self.e2_model.concentration(tau, item.dose.dose_amount, to_e2);
                } else if item.dose.route == "injection" {
                    total_mg += self.ester_model.injection_concentration(tau, item.dose.dose_amount, key, to_e2);
                } else {
                    total_mg += self.ester_model.oral_concentration(tau, item.dose.dose_amount, key, to_e2);
                }
            }

            time_h.push(t);
            concentrations.push(total_mg / vd_ml);
        }

        Some(SimulationOutput {
            time_h,
            concentrations,
            drug_name: "E2".to_string(),
            display_unit: "mg/L".to_string(),
        })
    }

    fn simulate_merged_cpa(&self, items: &[DoseWithDrug]) -> Option<SimulationOutput> {
        let (start_time, end_time) = Self::compute_time_range(items, 24.0, 24.0 * 14.0);

        let has_oral = items.iter().any(|i| i.dose.route != "injection");
        let max_step_h = if has_oral { 0.5 } else { 2.0 };
        let steps = ((end_time - start_time) / max_step_h).ceil() as usize;
        let steps = steps.max(1000);
        let step_size = (end_time - start_time) / (steps as f64 - 1.0);

        let vd_ml = self.cpa_model.v1_per_kg() * self.weight_kg * 1000.0;

        let mut time_h = Vec::with_capacity(steps);
        let mut concentrations = Vec::with_capacity(steps);

        for i in 0..steps {
            let t = start_time + i as f64 * step_size;
            let mut total_mg = 0.0;

            for item in items {
                let tau = t - item.dose.timestamp;
                if tau >= 0.0 {
                    total_mg += self.cpa_model.concentration(tau, item.dose.dose_amount);
                }
            }

            time_h.push(t);
            concentrations.push(total_mg / vd_ml);
        }

        Some(SimulationOutput {
            time_h,
            concentrations,
            drug_name: "CPA".to_string(),
            display_unit: "mg/L".to_string(),
        })
    }

    fn simulate_generic_group(&self, items: &[DoseWithDrug]) -> Option<SimulationOutput> {
        let rep_drug = items[0].drug;
        let (start_time, end_time) = Self::compute_time_range(items, 24.0, 24.0 * 7.0);

        let max_step_h = 0.5;
        let steps = ((end_time - start_time) / max_step_h).ceil() as usize;
        let steps = steps.max(500);
        let step_size = (end_time - start_time) / (steps as f64 - 1.0);

        let mut time_h = Vec::with_capacity(steps);
        let mut concentrations = Vec::with_capacity(steps);

        match rep_drug.model_type.as_str() {
            "one_compartment" => {
                let model = pk::one_comp::OneCompartment::from_params(
                    rep_drug.half_life,
                    rep_drug.volume_of_distribution,
                    rep_drug.ka,
                    rep_drug.bioavailability,
                );
                for i in 0..steps {
                    let t = start_time + i as f64 * step_size;
                    let mut total = 0.0;
                    for item in items {
                        let tau = t - item.dose.timestamp;
                        if tau >= 0.0 {
                            total += model.concentration(tau, item.dose.dose_amount, self.weight_kg);
                        }
                    }
                    time_h.push(t);
                    concentrations.push(total);
                }
            }
            "two_compartment" => {
                let model = pk::two_comp::TwoCompartment::from_params(
                    rep_drug.half_life,
                    rep_drug.volume_of_distribution,
                    rep_drug.ka,
                    rep_drug.bioavailability,
                    rep_drug.k12,
                    rep_drug.k21,
                );
                for i in 0..steps {
                    let t = start_time + i as f64 * step_size;
                    let mut total = 0.0;
                    for item in items {
                        let tau = t - item.dose.timestamp;
                        if tau >= 0.0 {
                            total += model.concentration(tau, item.dose.dose_amount, self.weight_kg);
                        }
                    }
                    time_h.push(t);
                    concentrations.push(total);
                }
            }
            "multi_compartment" => {
                let model = pk::one_comp::OneCompartment::from_params(
                    rep_drug.half_life,
                    rep_drug.volume_of_distribution,
                    rep_drug.ka,
                    rep_drug.bioavailability * rep_drug.equivalence_factor,
                );
                for i in 0..steps {
                    let t = start_time + i as f64 * step_size;
                    let mut total = 0.0;
                    for item in items {
                        let tau = t - item.dose.timestamp;
                        if tau >= 0.0 {
                            total += model.concentration(tau, item.dose.dose_amount, self.weight_kg);
                        }
                    }
                    time_h.push(t);
                    concentrations.push(total);
                }
            }
            _ => {
                return Some(SimulationOutput {
                    time_h: vec![],
                    concentrations: vec![],
                    drug_name: rep_drug.name.clone(),
                    display_unit: rep_drug.display_unit.clone(),
                });
            }
        }

        Some(SimulationOutput {
            time_h,
            concentrations,
            drug_name: rep_drug.name.clone(),
            display_unit: rep_drug.display_unit.clone(),
        })
    }
}
