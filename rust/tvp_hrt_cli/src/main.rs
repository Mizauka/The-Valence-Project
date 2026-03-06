use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tvp_hrt_engine::config::HrtModel;
use tvp_hrt_engine::db::HrtDatabase;
use tvp_hrt_engine::model::{DoseEvent, Ester, ExtraKey, Route};
use tvp_hrt_engine::sim::{run_simulation, run_simulation_with_grid, SimulationGrid};

mod schema;

#[derive(Debug, Parser)]
#[command(name = "tvp-hrt", version, about = "TVP HRT CLI (Oyama-compatible PK)")]
struct Cli {
    #[arg(
        long,
        default_value = "assets/database/hrt",
        help = "HRT 数据库根目录（包含 substances/routes/dosages）。若相对路径无效会自动探测 ../assets/database/hrt 等"
    )]
    db_dir: String,

    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Debug, Subcommand)]
enum Cmd {
    /// Run Oyama-compatible simulation from an events JSON file.
    Simulate {
        /// Path to JSON array of DoseEvent (fields: id, route, timeH, doseMG, ester, extras)
        #[arg(long)]
        events: PathBuf,

        #[arg(long, default_value_t = 70.0)]
        body_weight_kg: f64,

        /// optional: compound name/id (alias for --compound)
        #[arg(long)]
        substance: Option<String>,
        /// optional: compound name/id
        #[arg(long)]
        compound: Option<String>,
        /// optional: route name/id
        #[arg(long)]
        route: Option<String>,
    },

