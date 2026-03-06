use std::collections::HashMap;

use crate::model::{DoseEvent, Ester, ExtraKey, Route, SimulationResult};

#[derive(Debug, Clone, Copy)]
pub struct SimulationGrid {
    pub start_h: f64,
    pub end_h: f64,
    pub steps: usize,
}

#[derive(Debug, Clone, Copy)]
struct CorePk {
    vd_per_kg_e2_l: f64,
    vd_per_kg_cpa_l: f64,
    k_clear_per_h: f64,
    k_clear_inj_per_h: f64,
    depot_k1_corr: f64,
}

const CORE_PK: CorePk = CorePk {
    vd_per_kg_e2_l: 2.0,
    vd_per_kg_cpa_l: 14.0,
    k_clear_per_h: 0.41,
    k_clear_inj_per_h: 0.041,
    depot_k1_corr: 1.0,
};

fn ester_mw(e: Ester) -> f64 {
    match e {
        Ester::E2 => 272.38,
        Ester::EB => 376.50,
        Ester::EV => 356.50,
        Ester::EC => 396.58,
        Ester::EN => 384.56,
        Ester::CPA => 416.94,
    }
}

fn to_e2_factor(ester: Ester) -> f64 {
    if ester == Ester::E2 {
        return 1.0;
    }
    ester_mw(Ester::E2) / ester_mw(ester)
}

#[derive(Debug, Clone, Copy)]
struct PkParams {
    frac_fast: f64,
    k1_fast: f64,
    k1_slow: f64,
    k2: f64,
    k3: f64,
    f_total: f64,
    rate_mg_h: f64,
    f_fast: f64,
    f_slow: f64,
}

fn get_bioavailability_multiplier(route: Route, ester: Ester, extras: &HashMap<ExtraKey, f64>) -> f64 {
    let mw_factor = to_e2_factor(ester);

    match route {
        Route::Injection => {
            // Default formationFraction matches Oyama logic.ts (InjectionPK.formationFraction)
            let default_formation = match ester {
                Ester::EB => 0.1092,
                Ester::EV => 0.0623,
                Ester::EC => 0.1173,
                Ester::EN => 0.12,
                Ester::E2 => 1.0,
                _ => 0.08,
            };

            let formation = extras
                .get(&ExtraKey::InjectionFormationFraction)
                .copied()
                .filter(|v| v.is_finite() && *v > 0.0)
                .unwrap_or(default_formation);

            formation * mw_factor
        }
        Route::Oral => 0.03 * mw_factor,
        Route::Sublingual => {
            let mut theta = 0.11;
            if let Some(custom) = extras.get(&ExtraKey::SublingualTheta) {
                if custom.is_finite() {
                    theta = custom.clamp(0.0, 1.0);
                }
            } else if let Some(tier_raw) = extras.get(&ExtraKey::SublingualTier) {
                if tier_raw.is_finite() {
                    let tier_idx = tier_raw.round() as i32;
                    let tier_idx = tier_idx.clamp(0, 3) as usize;
                    // quick, casual, standard, strict
                    theta = match tier_idx {
                        0 => 0.01,
                        1 => 0.04,
                        2 => 0.11,
                        3 => 0.18,
                        _ => 0.11,
                    };
                }
            }
            (theta + (1.0 - theta) * 0.03) * mw_factor
        }
        Route::Gel => {
            let site_idx = extras
                .get(&ExtraKey::GelSite)
                .copied()
                .unwrap_or(0.0)
                .round() as i32;
            let site_idx = site_idx.clamp(0, 2) as usize;
            let bio = match site_idx {
                0 => 0.05, // arm
                1 => 0.05, // thigh
                2 => 0.40, // scrotal
                _ => 0.05,
            };
            bio * mw_factor
        }
        Route::PatchApply => 1.0 * mw_factor,
        Route::PatchRemove => 0.0,
    }
}

