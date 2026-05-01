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
    #[serde(default)]
    pub group_id: String,
    #[wasm_bindgen(getter_with_clone)]
    pub dose_unit: String,
    #[wasm_bindgen(skip)]
    pub routes: Vec<RouteInfo>,
    #[wasm_bindgen(getter_with_clone)]
    pub display_unit: String,
    #[serde(default)]
    pub molecular_weight: f64,
    #[serde(default)]
    pub depot_model: bool,
    #[wasm_bindgen(skip)]
    #[serde(default)]
    pub parameters: HashMap<String, f64>,
}

#[wasm_bindgen]
impl DrugRecord {
    #[wasm_bindgen(getter, js_name = "params")]
    pub fn params_js(&self) -> JsValue {
        let obj = js_sys::Object::new();
        for (k, v) in &self.parameters {
            js_sys::Reflect::set(&obj, &k.clone().into(), &JsValue::from_f64(*v)).ok();
        }
        obj.into()
    }
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
    weight_kg: f64,
}

fn param(map: &HashMap<String, f64>, key: &str, default: f64) -> f64 {
    map.get(key).copied().unwrap_or(default)
}

impl ValenceEngine {
    fn collect_doses_with_drug(&self) -> Vec<DoseWithDrug<'_>> {
        self.doses.iter()
            .filter_map(|d| self.drugs.get(&d.drug_id).map(|drug| DoseWithDrug { dose: d, drug }))
            .collect()
    }

    fn resolve_group_id(drug: &DrugRecord) -> String {
        if drug.group_id.is_empty() { drug.drug_id.clone() } else { drug.group_id.clone() }
    }

    fn compute_molar_factor(&self, drug: &DrugRecord) -> f64 {
        let eq = param(&drug.parameters, "equivalence_factor", 0.0);
        if eq > 0.0 { return eq; }
        if drug.group_id.is_empty() || drug.molecular_weight <= 0.0 {
            return 1.0;
        }
        for (_id, other) in &self.drugs {
            if other.group_id == drug.group_id
                && other.drug_id != drug.drug_id
                && !other.depot_model
                && other.molecular_weight > 0.0
            {
                return other.molecular_weight / drug.molecular_weight;
            }
        }
        1.0
    }

    fn resolve_group_vd(&self, items: &[DoseWithDrug]) -> f64 {
        let rep = items[0].drug;
        if rep.group_id.is_empty() {
            return param(&rep.parameters, "volume_of_distribution", 1.0);
        }
        for (_id, other) in &self.drugs {
            if other.group_id == rep.group_id && !other.depot_model {
                let vd = param(&other.parameters, "volume_of_distribution", 0.0);
                if vd > 0.0 { return vd; }
            }
        }
        param(&rep.parameters, "volume_of_distribution", 1.0)
    }
}

#[wasm_bindgen]
impl ValenceEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        ValenceEngine { drugs: HashMap::new(), doses: Vec::new(), weight_kg: 60.0 }
    }

    #[wasm_bindgen(js_name = setWeight)]
    pub fn set_weight(&mut self, kg: f64) { if kg > 0.0 { self.weight_kg = kg; } }

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
    pub fn remove_drug(&mut self, drug_id: &str) -> bool { self.drugs.remove(drug_id).is_some() }

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
        let payload = serde_json::json!({ "weight": self.weight_kg, "drugs": drugs, "doses": sorted_doses });
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
        if self.doses.is_empty() || self.weight_kg <= 0.0 { return vec![]; }
        let all_items = self.collect_doses_with_drug();
        if all_items.is_empty() { return vec![]; }

        let mut groups: HashMap<String, Vec<DoseWithDrug>> = HashMap::new();
        for item in &all_items {
            let gid = Self::resolve_group_id(item.drug);
            groups.entry(gid).or_default().push(*item);
        }

        let mut results = Vec::new();
        for (_gid, items) in &groups {
            if let Some(output) = self.simulate_group(items) {
                results.push(serde_wasm_bindgen::to_value(&output).unwrap_or(JsValue::NULL));
            }
        }
        results
    }
}

impl ValenceEngine {
    fn compute_time_range(items: &[DoseWithDrug], pad_before: f64, pad_after: f64) -> (f64, f64) {
        let mut times: Vec<f64> = items.iter().map(|i| i.dose.timestamp).collect();
        if times.is_empty() { return (0.0, 1.0); }
        times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        (times[0] - pad_before, *times.last().unwrap() + pad_after)
    }

    fn simulate_group(&self, items: &[DoseWithDrug]) -> Option<SimulationOutput> {
        if items.is_empty() { return None; }

        let rep_drug = items[0].drug;
        let group_name = if rep_drug.group_id.is_empty() {
            rep_drug.name.clone()
        } else {
            rep_drug.group_id.clone()
        };
        let display_unit = if rep_drug.display_unit.is_empty() {
            "mg/L".to_string()
        } else {
            rep_drug.display_unit.clone()
        };

        let any_injection = items.iter().any(|i| i.dose.route == "injection");
        let any_oral = items.iter().any(|i| i.dose.route != "injection");
        let is_depot = rep_drug.depot_model;

        let pad_after = if is_depot && any_injection { 24.0 * 14.0 } else { 24.0 * 7.0 };
        let max_step_h = if any_oral { 0.5 } else { 2.0 };
        let (start_time, end_time) = Self::compute_time_range(items, 24.0, pad_after);

        let steps = ((end_time - start_time) / max_step_h).ceil() as usize;
        let steps = steps.max(500);
        let step_size = (end_time - start_time) / (steps as f64 - 1.0);

        let vd_per_kg = self.resolve_group_vd(items);
        let vd_ml = vd_per_kg * self.weight_kg * 1000.0;
        if vd_ml <= 0.0 { return None; }

        let mut time_h = Vec::with_capacity(steps);
        let mut concentrations = Vec::with_capacity(steps);

        for i in 0..steps {
            let t = start_time + i as f64 * step_size;
            let mut total = 0.0;
            for item in items {
                let tau = t - item.dose.timestamp;
                if tau < 0.0 { continue; }
                let route = &item.dose.route;
                total += route_amount(item.drug, tau, item.dose.dose_amount, route,
                    self.compute_molar_factor(item.drug),
                );
            }
            time_h.push(t);
            concentrations.push(total / vd_ml);
        }

        Some(SimulationOutput { time_h, concentrations, drug_name: group_name, display_unit })
    }
}

fn route_amount(drug: &DrugRecord, tau: f64, dose_mg: f64, route: &str, molar_factor: f64) -> f64 {
    if drug.depot_model {
        if route == "injection" {
            return pk::ester::depot_injection_amount(tau, dose_mg, &drug.parameters, molar_factor);
        } else {
            return pk::ester::depot_oral_amount(tau, dose_mg, &drug.parameters, molar_factor);
        }
    }

    match drug.model_type.as_str() {
        "one_compartment" | "multi_compartment" => {
            let m = pk::one_comp::OneCompartment::from_params(&drug.parameters);
            let eq = param(&drug.parameters, "equivalence_factor", 1.0);
            m.amount(tau, dose_mg * eq)
        }
        "two_compartment" => {
            let m = pk::two_comp::TwoCompartment::from_params(&drug.parameters);
            m.amount(tau, dose_mg)
        }
        _ => 0.0,
    }
}
