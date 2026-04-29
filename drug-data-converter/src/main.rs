use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Clone)]
struct StandardDrug {
    drug_id: String,
    name: String,
    model_type: String,
    parameters: StandardDrugParameters,
}

#[derive(Serialize, Clone)]
struct StandardDrugParameters {
    half_life: f64,
    volume_of_distribution: f64,
    clearance: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    ka: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bioavailability: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    equivalence_factor: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_compound: Option<String>,
}

#[derive(Deserialize)]
struct JournalSubstanceFile {
    substances: Vec<JournalSubstance>,
}

#[derive(Deserialize)]
struct JournalSubstance {
    name: String,
    #[serde(default)]
    roas: Vec<JournalRoa>,
}

#[derive(Deserialize)]
struct JournalRoa {
    name: String,
    dose: Option<JournalDose>,
    duration: Option<JournalDuration>,
    bioavailability: Option<JournalBioavailability>,
}

#[derive(Deserialize)]
struct JournalDose {
    #[serde(default)]
    units: Option<String>,
    #[serde(default)]
    light_min: Option<f64>,
    #[serde(default = "default_common_min")]
    common_min: Option<f64>,
    #[serde(default)]
    strong_min: Option<f64>,
    #[serde(default)]
    heavy_min: Option<f64>,
}

fn default_common_min() -> Option<f64> { None }

#[derive(Deserialize)]
struct JournalDuration {
    #[serde(default)]
    total: Option<DurationRange>,
    #[serde(default)]
    onset: Option<DurationRange>,
    #[serde(default)]
    peak: Option<DurationRange>,
    #[serde(default)]
    offset: Option<DurationRange>,
    #[serde(default)]
    comeup: Option<DurationRange>,
}

#[derive(Deserialize)]
struct DurationRange {
    #[serde(default)]
    min: f64,
    #[serde(default)]
    max: f64,
    #[serde(default)]
    units: String,
}

#[derive(Deserialize)]
struct JournalBioavailability {
    #[serde(default)]
    min: Option<f64>,
    #[serde(default)]
    max: Option<f64>,
}

#[derive(Deserialize)]
struct HrtEsterInfo {
    name: String,
    mw: f64,
}

#[derive(Deserialize)]
struct HrtTwoPartDepotPK {
    #[serde(rename = "Frac_fast")]
    frac_fast: HashMap<String, f64>,
    #[serde(rename = "k1_fast")]
    k1_fast: HashMap<String, f64>,
    #[serde(rename = "k1_slow")]
    k1_slow: HashMap<String, f64>,
}

#[derive(Deserialize)]
struct HrtEsterPK {
    k2: HashMap<String, f64>,
}

#[derive(Deserialize)]
struct HrtOralPK {
    #[serde(rename = "kAbsE2")]
    k_abs_e2: f64,
    #[serde(rename = "kAbsEV")]
    k_abs_ev: f64,
    bioavailability: f64,
}

#[derive(Deserialize)]
struct HrtCorePK {
    #[serde(rename = "vdPerKG")]
    vd_per_kg: f64,
    #[serde(rename = "kClear")]
    k_clear: f64,
    #[serde(rename = "kClearInjection")]
    k_clear_injection: f64,
}

const E2_MW: f64 = 272.38;

fn to_e2_factor(ester_mw: f64) -> f64 {
    E2_MW / ester_mw
}

fn convert_hours(value: f64, units: &str) -> f64 {
    match units {
        "minutes" => value / 60.0,
        "hours" => value,
        "days" => value * 24.0,
        _ => value,
    }
}

fn estimate_half_life_from_duration(duration: &JournalDuration) -> Option<f64> {
    let total_hours = duration.total.as_ref().map(|r| {
        let avg = (r.min + r.max) / 2.0;
        convert_hours(avg, &r.units)
    })?;

    Some(total_hours * 0.3)
}