fn resolve_params(event: &DoseEvent) -> PkParams {
    let default_k3 = if event.route == Route::Injection {
        CORE_PK.k_clear_inj_per_h
    } else {
        CORE_PK.k_clear_per_h
    };

    let to_e2 = to_e2_factor(event.ester);
    let extras = &event.extras;

    match event.route {
        Route::Injection => {
            let (frac_fast_default, k1_fast_default, k1_slow_default) = match event.ester {
                Ester::EB => (0.90, 0.144, 0.114),
                Ester::EV => (0.40, 0.0216, 0.0138),
                Ester::EC => (0.229164549, 0.005035046, 0.004510574),
                Ester::EN => (0.05, 0.0010, 0.0050),
                Ester::E2 => (1.0, 0.5, 0.0),
                _ => (0.5, 0.1, 0.01),
            };
            let frac_fast = extras
                .get(&ExtraKey::InjectionFracFast)
                .copied()
                .filter(|v| v.is_finite())
                .map(|v| v.clamp(0.0, 1.0))
                .unwrap_or(frac_fast_default);

            let k1_fast = extras
                .get(&ExtraKey::InjectionK1FastPerHour)
                .copied()
                .filter(|v| v.is_finite() && *v > 0.0)
                .unwrap_or(k1_fast_default)
                * CORE_PK.depot_k1_corr;

            let k1_slow = extras
                .get(&ExtraKey::InjectionK1SlowPerHour)
                .copied()
                .filter(|v| v.is_finite() && *v >= 0.0)
                .unwrap_or(k1_slow_default)
                * CORE_PK.depot_k1_corr;

            let k2_default = match event.ester {
                Ester::EB => 0.090,
                Ester::EV => 0.070,
                Ester::EC => 0.045,
                Ester::EN => 0.015,
                _ => 0.0,
            };
            let k2 = extras
                .get(&ExtraKey::InjectionK2PerHour)
                .copied()
                .filter(|v| v.is_finite() && *v >= 0.0)
                .unwrap_or(k2_default);

            let k3 = extras
                .get(&ExtraKey::InjectionK3PerHour)
                .copied()
                .filter(|v| v.is_finite() && *v > 0.0)
                .unwrap_or(default_k3);

            let f = get_bioavailability_multiplier(Route::Injection, event.ester, extras);
            PkParams {
                frac_fast,
                k1_fast,
                k1_slow,
                k2,
                k3,
                f_total: f,
                rate_mg_h: 0.0,
                f_fast: f,
                f_slow: f,
            }
        }

        Route::Sublingual => {
            let mut theta = 0.11;
            if let Some(custom) = extras.get(&ExtraKey::SublingualTheta) {
                if custom.is_finite() {
                    theta = custom.clamp(0.0, 1.0);
                }
            } else if let Some(tier_raw) = extras.get(&ExtraKey::SublingualTier) {
                if tier_raw.is_finite() {
                    let tier_idx = tier_raw.round() as i32;
                    let tier_idx = tier_idx.clamp(0, 3) as usize;
                    theta = match tier_idx {
                        0 => 0.01,
                        1 => 0.04,
                        2 => 0.11,
                        3 => 0.18,
                        _ => 0.11,
                    };
                }
            }

            let k1_fast = 1.8;
            let k1_slow = if event.ester == Ester::EV { 0.05 } else { 0.32 };
            let k2 = match event.ester {
                Ester::EB => 0.090,
                Ester::EV => 0.070,
                Ester::EC => 0.045,
                Ester::EN => 0.015,
                _ => 0.0,
            };

            let f_fast = to_e2;
            let f_slow = 0.03 * to_e2;
            let f_total = theta * f_fast + (1.0 - theta) * f_slow;

            PkParams {
                frac_fast: theta,
                k1_fast,
                k1_slow,
                k2,
                k3: default_k3,
                f_total,
                rate_mg_h: 0.0,
                f_fast,
                f_slow,
            }
        }

        Route::Gel => {
            let f = get_bioavailability_multiplier(Route::Gel, event.ester, extras);
            let k1 = 0.022;
            PkParams {
                frac_fast: 1.0,
                k1_fast: k1,
                k1_slow: 0.0,
                k2: 0.0,
                k3: default_k3,
                f_total: f,
                rate_mg_h: 0.0,
                f_fast: f,
                f_slow: f,
            }
        }

        Route::PatchApply => {
            let f = get_bioavailability_multiplier(Route::PatchApply, event.ester, extras);
            let rate_mg_h = extras
                .get(&ExtraKey::ReleaseRateUgPerDay)
                .copied()
                .filter(|v| v.is_finite() && *v > 0.0)
                .map(|ug_per_day| (ug_per_day / 24.0 / 1000.0) * f)
                .unwrap_or(0.0);

            if rate_mg_h > 0.0 {
                return PkParams {
                    frac_fast: 1.0,
                    k1_fast: 0.0,
                    k1_slow: 0.0,
                    k2: 0.0,
                    k3: default_k3,
                    f_total: f,
                    rate_mg_h,
                    f_fast: f,
                    f_slow: f,
                };
            }

            let k1 = 0.0075;
            PkParams {
                frac_fast: 1.0,
                k1_fast: k1,
                k1_slow: 0.0,
                k2: 0.0,
                k3: default_k3,
                f_total: f,
                rate_mg_h: 0.0,
                f_fast: f,
                f_slow: f,
            }
        }

        Route::PatchRemove => PkParams {
            frac_fast: 0.0,
            k1_fast: 0.0,
            k1_slow: 0.0,
            k2: 0.0,
            k3: default_k3,
            f_total: 0.0,
            rate_mg_h: 0.0,
            f_fast: 0.0,
            f_slow: 0.0,
        },

        Route::Oral => {
            // CPA 特殊处理
            if event.ester == Ester::CPA {
                return PkParams {
                    frac_fast: 1.0,
                    k1_fast: 1.0,
                    k1_slow: 0.0,
                    k2: 0.0,
                    k3: 0.017,
                    f_total: 0.7,
                    rate_mg_h: 0.0,
                    f_fast: 0.7,
                    f_slow: 0.7,
                };
            }

            let k1 = if event.ester == Ester::EV { 0.05 } else { 0.32 };
            let k2 = if event.ester == Ester::EV { 0.070 } else { 0.0 };
            let f = 0.03 * to_e2;
            PkParams {
                frac_fast: 1.0,
                k1_fast: k1,
                k1_slow: 0.0,
                k2,
                k3: default_k3,
                f_total: f,
                rate_mg_h: 0.0,
                f_fast: f,
                f_slow: f,
            }
        }
    }
}

