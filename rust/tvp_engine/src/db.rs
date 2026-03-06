use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Database {
    pub root: PathBuf,
    pub substances: HashMap<String, L1Substance>, // id -> substance
    pub routes: HashMap<String, L2Route>,         // id -> route
    pub dosages: HashMap<(String, String), L3Dosage>, // (substance_id, route_id) -> dosage

    pub substance_name_to_id: HashMap<String, String>, // lower(name) -> id
    pub category_to_substance_ids: HashMap<String, Vec<String>>, // lower(category) -> [substance_id]
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct L1Substance {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub common_names: Vec<String>,
    pub url: Option<String>,
    #[serde(default)]
    pub categories: Vec<String>,
    pub summary: Option<String>,

    pub half_life_hours: Option<f64>,
    pub distribution_volume_l_per_kg: Option<f64>,
    pub mechanisms: Option<serde_json::Value>,

    pub ddi: DdiMatrix,

    #[serde(default)]
    pub legacy: serde_json::Value,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DdiMatrix {
    #[serde(default)]
    pub dangerous: Vec<String>,
    #[serde(rename = "unsafe", default)]
    pub unsafe_list: Vec<String>,
    #[serde(default)]
    pub uncertain: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct L2Route {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub default_ka_per_hour: f64,
    pub default_f: f64,
    pub equation: String,
    pub is_placeholder: bool,
    pub source: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Range {
    pub min: f64,
    pub max: f64,
    pub units: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct L3Dosage {
    pub schema_version: u32,
    pub substance_id: String,
    pub substance_name: String,
    pub route_id: String,
    pub route_name: String,

    pub dose_units: String,
    pub threshold: Option<f64>,
    pub light_min: Option<f64>,
    pub common_min: Option<f64>,
    pub strong_min: Option<f64>,
    pub heavy_min: Option<f64>,

    pub onset: Option<Range>,
    pub comeup: Option<Range>,
    pub peak: Option<Range>,
    pub offset: Option<Range>,
    pub total: Option<Range>,
    pub afterglow: Option<Range>,

    #[serde(default)]
    pub legacy: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct DbStats {
    pub l1_substances: usize,
    pub l2_routes: usize,
    pub l3_dosages: usize,
    pub ddi_ref_total: usize,
    pub categories: usize,
}

impl Database {
    pub fn load_from_assets_dir(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        let substances_dir = root.join("substances");
        let routes_dir = root.join("routes");
        let dosages_dir = root.join("dosages");

        let mut substances: HashMap<String, L1Substance> = HashMap::new();
        let mut routes: HashMap<String, L2Route> = HashMap::new();
        let mut dosages: HashMap<(String, String), L3Dosage> = HashMap::new();

        for entry in fs::read_dir(&substances_dir)
            .with_context(|| format!("读取目录失败：{}", substances_dir.to_string_lossy()))?
        {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let s: L1Substance = read_json(&path)?;
            substances.insert(s.id.clone(), s);
        }

        for entry in fs::read_dir(&routes_dir)
            .with_context(|| format!("读取目录失败：{}", routes_dir.to_string_lossy()))?
        {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let r: L2Route = read_json(&path)?;
            routes.insert(r.id.clone(), r);
        }

        for entry in fs::read_dir(&dosages_dir)
            .with_context(|| format!("读取目录失败：{}", dosages_dir.to_string_lossy()))?
        {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let d: L3Dosage = read_json(&path)?;
            dosages.insert((d.substance_id.clone(), d.route_id.clone()), d);
        }

        // 索引
        let mut substance_name_to_id: HashMap<String, String> = HashMap::new();
        let mut category_to_substance_ids: HashMap<String, Vec<String>> = HashMap::new();
        for (id, s) in &substances {
            substance_name_to_id.insert(s.name.to_ascii_lowercase(), id.clone());
            for cn in &s.common_names {
                substance_name_to_id
                    .entry(cn.to_ascii_lowercase())
                    .or_insert_with(|| id.clone());
            }
            for cat in &s.categories {
                category_to_substance_ids
                    .entry(cat.to_ascii_lowercase())
                    .or_default()
                    .push(id.clone());
            }
        }

        Ok(Self {
            root,
            substances,
            routes,
            dosages,
            substance_name_to_id,
            category_to_substance_ids,
        })
    }

    pub fn stats(&self) -> DbStats {
        let mut ddi_ref_total = 0usize;
        let mut cats: HashSet<String> = HashSet::new();
        for s in self.substances.values() {
            ddi_ref_total += s.ddi.dangerous.len();
            ddi_ref_total += s.ddi.unsafe_list.len();
            ddi_ref_total += s.ddi.uncertain.len();
            for c in &s.categories {
                cats.insert(c.to_ascii_lowercase());
            }
        }
        DbStats {
            l1_substances: self.substances.len(),
            l2_routes: self.routes.len(),
            l3_dosages: self.dosages.len(),
            ddi_ref_total,
            categories: cats.len(),
        }
    }

    pub fn substance_by_id(&self, id: &str) -> Option<&L1Substance> {
        self.substances.get(id)
    }

    pub fn substance_by_name_or_id(&self, name_or_id: &str) -> Option<&L1Substance> {
        if let Some(s) = self.substances.get(name_or_id) {
            return Some(s);
        }
        let key = name_or_id.to_ascii_lowercase();
        let id = self.substance_name_to_id.get(&key)?;
        self.substances.get(id)
    }

    pub fn route_by_id_or_name(&self, id_or_name: &str) -> Option<&L2Route> {
        if let Some(r) = self.routes.get(id_or_name) {
            return Some(r);
        }
        let key = id_or_name.to_ascii_lowercase();
        self.routes
            .values()
            .find(|r| r.name.to_ascii_lowercase() == key)
    }

    pub fn dosage(&self, substance_id: &str, route_id: &str) -> Option<&L3Dosage> {
        self.dosages
            .get(&(substance_id.to_string(), route_id.to_string()))
    }

    pub fn all_route_names(&self) -> BTreeMap<String, String> {
        let mut m = BTreeMap::new();
        for r in self.routes.values() {
            m.insert(r.id.clone(), r.name.clone());
        }
        m
    }
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("读取失败：{}", path.to_string_lossy()))?;
    serde_json::from_str(&raw).with_context(|| format!("解析失败：{}", path.to_string_lossy()))
}