fn convert_journal(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(input_path)?;
    let file: JournalSubstanceFile = serde_json::from_str(&content)?;

    let mut drugs: Vec<StandardDrug> = Vec::new();

    for substance in &file.substances {
        let mut best_half_life: Option<f64> = None;
        let mut best_bioavailability: Option<f64> = None;
        let mut best_ka: Option<f64> = None;
        let mut _has_mg_dose = false;

        for roa in &substance.roas {
            if let Some(ref dose) = roa.dose {
                if dose.units.as_deref() == Some("mg") {
                    _has_mg_dose = true;
                }
            }

            if let Some(ref duration) = roa.duration {
                if let Some(hl) = estimate_half_life_from_duration(duration) {
                    best_half_life = Some(match best_half_life {
                        Some(existing) => (existing + hl) / 2.0,
                        None => hl,
                    });
                }

                if let Some(ref onset) = duration.onset {
                    let avg_onset = (onset.min + onset.max) / 2.0;
                    let onset_h = convert_hours(avg_onset, &onset.units);
                    if onset_h > 0.0 {
                        let ka = 3.0 / onset_h;
                        best_ka = Some(match best_ka {
                            Some(existing) => (existing + ka) / 2.0,
                            None => ka,
                        });
                    }
                }
            }

            if let Some(ref bio) = roa.bioavailability {
                let avg_bio = match (bio.min, bio.max) {
                    (Some(min), Some(max)) => (min + max) / 2.0 / 100.0,
                    (Some(min), None) => min / 100.0,
                    (None, Some(max)) => max / 100.0,
                    _ => continue,
                };
                best_bioavailability = Some(avg_bio);
            }
        }

        let half_life = match best_half_life {
            Some(hl) if hl > 0.0 => hl,
            _ => continue,
        };

        let ke = 0.693 / half_life;
        let vd = 1.0;
        let clearance = ke * vd;

        drugs.push(StandardDrug {
            drug_id: format!("journal_{}", substance.name.to_lowercase().replace([' ', ',', '-', '/', '(', ')'], "_")),
            name: substance.name.clone(),
            model_type: "one_compartment".to_string(),
            parameters: StandardDrugParameters {
                half_life: (half_life * 100.0).round() / 100.0,
                volume_of_distribution: vd,
                clearance: (clearance * 1000.0).round() / 1000.0,
                ka: best_ka.map(|k| (k * 100.0).round() / 100.0),
                bioavailability: best_bioavailability.map(|b| (b * 1000.0).round() / 1000.0),
                equivalence_factor: None,
                parent_compound: None,
            },
        });
    }

    let output = serde_json::to_string_pretty(&drugs)?;
    std::fs::write(output_path, output)?;
    println!("Journal: converted {} substances -> {}", drugs.len(), output_path);
    Ok(())
}