fn analytic_3c(tau: f64, dose_mg: f64, f: f64, k1: f64, k2: f64, k3: f64) -> f64 {
    if k1 <= 0.0 || dose_mg <= 0.0 {
        return 0.0;
    }

    let k1_k2 = k1 - k2;
    let k1_k3 = k1 - k3;
    let k2_k3 = k2 - k3;

    if k1_k2.abs() < 1e-9 || k1_k3.abs() < 1e-9 || k2_k3.abs() < 1e-9 {
        return 0.0;
    }

    let term1 = (-k1 * tau).exp() / (k1_k2 * k1_k3);
    let term2 = (-k2 * tau).exp() / (-k1_k2 * k2_k3);
    let term3 = (-k3 * tau).exp() / (k1_k3 * k2_k3);

    dose_mg * f * k1 * k2 * (term1 + term2 + term3)
}

fn one_comp_amount(tau: f64, dose_mg: f64, p: PkParams) -> f64 {
    let k1 = p.k1_fast;
    if (k1 - p.k3).abs() < 1e-9 {
        return dose_mg * p.f_total * k1 * tau * (-p.k3 * tau).exp();
    }
    dose_mg * p.f_total * k1 / (k1 - p.k3) * ((-p.k3 * tau).exp() - (-k1 * tau).exp())
}

fn branch_amount(dose_mg: f64, f: f64, ka: f64, ke: f64, t: f64) -> f64 {
    if (ka - ke).abs() < 1e-9 {
        return dose_mg * f * ka * t * (-ke * t).exp();
    }
    dose_mg * f * ka / (ka - ke) * ((-ke * t).exp() - (-ka * t).exp())
}

