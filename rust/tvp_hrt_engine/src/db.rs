use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::{HrtL1Compound, HrtL2Route, HrtL3DoseModel};

#[derive(Debug, Clone)]
pub struct HrtDatabase {
    pub root: PathBuf,
    pub compounds: HashMap<String, HrtL1Compound>,
    pub routes: HashMap<String, HrtL2Route>,
    pub dosages: HashMap<(String, String), HrtL3DoseModel>,

    pub compound_name_to_id: HashMap<String, String>,
}

impl HrtDatabase {
    /// root 目录结构：
    /// - substances/*.json
    /// - routes/*.json
    /// - dosages/*.json
    pub fn load_from_assets_dir(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        let substances_dir = root.join("substances");
        let routes_dir = root.join("routes");
        let dosages_dir = root.join("dosages");

        let mut compounds: HashMap<String, HrtL1Compound> = HashMap::new();
        let mut routes: HashMap<String, HrtL2Route> = HashMap::new();
        let mut dosages: HashMap<(String, String), HrtL3DoseModel> = HashMap::new();

        for entry in fs::read_dir(&substances_dir)
            .with_context(|| format!("读取目录失败：{}", substances_dir.to_string_lossy()))?
        {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let c: HrtL1Compound = read_json(&path)?;
            compounds.insert(c.id.clone(), c);
        }

        for entry in fs::read_dir(&routes_dir)
            .with_context(|| format!("读取目录失败：{}", routes_dir.to_string_lossy()))?
        {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let r: HrtL2Route = read_json(&path)?;
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
            let d: HrtL3DoseModel = read_json(&path)?;
            dosages.insert((d.compound_id.clone(), d.route_id.clone()), d);
        }

        let mut compound_name_to_id: HashMap<String, String> = HashMap::new();
        for (id, c) in &compounds {
            compound_name_to_id.insert(c.name.to_ascii_lowercase(), id.clone());
        }

        Ok(Self {
            root,
            compounds,
            routes,
            dosages,
            compound_name_to_id,
        })
    }

    pub fn compound_by_name_or_id(&self, name_or_id: &str) -> Option<&HrtL1Compound> {
        if let Some(c) = self.compounds.get(name_or_id) {
            return Some(c);
        }
        let key = name_or_id.to_ascii_lowercase();
        let id = self.compound_name_to_id.get(&key)?;
        self.compounds.get(id)
    }

    pub fn route_by_id_or_name(&self, id_or_name: &str) -> Option<&HrtL2Route> {
        if let Some(r) = self.routes.get(id_or_name) {
            return Some(r);
        }
        let key = id_or_name.to_ascii_lowercase();
        self.routes
            .values()
            .find(|r| r.name.to_ascii_lowercase() == key)
    }

    pub fn dosage(&self, compound_id: &str, route_id: &str) -> Option<&HrtL3DoseModel> {
        self.dosages
            .get(&(compound_id.to_string(), route_id.to_string()))
    }
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("读取失败：{}", path.to_string_lossy()))?;
    serde_json::from_str(&raw).with_context(|| format!("解析失败：{}", path.to_string_lossy()))
}
