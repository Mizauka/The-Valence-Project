use crate::db::{Database, L1Substance, L2Route, L3Dosage, Range};
use anyhow::{Result, bail};

#[derive(Debug, Clone)]
pub struct PkParams {
    pub ka_per_hour: f64,
    pub ke_per_hour: f64,
    pub f: f64,
    pub kappa: f64,
    pub c0: f64,
}

#[derive(Debug, Clone)]
pub enum FixedIntervalMode {
    /// 有限次给药：从 t=0 开始每 tau_hours 给药一次，共 doses 次。
    Finite { tau_hours: f64, doses: usize },
    /// 稳态：假设从无穷久以前开始按间隔 tau_hours 重复给药。
    SteadyState { tau_hours: f64 },
}

#[derive(Debug, Clone)]
pub struct CurvePoint {
    pub t_hours: f64,
    pub c: f64,
}

#[derive(Debug, Clone)]
pub struct Curve {
    pub params: PkParams,
    pub points: Vec<CurvePoint>,
    pub t_max_hours: Option<f64>,
    pub c_max: Option<f64>,
}

#[derive(Debug, Clone)]
pub enum KeSource {
    HalfLifeHours(f64),
    ExplicitKePerHour(f64),
    EstimatedFromTotalDuration {
        total_max_hours: f64,
        assumed_half_lives: f64,
        estimated_half_life_hours: f64,
    },
}

#[derive(Debug, Clone)]
pub struct BuildPkOptions {
    pub override_ka_per_hour: Option<f64>,
    pub override_ke_per_hour: Option<f64>,
    pub override_half_life_hours: Option<f64>,
    pub estimate_ke_from_l3_total_max: bool,
}

impl Default for BuildPkOptions {
    fn default() -> Self {
        Self {
            override_ka_per_hour: None,
            override_ke_per_hour: None,
            override_half_life_hours: None,
            estimate_ke_from_l3_total_max: false,
        }
    }
}

pub fn curve_for_substance_route(
    _db: &Database,
    substance: &L1Substance,
    route: &L2Route,
    l3: Option<&L3Dosage>,
    dose: f64,
    dt_minutes: f64,
    total_hours: f64,
    opts: &BuildPkOptions,
) -> Result<(Curve, Option<KeSource>)> {
    if dose <= 0.0 {
        bail!("dose 必须 > 0");
    }
    if dt_minutes <= 0.0 {
        bail!("dt_minutes 必须 > 0");
    }
    if total_hours <= 0.0 {
        bail!("total_hours 必须 > 0");
    }

    let ka = opts
        .override_ka_per_hour
        .unwrap_or(route.default_ka_per_hour);
    if ka <= 0.0 {
        bail!("ka 必须 > 0");
    }

    let (ke, ke_source) = resolve_ke(substance, l3, opts)?;
    if ke <= 0.0 {
        bail!("ke 必须 > 0");
    }

    // pk_model_1.pdf 在推导中假设 ka > ke；但在真实数据里（尤其经皮/缓释）可能出现 flip-flop
    //（ka <= ke）。只要使用正确的 kappa=ka/(ka-ke) 并处理 ka≈ke 的极限形式，模型仍然可用。
    let kappa = compute_kappa(ka, ke);

    let params = PkParams {
        ka_per_hour: ka,
        ke_per_hour: ke,
        f: route.default_f,
        // 相对浓度：不引入 Vd，但仍保留 Bateman 的系数结构（kappa=ka/(ka-ke)）。
        kappa,
        c0: 0.0,
    };

    let dt_hours = dt_minutes / 60.0;
    let n = (total_hours / dt_hours).ceil() as usize;
    let mut points = Vec::with_capacity(n + 1);

    let mut c_max = None;
    let mut t_max = None;
    for i in 0..=n {
        let t = (i as f64) * dt_hours;
        let c = bateman_concentration(t, dose, &params);
        points.push(CurvePoint { t_hours: t, c });
        if c.is_finite() && c >= 0.0 {
            if c_max.map(|m| c > m).unwrap_or(true) {
                c_max = Some(c);
                t_max = Some(t);
            }
        }
    }

    Ok((
        Curve {
            params,
            points,
            t_max_hours: t_max,
            c_max,
        },
        ke_source,
    ))
}