#[derive(Debug, Clone)]
enum EventModel {
    Injection {
        start_h: f64,
        dose_mg: f64,
        p: PkParams,
    },
    OneComp {
        start_h: f64,
        dose_mg: f64,
        p: PkParams,
    },
    Sublingual {
        start_h: f64,
        dose_mg: f64,
        p: PkParams,
    },
    PatchApply {
        start_h: f64,
        dose_mg: f64,
        p: PkParams,
        wear_h: f64,
    },
}

impl EventModel {
    fn new(event: &DoseEvent, all_events: &[DoseEvent]) -> Option<Self> {
        if event.route == Route::PatchRemove {
            return None;
        }

        let p = resolve_params(event);
        let start_h = event.time_h;
        let dose_mg = event.dose_mg;

        match event.route {
            Route::Injection => Some(Self::Injection {
                start_h,
                dose_mg,
                p,
            }),
            Route::Gel | Route::Oral => Some(Self::OneComp {
                start_h,
                dose_mg,
                p,
            }),
            Route::Sublingual => Some(Self::Sublingual {
                start_h,
                dose_mg,
                p,
            }),
            Route::PatchApply => {
                let remove_time = all_events
                    .iter()
                    .find(|e| e.route == Route::PatchRemove && e.time_h > start_h)
                    .map(|e| e.time_h)
                    .unwrap_or(f64::INFINITY);
                let wear_h = remove_time - start_h;
                Some(Self::PatchApply {
                    start_h,
                    dose_mg,
                    p,
                    wear_h,
                })
            }
            Route::PatchRemove => None,
        }
    }

    fn amount_mg(&self, time_h: f64) -> f64 {
        match *self {
            Self::Injection {
                start_h,
                dose_mg,
                p,
            } => {
                let tau = time_h - start_h;
                if tau < 0.0 {
                    return 0.0;
                }
                let dose_fast = dose_mg * p.frac_fast;
                let dose_slow = dose_mg * (1.0 - p.frac_fast);
                analytic_3c(tau, dose_fast, p.f_total, p.k1_fast, p.k2, p.k3)
                    + analytic_3c(tau, dose_slow, p.f_total, p.k1_slow, p.k2, p.k3)
            }

            Self::OneComp {
                start_h,
                dose_mg,
                p,
            } => {
                let tau = time_h - start_h;
                if tau < 0.0 {
                    return 0.0;
                }
                one_comp_amount(tau, dose_mg, p)
            }

            Self::Sublingual {
                start_h,
                dose_mg,
                p,
            } => {
                let tau = time_h - start_h;
                if tau < 0.0 {
                    return 0.0;
                }

                let dose_f = dose_mg * p.frac_fast;
                let dose_s = dose_mg * (1.0 - p.frac_fast);

                let fast_amount = if p.k2 > 0.0 {
                    analytic_3c(tau, dose_f, p.f_fast, p.k1_fast, p.k2, p.k3)
                } else {
                    branch_amount(dose_f, p.f_fast, p.k1_fast, p.k3, tau)
                };

                let slow_amount = branch_amount(dose_s, p.f_slow, p.k1_slow, p.k3, tau);

                fast_amount + slow_amount
            }

            Self::PatchApply {
                start_h,
                dose_mg,
                p,
                wear_h,
            } => {
                let tau = time_h - start_h;
                if tau < 0.0 {
                    return 0.0;
                }

                if p.rate_mg_h > 0.0 {
                    // Zero-order
                    if tau <= wear_h {
                        p.rate_mg_h / p.k3 * (1.0 - (-p.k3 * tau).exp())
                    } else {
                        let amt_removal = p.rate_mg_h / p.k3 * (1.0 - (-p.k3 * wear_h).exp());
                        amt_removal * (-p.k3 * (tau - wear_h)).exp()
                    }
                } else {
                    // First-order legacy
                    let amt_under_patch = one_comp_amount(tau, dose_mg, p);
                    if tau > wear_h {
                        let amt_at_removal = one_comp_amount(wear_h, dose_mg, p);
                        amt_at_removal * (-p.k3 * (tau - wear_h)).exp()
                    } else {
                        amt_under_patch
                    }
                }
            }
        }
    }
}