fn convert_hrt_tracker(output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let ester_data = [
        ("E2", "Estradiol", 272.38),
        ("EB", "Estradiol Benzoate", 376.50),
        ("EV", "Estradiol Valerate", 356.50),
        ("EC", "Estradiol Cypionate", 396.58),
        ("EN", "Estradiol Enanthate", 384.56),
        ("CPA", "Cyproterone Acetate", 416.94),
    ];

    let depot_pk = [
        ("EB", 0.90, 0.144, 0.114),
        ("EV", 0.40, 0.0216, 0.0138),
        ("EC", 0.229164549, 0.005035046, 0.004510574),
        ("EN", 0.05, 0.0010, 0.0050),
    ];

    let ester_k2 = [
        ("EB", 0.090),
        ("EV", 0.070),
        ("EC", 0.045),
        ("EN", 0.015),
    ];

    let k_clear_injection = 0.041;
    let k_clear_oral = 0.41;
    let vd_per_kg = 2.0;

    let mut drugs: Vec<StandardDrug> = Vec::new();

    for (code, name, mw) in &ester_data {
        let eq_factor = to_e2_factor(*mw);
        let is_ester = code != &"E2" && code != &"CPA";

        let (half_life, model_type, clearance, ka, bioavailability): (f64, String, f64, Option<f64>, Option<f64>) = if code == &"CPA" {
            let alpha = 0.20;
            let beta = 0.01579;
            let apparent_hl = 0.693 / beta;
            let vd_cpa = 2.666;
            let cl_cpa = beta * vd_cpa;
            (apparent_hl, "two_compartment".to_string(), cl_cpa, Some(0.60), Some(0.88))
        } else if code == &"E2" {
            let hl = 0.693 / k_clear_oral;
            (hl, "one_compartment".to_string(), k_clear_oral * vd_per_kg, Some(0.32), Some(0.03))
        } else {
            let depot = depot_pk.iter().find(|(c, _, _, _)| c == code);
            let k2_entry = ester_k2.iter().find(|(c, _)| c == code);

            if let (Some((_, _, k1_fast, _)), Some((_, k2))) = (depot, k2_entry) {
                let effective_k = k1_fast + k2 + &k_clear_injection;
                let hl = 0.693 / (effective_k / 3.0);
                (hl, "multi_compartment".to_string(), k_clear_injection * vd_per_kg, None, None)
            } else {
                let hl = 0.693 / k_clear_oral;
                (hl, "one_compartment".to_string(), k_clear_oral * vd_per_kg, None, None)
            }
        };

        drugs.push(StandardDrug {
            drug_id: format!("hrt_{}", code.to_lowercase()),
            name: name.to_string(),
            model_type,
            parameters: StandardDrugParameters {
                half_life: (half_life * 100.0).round() / 100.0,
                volume_of_distribution: if code == &"CPA" { 2.666 } else { vd_per_kg },
                clearance: (clearance * 1000.0).round() / 1000.0,
                ka: ka.map(|k| (k * 1000.0).round() / 1000.0),
                bioavailability: bioavailability.map(|b| (b * 1000.0).round() / 1000.0),
                equivalence_factor: if is_ester { Some((eq_factor * 10000.0).round() / 10000.0) } else { None },
                parent_compound: if is_ester { Some("Estradiol".to_string()) } else { None },
            },
        });
    }

    let output = serde_json::to_string_pretty(&drugs)?;
    std::fs::write(output_path, output)?;
    println!("HRT Tracker: converted {} esters -> {}", drugs.len(), output_path);
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("journal") => {
            let input = args.get(2).map(|s| s.as_str()).unwrap_or(
                "../reference/psychonautwiki-journal-android/app/src/main/res/raw/substances.json"
            );
            let output = args.get(3).map(|s| s.as_str()).unwrap_or("../public/data/journal_drugs.json");
            if let Err(e) = convert_journal(input, output) {
                eprintln!("Error converting journal data: {}", e);
                std::process::exit(1);
            }
        }
        Some("hrt") => {
            let output = args.get(2).map(|s| s.as_str()).unwrap_or("../public/data/hrt_drugs.json");
            if let Err(e) = convert_hrt_tracker(output) {
                eprintln!("Error converting HRT tracker data: {}", e);
                std::process::exit(1);
            }
        }
        Some("all") => {
            let journal_input = "../reference/psychonautwiki-journal-android/app/src/main/res/raw/substances.json";
            let journal_output = "../public/data/journal_drugs.json";
            let hrt_output = "../public/data/hrt_drugs.json";

            if let Err(e) = convert_journal(journal_input, journal_output) {
                eprintln!("Error converting journal data: {}", e);
            }
            if let Err(e) = convert_hrt_tracker(hrt_output) {
                eprintln!("Error converting HRT tracker data: {}", e);
            }
        }
        _ => {
            println!("Usage: drug-data-converter <command> [args]");
            println!();
            println!("Commands:");
            println!("  journal [input] [output]  Convert PsychonautWiki Journal substances.json");
            println!("  hrt [output]              Convert HRT Tracker ester PK data");
            println!("  all                       Convert both data sources");
        }
    }
}