pub fn curve_fixed_interval(
    _db: &Database,
    substance: &L1Substance,
    route: &L2Route,
    l3: Option<&L3Dosage>,
    dose: f64,
    dt_minutes: f64,
    total_hours: f64,
    mode: FixedIntervalMode,
    opts: &BuildPkOptions,
) -> Result<(Curve, Option<KeSource>)> {
    if dose <= 0.0 {
        bail!("dose 必须 > 0");
    }
    if dt_minutes <= 0.0 {
        bail!("dt_minutes 必须 > 0");
    }
    if total_hours <= 0.0 {
        bail!("total_hours 必须 > 0");
    }

    let ka = opts
        .override_ka_per_hour
        .unwrap_or(route.default_ka_per_hour);
    if ka <= 0.0 {
        bail!("ka 必须 > 0");
    }

    let (ke, ke_source) = resolve_ke(substance, l3, opts)?;
    if ke <= 0.0 {
        bail!("ke 必须 > 0");
    }

    let kappa = compute_kappa(ka, ke);
    let params = PkParams {
        ka_per_hour: ka,
        ke_per_hour: ke,
        f: route.default_f,
        kappa,
        c0: 0.0,
    };

    let dt_hours = dt_minutes / 60.0;
    let n = (total_hours / dt_hours).ceil() as usize;
    let mut points = Vec::with_capacity(n + 1);

    let mut c_max = None;
    let mut t_max = None;
    for i in 0..=n {
        let t = (i as f64) * dt_hours;
        let c = match mode {
            FixedIntervalMode::Finite { tau_hours, doses } => {
                concentration_fixed_interval_finite(t, dose, &params, tau_hours, doses)
            }
            FixedIntervalMode::SteadyState { tau_hours } => {
                concentration_fixed_interval_steady_state(t, dose, &params, tau_hours)
            }
        };

        points.push(CurvePoint { t_hours: t, c });
        if c.is_finite() && c >= 0.0 {
            if c_max.map(|m| c > m).unwrap_or(true) {
                c_max = Some(c);
                t_max = Some(t);
            }
        }
    }

    Ok((
        Curve {
            params,
            points,
            t_max_hours: t_max,
            c_max,
        },
        ke_source,
    ))
}

pub fn bateman_concentration(t_hours: f64, dose: f64, p: &PkParams) -> f64 {
    // 一室模型 + 一级吸收（Bateman）
    // pk_model_1.pdf 推导的结构等价于：
    //   C(t) = C0*exp(-ke*t) + Dose * F * kappa * (exp(-ke*t) - exp(-ka*t))
    // 其中 kappa = ka/(ka-ke)。当 ka<ke（flip-flop）时该式仍为正（kappa 与括号同号）。
    if t_hours < 0.0 {
        return 0.0;
    }
    let ke = p.ke_per_hour;
    let ka = p.ka_per_hour;
    let c0_term = p.c0 * (-ke * t_hours).exp();
    let dose_term = dose * p.f * bateman_unit_response(t_hours, ka, ke);
    (c0_term + dose_term).max(0.0)
}

/// 单次给药的“单位剂量响应”（不含 Dose/F），用于稳态/重复给药求和。
///
/// 对应：kappa * (e^{-ke t} - e^{-ka t})，其中 kappa=ka/(ka-ke)。
fn bateman_unit_response(t_hours: f64, ka: f64, ke: f64) -> f64 {
    debug_assert!(t_hours >= 0.0);
    const EPS: f64 = 1e-10;
    if (ka - ke).abs() < EPS {
        // 极限：lim_{ka->ke} ka/(ka-ke)*(e^{-ke t} - e^{-ka t}) = ka*t*e^{-ka t}
        return ka * t_hours * (-ka * t_hours).exp();
    }
    let kappa = ka / (ka - ke);
    kappa * ((-ke * t_hours).exp() - (-ka * t_hours).exp())
}

fn compute_kappa(ka: f64, ke: f64) -> f64 {
    const EPS: f64 = 1e-10;
    if (ka - ke).abs() < EPS {
        // 使用极限形式时 kappa 不再单独有意义；保留为 ka 以便调试输出可读。
        return ka;
    }
    ka / (ka - ke)
}

fn concentration_fixed_interval_finite(
    t_hours: f64,
    dose: f64,
    p: &PkParams,
    tau_hours: f64,
    doses: usize,
) -> f64 {
    if tau_hours <= 0.0 || doses == 0 {
        return 0.0;
    }
    let mut c = 0.0;
    // 线性系统，直接叠加每次给药的 Bateman 响应
    for n in 0..doses {
        let tn = (n as f64) * tau_hours;
        let dt = t_hours - tn;
        if dt < 0.0 {
            break;
        }
        c += bateman_concentration(dt, dose, p);
    }
    c
}

