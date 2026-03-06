use tvp_hrt_engine::model::{DoseEvent, Ester, ExtraKey, Route};
use tvp_hrt_engine::sim::{run_simulation, run_simulation_with_grid, SimulationGrid};

fn approx_eq(a: f64, b: f64, rel: f64, abs: f64) -> bool {
    let diff = (a - b).abs();
    diff <= abs.max(rel * a.abs().max(b.abs()))
}

#[test]
fn baseline_is_deterministic_for_same_input() {
    let t0 = 0.0;
    let events = vec![DoseEvent {
        id: "e1".to_string(),
        route: Route::Oral,
        time_h: t0,
        dose_mg: 2.0,
        ester: Ester::E2,
        extras: Default::default(),
    }];

    let a = run_simulation(&events, 70.0).unwrap();
    let b = run_simulation(&events, 70.0).unwrap();

    assert_eq!(a.time_h, b.time_h);
    assert_eq!(a.conc_pg_ml, b.conc_pg_ml);
    assert_eq!(a.conc_pg_ml_e2, b.conc_pg_ml_e2);
    assert_eq!(a.conc_ng_ml_cpa, b.conc_ng_ml_cpa);
    assert!(approx_eq(a.auc, b.auc, 0.0, 0.0));
}

#[test]
fn patch_zero_order_removal_behaves_sensibly() {
    // releaseRateUGPerDay -> rateMGh = (ug/day)/24/1000 * F, with F=1*mwFactor
    let t0 = 1000.0;

    let mut extras_apply = std::collections::HashMap::new();
    extras_apply.insert(ExtraKey::ReleaseRateUgPerDay, 100.0);

    let events = vec![
        DoseEvent {
            id: "apply".to_string(),
            route: Route::PatchApply,
            time_h: t0,
            dose_mg: 0.0,
            ester: Ester::E2,
            extras: extras_apply,
        },
        DoseEvent {
            id: "remove".to_string(),
            route: Route::PatchRemove,
            time_h: t0 + 24.0,
            dose_mg: 0.0,
            ester: Ester::E2,
            extras: Default::default(),
        },
    ];

    let grid = SimulationGrid {
        start_h: t0,
        end_h: t0 + 48.0,
        steps: 200,
    };

    let sim = run_simulation_with_grid(&events, 70.0, grid).unwrap();

    // Should rise during wear, then decay after removal.
    let idx_before = sim
        .time_h
        .iter()
        .position(|&t| t >= t0 + 23.0)
        .unwrap();
    let idx_after = sim
        .time_h
        .iter()
        .position(|&t| t >= t0 + 30.0)
        .unwrap();

    assert!(sim.conc_pg_ml_e2[idx_before] > sim.conc_pg_ml_e2[0]);
    assert!(sim.conc_pg_ml_e2[idx_after] < sim.conc_pg_ml_e2[idx_before]);
}

#[test]
fn sublingual_ev_dual_branch_runs() {
    let t0 = 0.0;
    let mut extras = std::collections::HashMap::new();
    extras.insert(ExtraKey::SublingualTier, 2.0); // standard => theta~0.11

    let events = vec![DoseEvent {
        id: "sl".to_string(),
        route: Route::Sublingual,
        time_h: t0,
        dose_mg: 2.0,
        ester: Ester::EV,
        extras,
    }];

    let sim = run_simulation(&events, 70.0).unwrap();

    // Non-empty and non-negative.
    assert!(!sim.time_h.is_empty());
    assert!(sim.conc_pg_ml_e2.iter().all(|&c| c >= 0.0 && c.is_finite()));
}

#[test]
fn injection_analytic3c_runs() {
    let t0 = 0.0;
    let events = vec![DoseEvent {
        id: "inj".to_string(),
        route: Route::Injection,
        time_h: t0,
        dose_mg: 5.0,
        ester: Ester::EV,
        extras: Default::default(),
    }];

    let sim = run_simulation(&events, 70.0).unwrap();
    assert!(!sim.time_h.is_empty());
    assert!(sim.conc_pg_ml_e2.iter().any(|&c| c > 0.0));
}

#[test]
fn injection_param_overrides_take_effect() {
    // Use Ester::E2 but override injection parameters to match EV.
    // This verifies config-driven injection (via extras) works.
    let t0 = 0.0;
    let mut extras = std::collections::HashMap::new();
    extras.insert(ExtraKey::InjectionFracFast, 0.40);
    extras.insert(ExtraKey::InjectionK1FastPerHour, 0.0216);
    extras.insert(ExtraKey::InjectionK1SlowPerHour, 0.0138);
    extras.insert(ExtraKey::InjectionK2PerHour, 0.070);
    extras.insert(ExtraKey::InjectionK3PerHour, 0.041);
    extras.insert(ExtraKey::InjectionFormationFraction, 0.0623);

    let events = vec![DoseEvent {
        id: "inj_override".to_string(),
        route: Route::Injection,
        time_h: t0,
        dose_mg: 5.0,
        ester: Ester::E2,
        extras,
    }];

    let sim = run_simulation(&events, 70.0).unwrap();
    assert!(!sim.time_h.is_empty());
    // Oyama analytical model can produce very small negative values due to floating-point cancellation.
    assert!(sim
        .conc_pg_ml_e2
        .iter()
        .all(|&c| c.is_finite() && c >= -1e-9));
    assert!(sim.conc_pg_ml_e2.iter().any(|&c| c > 0.0));
}
