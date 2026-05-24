use std::collections::HashMap;

/// A single lab result for calibration.
#[derive(Clone)]
pub struct LabResult {
    pub time_h: f64,
    pub conc_value: f64,
    /// "pg/ml" | "pmol/l"
    pub unit: String,
    /// Drug group this lab result applies to (e.g. "E2", "CPA", or "" for all)
    pub group_id: String,
}

/// Convert a lab value to pg/mL (internal calibration unit).
fn convert_to_pg_ml(val: f64, unit: &str) -> f64 {
    if unit == "pmol/l" {
        val / 3.671
    } else {
        val
    }
}

// ─── Linear interpolation helper ──────────────────────────────────

/// Linearly interpolate concentration at a given time from a sorted time-concentration series.
fn interpolate_conc(time_h: &[f64], conc: &[f64], target: f64) -> Option<f64> {
    if time_h.is_empty() { return None; }
    if target <= time_h[0] { return Some(conc[0]); }
    if target >= time_h[time_h.len() - 1] { return Some(conc[conc.len() - 1]); }

    let mut low = 0;
    let mut high = time_h.len() - 1;
    while high - low > 1 {
        let mid = (low + high) / 2;
        if (time_h[mid] - target).abs() < 1e-9 { return Some(conc[mid]); }
        if time_h[mid] < target { low = mid; } else { high = mid; }
    }
    let t0 = time_h[low];
    let t1 = time_h[high];
    if (t1 - t0).abs() < 1e-12 { return Some(conc[low]); }
    let frac = (target - t0) / (t1 - t0);
    Some(conc[low] + (conc[high] - conc[low]) * frac)
}

// ─── Ratio Interpolator ───────────────────────────────────────────

/// Build a lightweight ratio-based calibration interpolator from lab results.
/// Returns a vector of (timeH, ratio) sorted by timeH.
/// Both sim_conc and lab values are expected to already be in pg/mL.
pub fn build_ratio_interpolator(
    sim_time_h: &[f64],
    sim_conc: &[f64],
    labs: &[LabResult],
) -> Vec<(f64, f64)> {
    if sim_time_h.is_empty() || labs.is_empty() {
        return vec![(0.0, 1.0)];
    }

    let eps = 0.01; // 0.01 pg/mL minimum

    let mut points: Vec<(f64, f64)> = labs
        .iter()
        .filter_map(|lab| {
            let obs = convert_to_pg_ml(lab.conc_value, &lab.unit);
            // Use linear interpolation for predicted concentration (matching HRT Tracker)
            let pred = interpolate_conc(sim_time_h, sim_conc, lab.time_h);
            if pred.is_none() || pred.unwrap() < eps || obs <= 0.0 {
                return None;
            }
            let ratio = (obs / pred.unwrap()).clamp(0.1, 10.0);
            Some((lab.time_h, ratio))
        })
        .collect();

    points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    if points.is_empty() {
        return vec![(0.0, 1.0)];
    }

    points
}

/// Evaluate the ratio at a given timeH using piecewise-linear interpolation.
pub fn eval_ratio_interpolator(points: &[(f64, f64)], time_h: f64) -> f64 {
    if points.is_empty() {
        return 1.0;
    }
    if points.len() == 1 {
        return points[0].1;
    }
    if time_h <= points[0].0 {
        return points[0].1;
    }
    if time_h >= points.last().unwrap().0 {
        return points.last().unwrap().1;
    }

    let mut low = 0;
    let mut high = points.len() - 1;
    while high - low > 1 {
        let mid = (low + high) / 2;
        if (points[mid].0 - time_h).abs() < 1e-9 {
            return points[mid].1;
        }
        if points[mid].0 < time_h {
            low = mid;
        } else {
            high = mid;
        }
    }

    let left = &points[low];
    let right = &points[high];
    let t = (time_h - left.0) / (right.0 - left.0);
    (left.1 + (right.1 - left.1) * t).clamp(0.1, 10.0)
}

// ─── OU-Kalman Filter + RTS Smoother ─────────────────────────────

pub struct OUKalmanParams {
    pub tau: f64,
    pub theta: f64,
    pub sigma: f64,
    pub mu: f64,
}

impl Default for OUKalmanParams {
    fn default() -> Self {
        OUKalmanParams {
            tau: 0.198,
            theta: std::f64::consts::LN_2 / (7.0 * 24.0),
            sigma: 0.02,
            mu: 0.0,
        }
    }
}