    /// Load HRT 3-layer config (L1/L2/L3), build events, and run a curve.
    Curve {
        #[arg(long, help = "compound 名称或 id（例如 estradiol / cpa）")]
        compound: String,
        #[arg(long, help = "alias: substance 名称或 id（可选）")]
        substance: Option<String>,

        #[arg(long, help = "route 名称或 id（例如 oral / patch / sublingual / injection）")]
        route: String,

        #[arg(long, help = "剂量数值，单位取决于 L3.dose_units（例如 mg 或 ug/day）")]
        dose: f64,

        #[arg(long, default_value_t = 70.0)]
        body_weight_kg: f64,

        #[arg(long, default_value_t = 0.0, help = "给药开始时间（hour）。默认 0")]
        t0_h: f64,

        #[arg(long, default_value_t = 48.0, help = "输出曲线总时长（hour）")]
        total_h: f64,

        #[arg(long, default_value_t = 400, help = "网格步数（越大越细）")]
        steps: usize,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let db_dir = resolve_db_dir(&cli.db_dir)?;

    match cli.cmd {
        Cmd::Simulate {
            events,
            body_weight_kg,
            substance,
            compound,
            route,
        } => {
            let events_path = events;
            let text = std::fs::read_to_string(&events_path)
                .with_context(|| format!("failed to read {}", events_path.display()))?;
            let parsed_events: Vec<DoseEvent> = serde_json::from_str(&text)
                .with_context(|| format!("failed to parse events JSON: {}", events_path.display()))?;

            let sim = run_simulation(&parsed_events, body_weight_kg)
                .context("simulation returned null")?;

            // Unified envelope with meta/events shape.
            let events_obj = serde_json::json!({
                "meta": {
                    "events_path": events_path.to_string_lossy(),
                    "body_weight_kg": body_weight_kg,
                    "provided_compound": compound,
                    "provided_substance": substance,
                    "provided_route": route
                },
                "events": parsed_events
            });

            let mut env = schema::envelope::Envelope::<
                serde_json::Value,
                serde_json::Value,
                serde_json::Value,
                serde_json::Value,
                _,
            >::new(
                "tvp_hrt_engine.sim",
                "simulate",
                db_dir.to_string(),
                events_obj,
                sim,
            );
            env.subject_kind = Some("compound");
            // We cannot reliably infer a single compound/route from arbitrary events file.
            println!("{}", serde_json::to_string_pretty(&env)?);
        }

        Cmd::Curve {
            compound,
            substance,
            route,
            dose,
            body_weight_kg,
            t0_h,
            total_h,
            steps,
        } => {
            let db = HrtDatabase::load_from_assets_dir(&db_dir)
                .with_context(|| format!("加载 HRT 数据库失败：{}", db_dir))?;

            let compound_name = if let Some(sup) = &substance { sup.as_str() } else { &compound };
            let c = db
                .compound_by_name_or_id(compound_name)
                .with_context(|| format!("找不到 compound：{}", compound_name))?;
            let r = db
                .route_by_id_or_name(&route)
                .with_context(|| format!("找不到 route：{}", route))?;

            let d = db
                .dosage(&c.id, &r.id)
                .with_context(|| format!("找不到 dosage：compound={} route={}", c.id, r.id))?;

            // NOTE: 目前只把 3-layer 配置映射为事件集，然后复用 Oyama-compatible run_simulation。
            // 将来会把 HrtModel 直接作为模拟后端（不依赖 Route/Ester 分支），但这里先建立可验证的最小闭环。

            let mut events: Vec<DoseEvent> = Vec::new();

            match &d.model {
                HrtModel::OneCompFirstOrder { .. } => {
                    // 选择与 d.route_id 对齐的 Route。Ester：目前用 E2/CPA 两个 compound id 做映射。
                    let route_enum = match d.route_id.as_str() {
                        "oral" => Route::Oral,
                        "sublingual" => Route::Sublingual,
                        "injection" => Route::Injection,
                        "patch" => Route::PatchApply,
                        _ => {
                            anyhow::bail!("unsupported route_id in HRT config: {}", d.route_id)
                        }
                    };
                    let ester = compound_id_to_ester(&d.compound_id);

                    events.push(DoseEvent {
                        id: "dose".to_string(),
                        route: route_enum,
                        time_h: t0_h,
                        dose_mg: if d.dose_units.to_ascii_lowercase() == "mg" {
                            dose
                        } else {
                            // 对于不是 mg 的 OneComp 配置，这里先拒绝，避免隐式单位错误。
                            anyhow::bail!(
                                "dose_units must be mg for OneCompFirstOrder, got {}",
                                d.dose_units
                            )
                        },
                        ester,
                        extras: Default::default(),
                    });
                }

                HrtModel::ZeroOrderPatch {
                    rate_mg_per_hour,
                    wear_hours,
                    ..
                } => {
                    // 贴片：用 patchApply + extras.releaseRateUGPerDay 走 Oyama 零阶分支。
                    // 这里支持两种输入：
                    // - dose_units == ug/day：dose 就是 ug/day
                    // - dose_units == mg/h：dose 就是 mg/h（会转换成 ug/day 供 extras）
                    let ester = compound_id_to_ester(&d.compound_id);
                    let release_ug_per_day = if d.dose_units.to_ascii_lowercase() == "ug/day" {
                        dose
                    } else if d.dose_units.to_ascii_lowercase() == "mg/h" {
                        dose * 24.0 * 1000.0
                    } else {
                        // 作为兜底：如果配置里已经给了 rate_mg_per_hour，则用它；否则拒绝。
                        if *rate_mg_per_hour > 0.0 {
                            rate_mg_per_hour * 24.0 * 1000.0
                        } else {
                            anyhow::bail!("unsupported patch dose_units: {}", d.dose_units)
                        }
                    };

                    let mut extras = std::collections::HashMap::new();
                    extras.insert(ExtraKey::ReleaseRateUgPerDay, release_ug_per_day);

                    events.push(DoseEvent {
                        id: "patch_apply".to_string(),
                        route: Route::PatchApply,
                        time_h: t0_h,
                        dose_mg: 0.0,
                        ester,
                        extras,
                    });

                    events.push(DoseEvent {
                        id: "patch_remove".to_string(),
                        route: Route::PatchRemove,
                        time_h: t0_h + *wear_hours,
                        dose_mg: 0.0,
                        ester,
                        extras: Default::default(),
                    });
                }

                HrtModel::DualBranch { frac_fast, .. } => {
                    // 先用 Oyama 的 sublingual 路径：用 extras.sublingualTheta 驱动分支占比。
                    let mut extras = std::collections::HashMap::new();
                    extras.insert(ExtraKey::SublingualTheta, *frac_fast);

                    events.push(DoseEvent {
                        id: "sl".to_string(),
                        route: Route::Sublingual,
                        time_h: t0_h,
                        dose_mg: if d.dose_units.to_ascii_lowercase() == "mg" {
                            dose
                        } else {
                            anyhow::bail!("dose_units must be mg for DualBranch, got {}", d.dose_units)
                        },
                        ester: compound_id_to_ester(&d.compound_id),
                        extras,
                    });
                }

                HrtModel::InjectionTwoDepot3C { frac_fast, fast, slow } => {
                    let ester = compound_id_to_ester(&d.compound_id);

                    if d.dose_units.to_ascii_lowercase() != "mg" {
                        anyhow::bail!(
                            "dose_units must be mg for InjectionTwoDepot3C, got {}",
                            d.dose_units
                        )
                    }

                    let (k1_fast, k2_fast, k3_fast, f_fast) =
                        extract_three_comp_params(fast).context("invalid fast model")?;
                    let (k1_slow, k2_slow, k3_slow, f_slow) =
                        extract_three_comp_params(slow).context("invalid slow model")?;

                    // In Oyama baseline injection route: both depots use same k2/k3 and same formationFraction (inside F).
                    // We enforce that here to avoid silent divergence.
                    if (k2_fast - k2_slow).abs() > 1e-9 {
                        anyhow::bail!(
                            "InjectionTwoDepot3C requires fast.k2 == slow.k2 (got {} vs {})",
                            k2_fast,
                            k2_slow
                        );
                    }
                    if (k3_fast - k3_slow).abs() > 1e-9 {
                        anyhow::bail!(
                            "InjectionTwoDepot3C requires fast.k3 == slow.k3 (got {} vs {})",
                            k3_fast,
                            k3_slow
                        );
                    }
                    if (f_fast - f_slow).abs() > 1e-9 {
                        anyhow::bail!(
                            "InjectionTwoDepot3C requires fast.f == slow.f (got {} vs {})",
                            f_fast,
                            f_slow
                        );
                    }

                    let mut extras = std::collections::HashMap::new();
                    extras.insert(ExtraKey::InjectionFracFast, *frac_fast);
                    extras.insert(ExtraKey::InjectionK1FastPerHour, k1_fast);
                    extras.insert(ExtraKey::InjectionK1SlowPerHour, k1_slow);
                    extras.insert(ExtraKey::InjectionK2PerHour, k2_fast);
                    extras.insert(ExtraKey::InjectionK3PerHour, k3_fast);
                    extras.insert(ExtraKey::InjectionFormationFraction, f_fast);

                    events.push(DoseEvent {
                        id: "inj".to_string(),
                        route: Route::Injection,
                        time_h: t0_h,
                        dose_mg: dose,
                        ester,
                        extras,
                    });
                }

                // 其它模型先不接入（InjectionTwoDepot3C / ThreeCompAnalytical）
                other => {
                    anyhow::bail!("model type not wired to event-based baseline yet: {:?}", other);
                }
            }

            // 构造固定网格，避免默认窗口过长导致输出巨大。
            let grid = SimulationGrid {
                start_h: t0_h,
                end_h: t0_h + total_h,
                steps,
            };

            events.sort_by(|a, b| a.time_h.total_cmp(&b.time_h));
            let sim = run_simulation_with_grid(&events, body_weight_kg, grid)
                .context("simulation returned null")?;

            let events_obj = serde_json::json!({
                "meta": {"t0_h": t0_h, "total_h": total_h, "steps": steps, "body_weight_kg": body_weight_kg},
                "events": events
            });

            let mut env = schema::envelope::Envelope::new(
                "tvp_hrt_engine.sim",
                "curve",
                db_dir.to_string(),
                events_obj,
                sim,
            );
            env.subject_kind = Some("compound");
            env.subject = Some(c);
            env.route = Some(r);
            env.dosage = Some(d);

            println!("{}", serde_json::to_string_pretty(&env)?);
        }
    }

    Ok(())
}

fn resolve_db_dir(arg: &str) -> Result<String> {
    let candidates: Vec<&str> = if arg.trim().is_empty() {
        vec![
            "assets/database/hrt",
            "../assets/database/hrt",
            "../../assets/database/hrt",
        ]
    } else {
        vec![
            arg,
            "assets/database/hrt",
            "../assets/database/hrt",
            "../../assets/database/hrt",
        ]
    };

    for c in candidates {
        let p = std::path::Path::new(c);
        if p.join("substances").exists() && p.join("routes").exists() && p.join("dosages").exists() {
            return Ok(c.to_string());
        }
    }

    anyhow::bail!(
        "无法定位 HRT 数据库目录。请显式传入 --db-dir，例如：--db-dir \"../assets/database/hrt\""
    );
}

fn compound_id_to_ester(compound_id: &str) -> Ester {
    match compound_id {
        "cpa" => Ester::CPA,
        "eb" => Ester::EB,
        "ev" => Ester::EV,
        "ec" => Ester::EC,
        "en" => Ester::EN,
        _ => Ester::E2,
    }
}

fn extract_three_comp_params(model: &HrtModel) -> Result<(f64, f64, f64, f64)> {
    match model {
        HrtModel::ThreeCompAnalytical {
            k1_per_hour,
            k2_per_hour,
            k3_per_hour,
            f,
        } => Ok((*k1_per_hour, *k2_per_hour, *k3_per_hour, *f)),
        other => anyhow::bail!("expected ThreeCompAnalytical, got {:?}", other),
    }
}