fn concentration_fixed_interval_steady_state(t_hours: f64, dose: f64, p: &PkParams, tau_hours: f64) -> f64 {
    if tau_hours <= 0.0 {
        return 0.0;
    }

    // 与 pk_model_1.pdf 的 Css(t) 等价（在 tn=0 的固定间隔给药场景下）。
    // 对于 t，取其在一个周期内的位置：phi = t mod tau。
    let phi = t_hours.rem_euclid(tau_hours);
    let ka = p.ka_per_hour;
    let ke = p.ke_per_hour;

    const EPS: f64 = 1e-10;
    if (ka - ke).abs() < EPS {
        // ka≈ke 时解析式更复杂；用截断级数求和（收敛很快）。
        let mut sum = 0.0;
        let mut n = 0usize;
        loop {
            let dt = phi + (n as f64) * tau_hours;
            let term = bateman_unit_response(dt, ka, ke);
            sum += term;
            // 单调衰减（指数），阈值足够小即可停
            if term.abs() < 1e-12 || n >= 10000 {
                break;
            }
            n += 1;
        }
        return (dose * p.f * sum).max(0.0);
    }

    let denom_ke = 1.0 - (-ke * tau_hours).exp();
    let denom_ka = 1.0 - (-ka * tau_hours).exp();
    if denom_ke.abs() < 1e-12 || denom_ka.abs() < 1e-12 {
        return 0.0;
    }

    // 无限次重复给药的几何级数求和
    let kappa = ka / (ka - ke);
    let term_ke = (-ke * phi).exp() / denom_ke;
    let term_ka = (-ka * phi).exp() / denom_ka;
    let c = dose * p.f * kappa * (term_ke - term_ka);
    c.max(0.0)
}

pub fn peak_range_hours(l3: &L3Dosage) -> Option<(f64, f64)> {
    to_hours_range(l3.peak.as_ref())
}

pub fn total_range_hours(l3: &L3Dosage) -> Option<(f64, f64)> {
    to_hours_range(l3.total.as_ref())
}

fn to_hours_range(r: Option<&Range>) -> Option<(f64, f64)> {
    let r = r?;
    let (min, max) = (r.min, r.max);
    if min <= 0.0 || max <= 0.0 || max < min {
        return None;
    }
    let u = r.units.to_ascii_lowercase();
    let mult = match u.as_str() {
        "minutes" | "minute" | "min" => 1.0 / 60.0,
        "hours" | "hour" | "h" => 1.0,
        "days" | "day" => 24.0,
        _ => return None,
    };
    Some((min * mult, max * mult))
}

fn resolve_ke(
    substance: &L1Substance,
    l3: Option<&L3Dosage>,
    opts: &BuildPkOptions,
) -> Result<(f64, Option<KeSource>)> {
    if let Some(ke) = opts.override_ke_per_hour {
        return Ok((ke, Some(KeSource::ExplicitKePerHour(ke))));
    }
    if let Some(hl) = opts.override_half_life_hours {
        let ke = half_life_to_ke(hl)?;
        return Ok((ke, Some(KeSource::HalfLifeHours(hl))));
    }
    if let Some(hl) = substance.half_life_hours {
        let ke = half_life_to_ke(hl)?;
        return Ok((ke, Some(KeSource::HalfLifeHours(hl))));
    }

    if opts.estimate_ke_from_l3_total_max {
        if let Some(l3) = l3 {
            if let Some((_min_h, max_h)) = total_range_hours(l3) {
                // 这是“假设”，用于 CLI 验证场景：把 total.max 视作约 5 个半衰期后的接近清除
                let assumed_half_lives = 5.0;
                let est_half_life = max_h / assumed_half_lives;
                let ke = half_life_to_ke(est_half_life)?;
                return Ok((
                    ke,
                    Some(KeSource::EstimatedFromTotalDuration {
                        total_max_hours: max_h,
                        assumed_half_lives,
                        estimated_half_life_hours: est_half_life,
                    }),
                ));
            }
        }
    }

    bail!(
        "无法得到 ke：L1.half_life_hours 为空。请用 --half-life-hours 或 --ke-per-hour，或启用 --estimate-ke-from-total。"
    );
}

fn half_life_to_ke(half_life_hours: f64) -> Result<f64> {
    if half_life_hours <= 0.0 {
        bail!("half_life_hours 必须 > 0");
    }
    Ok(std::f64::consts::LN_2 / half_life_hours)
}