/// Result of OU-Kalman calibration: smoothed mean (log-ratio) and variance.
pub struct OUKalmanResult {
    pub m: Vec<f64>,
    pub p: Vec<f64>,
}

/// Build OU-Kalman calibration: forward filter + RTS backward smoother.
/// Output is aligned with `sim_time_h`.
/// Both sim_conc and lab values are expected to already be in pg/mL.
pub fn build_ou_kalman(
    sim_time_h: &[f64],
    sim_conc: &[f64],
    labs: &[LabResult],
    params: &OUKalmanParams,
) -> OUKalmanResult {
    let n = sim_time_h.len();
    if n == 0 {
        return OUKalmanResult { m: vec![], p: vec![] };
    }

    let tau2 = params.tau * params.tau;
    let p_inf = (params.sigma * params.sigma) / (2.0 * params.theta);
    let eps = 0.01; // 0.01 pg/mL minimum

    let t_min = sim_time_h[0];
    let t_max = sim_time_h[n - 1];

    // Build lab observations (log-ratio of pg/mL values)
    let mut lab_obs: Vec<(f64, f64)> = labs
        .iter()
        .filter_map(|lab| {
            if lab.time_h < t_min || lab.time_h > t_max {
                return None;
            }
            let obs = convert_to_pg_ml(lab.conc_value, &lab.unit);
            if obs <= 0.0 {
                return None;
            }
            // Use linear interpolation (matching HRT Tracker's interpolateConcentration_E2)
            let c0 = interpolate_conc(sim_time_h, sim_conc, lab.time_h);
            if c0.is_none() || c0.unwrap() < eps {
                return None;
            }
            // Both obs and c0 are in pg/mL now, so log-ratio is meaningful
            let z = obs.ln() - c0.unwrap().ln();
            if !z.is_finite() || z.abs() > 3.5 {
                return None;
            }
            Some((lab.time_h, z))
        })
        .collect();
    lab_obs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    // Build merged grid (sim times + lab times)
    let mut grid_set: HashMap<u64, f64> = HashMap::new();
    for &t in sim_time_h {
        grid_set.insert(t.to_bits(), t);
    }
    for &(t, _) in &lab_obs {
        grid_set.insert(t.to_bits(), t);
    }
    let mut grid: Vec<f64> = grid_set.values().copied().collect();
    grid.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let grid_n = grid.len();
    let mut grid_index: HashMap<u64, usize> = HashMap::new();
    for (i, &t) in grid.iter().enumerate() {
        grid_index.insert(t.to_bits(), i);
    }

    // Forward pass
    let mut m_fwd = vec![params.mu; grid_n];
    let mut p_fwd = vec![p_inf; grid_n];
    let mut m_pred = vec![params.mu; grid_n];
    let mut p_pred = vec![p_inf; grid_n];

    let mut m = params.mu;
    let mut p = p_inf;
    let mut lab_ptr = 0;

    for i in 0..grid_n {
        if i > 0 {
            let dt = grid[i] - grid[i - 1];
            if dt > 0.0 {
                let phi = (-params.theta * dt).exp();
                let q = p_inf * (1.0 - phi * phi);
                m = params.mu + phi * (m - params.mu);
                p = phi * phi * p + q;
            }
        }

        m_pred[i] = m;
        p_pred[i] = p;

        while lab_ptr < lab_obs.len() && (lab_obs[lab_ptr].0 - grid[i]).abs() < 1e-9 {
            let s = p + tau2;
            let k = p / s;
            m += k * (lab_obs[lab_ptr].1 - m);
            p = (1.0 - k) * p;
            lab_ptr += 1;
        }

        m_fwd[i] = m;
        p_fwd[i] = p.max(1e-12);
    }

    // RTS backward smoother
    let mut m_smooth = m_fwd.clone();
    let mut p_smooth = p_fwd.clone();

    for i in (0..grid_n - 1).rev() {
        let dt = grid[i + 1] - grid[i];
        if dt <= 0.0 {
            continue;
        }
        let phi = (-params.theta * dt).exp();
        let gain = if p_pred[i + 1] > 1e-12 {
            p_fwd[i] * phi / p_pred[i + 1]
        } else {
            0.0
        };
        m_smooth[i] = m_fwd[i] + gain * (m_smooth[i + 1] - m_pred[i + 1]);
        p_smooth[i] = (p_fwd[i] + gain * gain * (p_smooth[i + 1] - p_pred[i + 1])).max(1e-9);
    }

    // Map back to original sim time grid
    let mut out_m = vec![params.mu; n];
    let mut out_p = vec![p_inf; n];
    for i in 0..n {
        if let Some(&idx) = grid_index.get(&sim_time_h[i].to_bits()) {
            out_m[i] = m_smooth[idx];
            out_p[i] = p_smooth[idx];
        }
    }

    OUKalmanResult { m: out_m, p: out_p }
}