pub fn run_simulation(events: &[DoseEvent], body_weight_kg: f64) -> Option<SimulationResult> {
    if events.is_empty() || body_weight_kg <= 0.0 {
        return None;
    }

    let mut sorted = events.to_vec();
    sorted.sort_by(|a, b| a.time_h.total_cmp(&b.time_h));

    // Oyama TS baseline:
    // start = first - 24h
    // end   = last + 14d
    // steps = 1000
    let grid = SimulationGrid {
        start_h: sorted[0].time_h - 24.0,
        end_h: sorted[sorted.len() - 1].time_h + 24.0 * 14.0,
        steps: 1000,
    };
    run_simulation_with_grid(&sorted, body_weight_kg, grid)
}

pub fn run_simulation_with_grid(
    events_sorted_by_time: &[DoseEvent],
    body_weight_kg: f64,
    grid: SimulationGrid,
) -> Option<SimulationResult> {
    if events_sorted_by_time.is_empty() || body_weight_kg <= 0.0 {
        return None;
    }
    if !grid.start_h.is_finite() || !grid.end_h.is_finite() || grid.end_h <= grid.start_h {
        return None;
    }
    if grid.steps < 2 {
        return None;
    }

    // NOTE: caller contract: events_sorted_by_time should already be sorted asc.
    let models: Vec<(EventModel, Ester)> = events_sorted_by_time
        .iter()
        .filter_map(|e| EventModel::new(e, events_sorted_by_time).map(|m| (m, e.ester)))
        .collect();

    let step_size = (grid.end_h - grid.start_h) / (grid.steps as f64 - 1.0);
    let mut all_times: Vec<f64> = (0..grid.steps)
        .map(|i| grid.start_h + (i as f64) * step_size)
        .collect();
    // Oyama TS baseline: union with exact event times
    all_times.extend(events_sorted_by_time.iter().map(|e| e.time_h));
    all_times.sort_by(|a, b| a.total_cmp(b));
    all_times.dedup_by(|a, b| (*a - *b).abs() < 1e-12);

    let plasma_ml_e2 = CORE_PK.vd_per_kg_e2_l * body_weight_kg * 1000.0;
    let plasma_ml_cpa = CORE_PK.vd_per_kg_cpa_l * body_weight_kg * 1000.0;

    let mut time_h = Vec::with_capacity(all_times.len());
    let mut conc_pg_ml = Vec::with_capacity(all_times.len());
    let mut conc_pg_ml_e2 = Vec::with_capacity(all_times.len());
    let mut conc_ng_ml_cpa = Vec::with_capacity(all_times.len());

    let mut auc = 0.0;

    for (i, &t) in all_times.iter().enumerate() {
        let mut total_mg_e2 = 0.0;
        let mut total_mg_cpa = 0.0;

        for (m, ester) in &models {
            let amount = m.amount_mg(t);
            if *ester == Ester::CPA {
                total_mg_cpa += amount;
            } else {
                total_mg_e2 += amount;
            }
        }

        // Oyama units:
        // - E2: pg/mL (mg -> pg => 1e9)
        // - CPA: ng/mL (mg -> ng => 1e6)
        let current_e2_pg_ml = (total_mg_e2 * 1e9) / plasma_ml_e2;
        let current_cpa_ng_ml = (total_mg_cpa * 1e6) / plasma_ml_cpa;
        // Total in pg/mL for compatibility
        let current_total_pg_ml = current_e2_pg_ml + current_cpa_ng_ml * 1000.0;

        time_h.push(t);
        conc_pg_ml.push(current_total_pg_ml);
        conc_pg_ml_e2.push(current_e2_pg_ml);
        conc_ng_ml_cpa.push(current_cpa_ng_ml);

        if i > 0 {
            let dt = t - all_times[i - 1];
            auc += 0.5 * (current_total_pg_ml + conc_pg_ml[i - 1]) * dt;
        }
    }

    Some(SimulationResult {
        time_h,
        conc_pg_ml,
        conc_pg_ml_e2,
        conc_ng_ml_cpa,
        auc,
    })
}
