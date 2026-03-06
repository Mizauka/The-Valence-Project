use crate::db::{Database, L1Substance};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    None = 0,
    Uncertain = 1,
    Unsafe = 2,
    Dangerous = 3,
}

#[derive(Debug, Clone)]
pub struct DdiHit {
    pub a_substance_id: String,
    pub a_substance_name: String,
    pub b_input: String,
    pub matched_as: MatchKind,
    pub risk: RiskLevel,
}

#[derive(Debug, Clone)]
pub enum MatchKind {
    SubstanceName(String), // matched substance_id
    Category(String),      // matched category name
    Raw(String),           // fallback: raw token matched literally
}

#[derive(Debug, Clone)]
pub struct DdiReport {
    pub highest: RiskLevel,
    pub hits: Vec<DdiHit>,
}

pub fn check_ddi(db: &Database, substances: &[&L1Substance]) -> DdiReport {
    let mut hits = Vec::new();
    let mut highest = RiskLevel::None;

    // build quick lookup: substance_id set and category set
    let mut present_ids = std::collections::HashSet::new();
    let mut present_categories = std::collections::HashSet::new();
    for s in substances {
        present_ids.insert(s.id.clone());
        for c in &s.categories {
            present_categories.insert(c.to_ascii_lowercase());
        }
    }

    for a in substances {
        // Each token can refer to a concrete substance name, or a category label in the legacy data.
        scan_tokens(
            db,
            &present_ids,
            &present_categories,
            a,
            &a.ddi.dangerous,
            RiskLevel::Dangerous,
            &mut hits,
            &mut highest,
        );
        scan_tokens(
            db,
            &present_ids,
            &present_categories,
            a,
            &a.ddi.unsafe_list,
            RiskLevel::Unsafe,
            &mut hits,
            &mut highest,
        );
        scan_tokens(
            db,
            &present_ids,
            &present_categories,
            a,
            &a.ddi.uncertain,
            RiskLevel::Uncertain,
            &mut hits,
            &mut highest,
        );
    }

    DdiReport { highest, hits }
}

fn scan_tokens(
    db: &Database,
    present_ids: &std::collections::HashSet<String>,
    present_categories: &std::collections::HashSet<String>,
    a: &L1Substance,
    tokens: &[String],
    risk: RiskLevel,
    hits: &mut Vec<DdiHit>,
    highest: &mut RiskLevel,
) {
    for raw in tokens {
        let token = raw.trim();
        if token.is_empty() {
            continue;
        }
        let token_l = token.to_ascii_lowercase();

        // 1) Match to substance by name/commonNames
        if let Some(b_id) = db.substance_name_to_id.get(&token_l) {
            // Only count if that substance is present in the set (avoid reporting generic tokens)
            if present_ids.contains(b_id) && b_id != &a.id {
                *highest = (*highest).max(risk);
                hits.push(DdiHit {
                    a_substance_id: a.id.clone(),
                    a_substance_name: a.name.clone(),
                    b_input: token.to_string(),
                    matched_as: MatchKind::SubstanceName(b_id.clone()),
                    risk,
                });
                continue;
            }
        }

        // 2) Match to a present category
        if present_categories.contains(&token_l) {
            *highest = (*highest).max(risk);
            hits.push(DdiHit {
                a_substance_id: a.id.clone(),
                a_substance_name: a.name.clone(),
                b_input: token.to_string(),
                matched_as: MatchKind::Category(token.to_string()),
                risk,
            });
            continue;
        }

        // 3) Legacy tokens sometimes are group labels (capitalized). If token literally matches any
        //    present substance name, treat as raw match.
        for b_id in present_ids {
            if let Some(b) = db.substances.get(b_id) {
                if b.name.eq_ignore_ascii_case(token) && b.id != a.id {
                    *highest = (*highest).max(risk);
                    hits.push(DdiHit {
                        a_substance_id: a.id.clone(),
                        a_substance_name: a.name.clone(),
                        b_input: token.to_string(),
                        matched_as: MatchKind::Raw(token.to_string()),
                        risk,
                    });
                    break;
                }
            }
        }
    }
}