/// Apply the OU-Kalman calibration to the concentration array.
/// Returns calibrated concentrations = sim_conc * exp(m[i] + 0.5*p[i]) (log-normal mean).
pub fn apply_ou_kalman_to_conc(
    sim_time_h: &[f64],
    sim_conc: &[f64],
    labs: &[LabResult],
    params: &OUKalmanParams,
) -> Vec<f64> {
    let result = build_ou_kalman(sim_time_h, sim_conc, labs, params);
    let n = sim_conc.len();
    let mut calibrated = vec![0.0; n];
    for i in 0..n {
        if i < result.m.len() {
            // Log-normal mean: E[exp(X)] = exp(μ + σ²/2)
            calibrated[i] = sim_conc[i] * (result.m[i] + 0.5 * result.p[i]).exp();
        } else {
            calibrated[i] = sim_conc[i];
        }
    }
    calibrated
}

// ─── Calibration Band (with confidence intervals) ──────────────────

/// Structured calibration band with confidence intervals.
pub struct CalibrationBand {
    /// Calibrated (point estimate) concentrations in the same unit as sim_conc.
    pub calibrated: Vec<f64>,
    /// 95% CI lower bound.
    pub ci95_low: Vec<f64>,
    /// 95% CI upper bound.
    pub ci95_high: Vec<f64>,
    /// 68% CI lower bound (≈ ±1σ).
    pub ci68_low: Vec<f64>,
    /// 68% CI upper bound.
    pub ci68_high: Vec<f64>,
}

/// Generate calibration band with confidence intervals.
/// Both sim_conc and lab values are expected to already be in the same unit (pg/mL).
pub fn generate_calibration_band(
    model: &str,
    sim_time_h: &[f64],
    sim_conc: &[f64],
    labs: &[LabResult],
) -> Option<CalibrationBand> {
    if sim_time_h.is_empty() || labs.is_empty() {
        return None;
    }

    let n = sim_conc.len();
    let eps = 0.01;

    if model == "ou-kalman" {
        let params = OUKalmanParams::default();
        let result = build_ou_kalman(sim_time_h, sim_conc, labs, &params);

        let mut calibrated = vec![0.0; n];
        let mut ci95_low = vec![0.0; n];
        let mut ci95_high = vec![0.0; n];
        let mut ci68_low = vec![0.0; n];
        let mut ci68_high = vec![0.0; n];

        for i in 0..n {
            let c0 = sim_conc[i].max(eps);
            let m = result.m.get(i).copied().unwrap_or(params.mu);
            let p = result.p.get(i).copied().unwrap_or(0.0);
            let std = p.sqrt().max(0.0);

            // Log-normal mean: exp(μ + σ²/2)
            calibrated[i] = c0 * (m + 0.5 * p).exp();

            // 95% CI
            ci95_low[i] = c0 * (m - 1.96 * std).exp();
            ci95_high[i] = c0 * (m + 1.96 * std).exp();

            // 68% CI (±1σ)
            ci68_low[i] = c0 * (m - std).exp();
            ci68_high[i] = c0 * (m + std).exp();
        }

        Some(CalibrationBand { calibrated, ci95_low, ci95_high, ci68_low, ci68_high })
    } else {
        // ratio interpolator (no CI — just point estimate)
        let points = build_ratio_interpolator(sim_time_h, sim_conc, labs);
        let mut calibrated = vec![0.0; n];
        for i in 0..n {
            let ratio = eval_ratio_interpolator(&points, sim_time_h[i]);
            calibrated[i] = (sim_conc[i] * ratio).max(0.0);
        }
        let ci = calibrated.clone();
        Some(CalibrationBand { calibrated, ci95_low: ci.clone(), ci95_high: ci.clone(), ci68_low: ci.clone(), ci68_high: ci.clone() })
    }
}
