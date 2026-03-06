use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tvp_engine::{
    db::Database,
    ddi::{self},
    effect,
    pk::{self, BuildPkOptions, FixedIntervalMode},
};

mod schema;

#[derive(Debug, Parser)]
#[command(
    name = "tvp",
    version,
    about = "The-Valence-Project CLI (Rust backend validation)"
)]
struct Cli {
    #[arg(
        long,
        default_value = "assets/database",
        help = "数据库根目录（包含 substances/routes/dosages）。若相对路径无效会自动探测 ../assets/database 等"
    )]
    db_dir: String,

    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Db {
        #[command(subcommand)]
        cmd: DbCmd,
    },
    Ddi {
        #[command(subcommand)]
        cmd: DdiCmd,
    },
    Pk {
        #[command(subcommand)]
        cmd: PkCmd,
    },
    /// Journal-style effect timeline (durations + strength + convolution for time-range ingestion)
    Effect {
        #[command(subcommand)]
        cmd: EffectCmd,
    },

    /// Run simulations from an events JSON file (like tvp_hrt_cli simulate)
    Simulate {
        /// Simulation kind: effect | pk
        #[arg(long)]
        kind: String,

        /// Path to events json
        #[arg(long)]
        events: std::path::PathBuf,

        /// substance name/id
        #[arg(long)]
        substance: String,

        /// alias: compound name/id
        #[arg(long)]
        compound: Option<String>,

        /// route name/id
        #[arg(long)]
        route: String,

        /// for effect: total output hours
        #[arg(long, default_value_t = 24.0)]
        total_h: f64,

        /// grid steps
        #[arg(long, default_value_t = 400)]
        steps: usize,

        /// for pk: dt minutes
        #[arg(long, default_value_t = 2.0)]
        dt_minutes: f64,

        /// for pk: optional overrides
        #[arg(long)]
        ka_per_hour: Option<f64>,
        #[arg(long)]
        ke_per_hour: Option<f64>,
        #[arg(long)]
        half_life_hours: Option<f64>,
        #[arg(long, default_value_t = false)]
        estimate_ke_from_total: bool,
    },
}

#[derive(Debug, Subcommand)]
enum EffectCmd {
    /// Build a single-effect timeline curve from L3 durations.
    Curve {
        #[arg(long)]
        substance: String,

        /// alias: compound name/id
        #[arg(long)]
        compound: Option<String>,

        #[arg(long)]
        route: String,

        /// Ingestion dose value (must match L3.dose_units).
        #[arg(long)]
        dose: f64,

        /// Start time (hour). Default 0.
        #[arg(long, default_value_t = 0.0)]
        t0_h: f64,

        /// Optional end time (hour). If set, models time-range ingestion (evenly consumed).
        #[arg(long)]
        t1_h: Option<f64>,

        /// Horizontal weight (derived from dots in app; default 0.5).
        #[arg(long, default_value_t = 0.5)]
        horizontal_weight: f64,

        /// Total curve duration to output (hour).
        #[arg(long, default_value_t = 24.0)]
        total_h: f64,

        /// Grid steps.
        #[arg(long, default_value_t = 400)]
        steps: usize,

        /// Override reference common dose (otherwise uses (common_min+strong_min)/2 when available).
        #[arg(long)]
        common_dose: Option<f64>,
    },
}

#[derive(Debug, Subcommand)]
enum DbCmd {
    Stats,
    Get {
        #[arg(help = "substance 名称或 id（支持 commonNames）")]
        substance: String,
    },
}

#[derive(Debug, Subcommand)]
enum DdiCmd {
    Check {
        #[arg(
            long,
            help = "用逗号分隔的物质列表（名称/id/commonName 均可）例如: \"Alcohol,MDMA\""
        )]
        substances: String,
    },
}

#[derive(Debug, Subcommand)]
enum PkCmd {
    Curve {
        #[arg(long)]
        substance: String,
        #[arg(long)]
        compound: Option<String>,
        #[arg(long)]
        route: String,
        #[arg(long)]
        dose: f64,
        #[arg(long, default_value_t = 2.0)]
        dt_minutes: f64,
        #[arg(long, default_value_t = 24.0)]
        total_hours: f64,

        #[arg(long)]
        ka_per_hour: Option<f64>,
        #[arg(long)]
        ke_per_hour: Option<f64>,
        #[arg(long)]
        half_life_hours: Option<f64>,
        #[arg(long, default_value_t = false)]
        estimate_ke_from_total: bool,

        #[arg(long, default_value_t = 50, help = "最多输出多少个点（避免刷屏）")]
        max_points: usize,
    },
    Repeat {
        #[arg(long)]
        substance: String,
            #[arg(long)]
            compound: Option<String>,
        #[arg(long)]
        route: String,
        #[arg(long)]
        dose: f64,

        #[arg(long, help = "给药间隔（小时），例如 q24h => 24")]
        tau_hours: f64,
        #[arg(long, help = "给药次数")]
        doses: usize,

        #[arg(long, default_value_t = 2.0)]
        dt_minutes: f64,
        #[arg(long, default_value_t = 24.0)]
        total_hours: f64,

        #[arg(long)]
        ka_per_hour: Option<f64>,
        #[arg(long)]
        ke_per_hour: Option<f64>,
        #[arg(long)]
        half_life_hours: Option<f64>,
        #[arg(long, default_value_t = false)]
        estimate_ke_from_total: bool,

        #[arg(long, default_value_t = 50, help = "最多输出多少个点（避免刷屏）")]
        max_points: usize,
    },
    SteadyState {
        #[arg(long)]
        substance: String,
            #[arg(long)]
            compound: Option<String>,
        #[arg(long)]
        route: String,
        #[arg(long)]
        dose: f64,

        #[arg(long, help = "给药间隔（小时），例如 q24h => 24")]
        tau_hours: f64,

        #[arg(long, default_value_t = 2.0)]
        dt_minutes: f64,
        #[arg(long, default_value_t = 24.0)]
        total_hours: f64,

        #[arg(long)]
        ka_per_hour: Option<f64>,
        #[arg(long)]
        ke_per_hour: Option<f64>,
        #[arg(long)]
        half_life_hours: Option<f64>,
        #[arg(long, default_value_t = false)]
        estimate_ke_from_total: bool,

        #[arg(long, default_value_t = 50, help = "最多输出多少个点（避免刷屏）")]
        max_points: usize,
    },
    Validate {
        #[arg(long)]
        substance: String,
            #[arg(long)]
            compound: Option<String>,
        #[arg(long)]
        route: String,
        #[arg(long)]
        dose: f64,

        #[arg(long, default_value_t = 2.0)]
        dt_minutes: f64,
        #[arg(long, default_value_t = 24.0)]
        total_hours: f64,

        #[arg(long)]
        ka_per_hour: Option<f64>,
        #[arg(long)]
        ke_per_hour: Option<f64>,
        #[arg(long)]
        half_life_hours: Option<f64>,
        #[arg(long, default_value_t = false)]
        estimate_ke_from_total: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let db_dir = resolve_db_dir(&cli.db_dir)?;
    let db =
        Database::load_from_assets_dir(&db_dir).with_context(|| format!("加载数据库失败：{}", db_dir))?;

    match cli.cmd {
        Commands::Db { cmd } => cmd_db(&db, cmd),
        Commands::Ddi { cmd } => cmd_ddi(&db, cmd),
        Commands::Pk { cmd } => cmd_pk(&db, cmd),
        Commands::Effect { cmd } => cmd_effect(&db_dir, &db, cmd),
        Commands::Simulate {
            kind,
            events,
            substance,
            compound,
            route,
            total_h,
            steps,
            dt_minutes,
            ka_per_hour,
            ke_per_hour,
            half_life_hours,
            estimate_ke_from_total,
        } => cmd_simulate(
            &db_dir,
            &db,
            &kind,
            events,
            &substance,
            compound.as_deref(),
            &route,
            total_h,
            steps,
            dt_minutes,
            ka_per_hour,
            ke_per_hour,
            half_life_hours,
            estimate_ke_from_total,
        ),
    }
}

fn cmd_simulate(
    db_dir: &str,
    db: &Database,
    kind: &str,
    events_path: std::path::PathBuf,
    substance: &str,
    compound: Option<&str>,
    route: &str,
    total_h: f64,
    steps: usize,
    dt_minutes: f64,
    ka_per_hour: Option<f64>,
    ke_per_hour: Option<f64>,
    half_life_hours: Option<f64>,
    estimate_ke_from_total: bool,
) -> Result<()> {
    let kind_lc = kind.to_ascii_lowercase();
    let subject_name = compound.unwrap_or(substance);
    let (s, r, l3) = resolve_substance_route(db, subject_name, route)?;

    let text = std::fs::read_to_string(&events_path)
        .with_context(|| format!("failed to read {}", events_path.display()))?;

    if kind_lc == "effect" {
        #[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
        struct EffectEventIn {
            #[serde(rename = "t0_h")]
            t0_h: f64,
            #[serde(rename = "t1_h")]
            t1_h: Option<f64>,
            dose: f64,
            #[serde(default)]
            horizontal_weight: Option<f64>,
            #[serde(default)]
            common_dose: Option<f64>,
        }

        let l3 = l3.with_context(|| {
            format!(
                "找不到 L3 配置：substance={} route={}（无法构建 effect timeline durations）",
                s.id, r.id
            )
        })?;

        let input_events: Vec<EffectEventIn> = serde_json::from_str(&text)
            .with_context(|| format!("failed to parse events JSON: {}", events_path.display()))?;

        let durations = effect::build_durations_seconds_from_l3(l3)?;

        let mut sim_events: Vec<effect::EffectEvent> = Vec::new();
        let mut out_events_json: Vec<serde_json::Value> = Vec::new();

        for ev in &input_events {
            let common = ev.common_dose.unwrap_or_else(|| if ev.dose > 0.0 { ev.dose } else { 1.0 });
            let height = if common > 0.0 && ev.dose > 0.0 { ev.dose / common } else { 1.0 };
            let hw = ev.horizontal_weight.unwrap_or(0.5);
            sim_events.push(effect::EffectEvent {
                start_h: ev.t0_h,
                end_h: ev.t1_h,
                height,
                horizontal_weight: hw,
            });
            out_events_json.push(serde_json::json!({
                "t0_h": ev.t0_h,
                "t1_h": ev.t1_h,
                "dose": ev.dose,
                "dose_units": l3.dose_units,
                "common_dose": common,
                "height": height,
                "horizontal_weight": hw
            }));
        }

        let grid = effect::SimulationGrid {
            start_h: sim_events
                .iter()
                .map(|e| e.start_h)
                .fold(f64::INFINITY, f64::min),
            end_h: sim_events
                .iter()
                .map(|e| e.start_h)
                .fold(f64::INFINITY, f64::min)
                + total_h,
            steps,
        };

        let (_timeline, result) = effect::simulate_timeline(&sim_events, durations, grid)?;

        let events_obj = serde_json::json!({
            "meta": {"type": "effect", "total_h": total_h, "steps": steps},
            "events": out_events_json
        });

        let mut env = schema::envelope::Envelope::new(
            "tvp_engine.effect",
            "simulate",
            db_dir.to_string(),
            events_obj,
            result,
        );
        env.subject_kind = Some("substance");
        env.subject = Some(s);
        env.route = Some(r);
        env.dosage = Some(l3);

        println!("{}", serde_json::to_string_pretty(&env)?);
        return Ok(());
    }

    if kind_lc == "pk" {
        #[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
        struct PkDoseEventIn {
            #[serde(rename = "time_h")]
            time_h: f64,
            dose: f64,
        }

        let input_events: Vec<PkDoseEventIn> = serde_json::from_str(&text)
            .with_context(|| format!("failed to parse events JSON: {}", events_path.display()))?;

        let opts = BuildPkOptions {
            override_ka_per_hour: ka_per_hour,
            override_ke_per_hour: ke_per_hour,
            override_half_life_hours: half_life_hours,
            estimate_ke_from_l3_total_max: estimate_ke_from_total,
        };

        // For now: sum independent single-dose curves shifted by time_h (matches linearity of Bateman).
        // Output points are on a fixed grid [t0, t0+total_h].
        let l3_opt = l3;
        let (base_curve, ke_source) = pk::curve_for_substance_route(
            db,
            s,
            r,
            l3_opt,
            1.0,
            dt_minutes,
            total_h,
            &opts,
        )?;

        let dt_h = dt_minutes / 60.0;
        let n = (total_h / dt_h).ceil() as usize;
        let mut points: Vec<serde_json::Value> = Vec::with_capacity(n + 1);
        for i in 0..=n {
            let t = (i as f64) * dt_h;
            let mut c_sum = 0.0;
            for ev in &input_events {
                let shifted_t = t - ev.time_h;
                if shifted_t < 0.0 {
                    continue;
                }
                c_sum += pk::bateman_concentration(shifted_t, ev.dose, &base_curve.params);
            }
            points.push(serde_json::json!({"t_hours": t, "c": c_sum}));
        }

        let warnings = if ke_source.is_none() {
            Some(vec!["ke 来源未知".to_string()])
        } else {
            None
        };

        let events_obj = serde_json::json!({
            "meta": {"type": "pk", "dt_minutes": dt_minutes, "total_hours": total_h},
            "events": input_events
                .iter()
                .map(|e| serde_json::json!({"time_h": e.time_h, "dose": e.dose}))
                .collect::<Vec<_>>()
        });

        let mut env = schema::envelope::Envelope::new(
            "tvp_engine.pk",
            "simulate",
            db_dir.to_string(),
            events_obj,
            serde_json::json!({
                "params": {
                    "ka_per_hour": base_curve.params.ka_per_hour,
                    "ke_per_hour": base_curve.params.ke_per_hour,
                    "kappa": base_curve.params.kappa,
                    "f": base_curve.params.f,
                },
                "ke_source": ke_source.as_ref().map(|x| format!("{:?}", x)),
                "points": points
            }),
        );
        env.subject_kind = Some("substance");
        env.subject = Some(s);
        env.route = Some(r);
        env.dosage = l3_opt;
        env.warnings = warnings;

        println!("{}", serde_json::to_string_pretty(&env)?);
        return Ok(());
    }

    anyhow::bail!("unsupported kind: {} (expected effect|pk)", kind);
}

fn cmd_effect(db_dir: &str, db: &Database, cmd: EffectCmd) -> Result<()> {
    match cmd {
        EffectCmd::Curve {
            substance,
            compound,
            route,
            dose,
            t0_h,
            t1_h,
            horizontal_weight,
            total_h,
            steps,
            common_dose,
        } => {
            let subject_name = compound.as_deref().unwrap_or(&substance);
            let (s, r, l3) = resolve_substance_route(db, subject_name, &route)?;
            let l3 = l3.with_context(|| {
                format!(
                    "找不到 L3 配置：substance={} route={}（无法构建 effect timeline durations）",
                    s.id, r.id
                )
            })?;

            // Journal: height = dose/commonDose; commonDose = averageCommonDose if available.
            let avg_common = if let (Some(common_min), Some(strong_min)) = (l3.common_min, l3.strong_min)
            {
                Some((common_min + strong_min) / 2.0)
            } else {
                None
            };
            let common = match common_dose.or(avg_common) {
                Some(v) => v,
                None => {
                    // Mirror Journal fallback for "averageDose" when no roaDose.averageCommonDose is available.
                    // With a single known ingestion, averageDose == dose -> height becomes 1.0.
                    if dose > 0.0 { dose } else { 1.0 }
                }
            };
            if common <= 0.0 {
                anyhow::bail!("common dose 必须 > 0");
            }
            let height = if dose > 0.0 { dose / common } else { 1.0 };

            let durations = effect::build_durations_seconds_from_l3(l3)?;

            if !(horizontal_weight.is_finite() && horizontal_weight > 0.0) {
                anyhow::bail!("horizontal_weight 必须为有限且 > 0");
            }

            let event = effect::EffectEvent {
                start_h: t0_h,
                end_h: t1_h,
                height,
                horizontal_weight,
            };

            let grid = effect::SimulationGrid {
                start_h: t0_h,
                end_h: t0_h + total_h,
                steps,
            };

            let (_timeline, result) = effect::simulate_timeline(&[event], durations, grid)?;

            let events_obj = serde_json::json!({
                "meta": {"type": "curve", "dose_units": l3.dose_units, "total_h": total_h, "steps": steps},
                "events": [serde_json::json!({
                    "t0_h": t0_h,
                    "t1_h": t1_h,
                    "dose": dose,
                    "dose_units": l3.dose_units,
                    "common_dose": common,
                    "height": height,
                    "horizontal_weight": horizontal_weight
                })]
            });

            let mut env = schema::envelope::Envelope::new(
                "tvp_engine.effect",
                "curve",
                db_dir.to_string(),
                events_obj,
                result,
            );
            env.subject_kind = Some("substance");
            env.subject = Some(s);
            env.route = Some(r);
            env.dosage = Some(l3);

            println!("{}", serde_json::to_string_pretty(&env)?);

            Ok(())
        }
    }
}

fn resolve_db_dir(arg: &str) -> Result<String> {
    let candidates: Vec<&str> = if arg.trim().is_empty() {
        vec!["assets/database", "../assets/database", "../../assets/database"]
    } else {
        vec![arg, "assets/database", "../assets/database", "../../assets/database"]
    };

    for c in candidates {
        let p = std::path::Path::new(c);
        if p.join("substances").exists() && p.join("routes").exists() && p.join("dosages").exists() {
            return Ok(c.to_string());
        }
    }

    anyhow::bail!(
        "无法定位数据库目录。请显式传入 --db-dir，例如：--db-dir \"../assets/database\""
    );
}

fn cmd_db(db: &Database, cmd: DbCmd) -> Result<()> {
    match cmd {
        DbCmd::Stats => {
            let s = db.stats();
            println!(
                "{}",
                serde_json::json!({
                    "l1_substances": s.l1_substances,
                    "l2_routes": s.l2_routes,
                    "l3_dosages": s.l3_dosages,
                    "ddi_ref_total": s.ddi_ref_total,
                    "categories": s.categories,
                })
            );
            Ok(())
        }
        DbCmd::Get { substance } => {
            let s = db
                .substance_by_name_or_id(&substance)
                .with_context(|| format!("找不到 substance：{substance}"))?;
            println!("{}", serde_json::to_string_pretty(s)?);
            Ok(())
        }
    }
}

fn cmd_ddi(db: &Database, cmd: DdiCmd) -> Result<()> {
    match cmd {
        DdiCmd::Check { substances } => {
            let names = split_csv(&substances);
            let mut subs = Vec::new();
            for n in &names {
                let s = db
                    .substance_by_name_or_id(n)
                    .with_context(|| format!("找不到 substance：{n}"))?;
                subs.push(s);
            }
            let report = ddi::check_ddi(db, &subs);
            println!(
                "{}",
                serde_json::json!({
                    "highest": format!("{:?}", report.highest),
                    "hits": report.hits.iter().map(|h| {
                        serde_json::json!({
                            "a": {"id": h.a_substance_id, "name": h.a_substance_name},
                            "b_input": h.b_input,
                            "matched_as": format!("{:?}", h.matched_as),
                            "risk": format!("{:?}", h.risk),
                        })
                    }).collect::<Vec<_>>()
                })
            );
            Ok(())
        }
    }
}

fn cmd_pk(db: &Database, cmd: PkCmd) -> Result<()> {
    match cmd {
        PkCmd::Curve {
            substance,
            compound,
            route,
            dose,
            dt_minutes,
            total_hours,
            ka_per_hour,
            ke_per_hour,
            half_life_hours,
            estimate_ke_from_total,
            max_points,
        } => {
            let subject_name = compound.as_deref().unwrap_or(&substance);
            let (s, r, l3) = resolve_substance_route(db, subject_name, &route)?;
            let opts = BuildPkOptions {
                override_ka_per_hour: ka_per_hour,
                override_ke_per_hour: ke_per_hour,
                override_half_life_hours: half_life_hours,
                estimate_ke_from_l3_total_max: estimate_ke_from_total,
            };
            let (curve, ke_source) =
                pk::curve_for_substance_route(db, s, r, l3, dose, dt_minutes, total_hours, &opts)?;

            let mut pts = curve.points;
            if pts.len() > max_points {
                let step = (pts.len() as f64 / max_points as f64).ceil() as usize;
                pts = pts.into_iter().step_by(step).collect();
            }

            let mut warnings: Vec<String> = Vec::new();
            if ke_source.is_none() {
                warnings.push("ke 来源未知".to_string());
            }

            let events_obj = serde_json::json!({
                "meta": {"type": "pk_curve", "dose": dose, "dt_minutes": dt_minutes, "total_hours": total_hours},
                "events": [serde_json::json!({
                    "overrides": {
                        "ka_per_hour": ka_per_hour,
                        "ke_per_hour": ke_per_hour,
                        "half_life_hours": half_life_hours,
                        "estimate_ke_from_total": estimate_ke_from_total
                    }
                })]
            });

            let mut env = schema::envelope::Envelope::new(
                "tvp_engine.pk",
                "curve",
                db.root.to_string_lossy().to_string(),
                events_obj,
                serde_json::json!({
                    "params": {
                        "ka_per_hour": curve.params.ka_per_hour,
                        "ke_per_hour": curve.params.ke_per_hour,
                        "kappa": curve.params.kappa,
                        "c0": curve.params.c0,
                        "f": curve.params.f,
                    },
                    "ke_source": ke_source.as_ref().map(|x| format!("{:?}", x)),
                    "t_max_hours": curve.t_max_hours,
                    "c_max": curve.c_max,
                    "points": pts.iter().map(|p| serde_json::json!({"t_hours": p.t_hours, "c": p.c})).collect::<Vec<_>>()
                }),
            );
            env.subject_kind = Some("substance");
            env.subject = Some(s);
            env.route = Some(r);
            env.dosage = l3;
            if !warnings.is_empty() {
                env.warnings = Some(warnings);
            }

            println!("{}", serde_json::to_string_pretty(&env)?);
            Ok(())
        }
        PkCmd::Repeat {
            substance,
            compound,
            route,
            dose,
            tau_hours,
            doses,
            dt_minutes,
            total_hours,
            ka_per_hour,
            ke_per_hour,
            half_life_hours,
            estimate_ke_from_total,
            max_points,
        } => {
            let subject_name = compound.as_deref().unwrap_or(&substance);
            let (s, r, l3) = resolve_substance_route(db, subject_name, &route)?;
            let opts = BuildPkOptions {
                override_ka_per_hour: ka_per_hour,
                override_ke_per_hour: ke_per_hour,
                override_half_life_hours: half_life_hours,
                estimate_ke_from_l3_total_max: estimate_ke_from_total,
            };

            let (curve, ke_source) = pk::curve_fixed_interval(
                db,
                s,
                r,
                l3,
                dose,
                dt_minutes,
                total_hours,
                FixedIntervalMode::Finite {
                    tau_hours,
                    doses,
                },
                &opts,
            )?;

            let mut pts = curve.points;
            if pts.len() > max_points {
                let step = (pts.len() as f64 / max_points as f64).ceil() as usize;
                pts = pts.into_iter().step_by(step).collect();
            }

            println!(
                "{}",
                serde_json::json!({
                    "mode": "repeat",
                    "substance": {"id": s.id, "name": s.name},
                    "route": {"id": r.id, "name": r.name},
                    "dose": dose,
                    "tau_hours": tau_hours,
                    "doses": doses,
                    "dt_minutes": dt_minutes,
                    "total_hours": total_hours,
                    "params": {
                        "ka_per_hour": curve.params.ka_per_hour,
                        "ke_per_hour": curve.params.ke_per_hour,
                        "f": curve.params.f,
                        "kappa": curve.params.kappa,
                    },
                    "ke_source": ke_source.as_ref().map(|x| format!("{:?}", x)),
                    "t_max_hours": curve.t_max_hours,
                    "c_max": curve.c_max,
                    "points": pts.iter().map(|p| serde_json::json!({"t_hours": p.t_hours, "c": p.c})).collect::<Vec<_>>()
                })
            );
            Ok(())
        }
        PkCmd::SteadyState {
            substance,
            compound,
            route,
            dose,
            tau_hours,
            dt_minutes,
            total_hours,
            ka_per_hour,
            ke_per_hour,
            half_life_hours,
            estimate_ke_from_total,
            max_points,
        } => {
            let subject_name = compound.as_deref().unwrap_or(&substance);
            let (s, r, l3) = resolve_substance_route(db, subject_name, &route)?;
            let opts = BuildPkOptions {
                override_ka_per_hour: ka_per_hour,
                override_ke_per_hour: ke_per_hour,
                override_half_life_hours: half_life_hours,
                estimate_ke_from_l3_total_max: estimate_ke_from_total,
            };

            let (curve, ke_source) = pk::curve_fixed_interval(
                db,
                s,
                r,
                l3,
                dose,
                dt_minutes,
                total_hours,
                FixedIntervalMode::SteadyState { tau_hours },
                &opts,
            )?;

            let mut pts = curve.points;
            if pts.len() > max_points {
                let step = (pts.len() as f64 / max_points as f64).ceil() as usize;
                pts = pts.into_iter().step_by(step).collect();
            }

            println!(
                "{}",
                serde_json::json!({
                    "mode": "steady_state",
                    "substance": {"id": s.id, "name": s.name},
                    "route": {"id": r.id, "name": r.name},
                    "dose": dose,
                    "tau_hours": tau_hours,
                    "dt_minutes": dt_minutes,
                    "total_hours": total_hours,
                    "params": {
                        "ka_per_hour": curve.params.ka_per_hour,
                        "ke_per_hour": curve.params.ke_per_hour,
                        "f": curve.params.f,
                        "kappa": curve.params.kappa,
                    },
                    "ke_source": ke_source.as_ref().map(|x| format!("{:?}", x)),
                    "t_max_hours": curve.t_max_hours,
                    "c_max": curve.c_max,
                    "points": pts.iter().map(|p| serde_json::json!({"t_hours": p.t_hours, "c": p.c})).collect::<Vec<_>>()
                })
            );
            Ok(())
        }
        PkCmd::Validate {
            substance,
            compound,
            route,
            dose,
            dt_minutes,
            total_hours,
            ka_per_hour,
            ke_per_hour,
            half_life_hours,
            estimate_ke_from_total,
        } => {
            let subject_name = compound.as_deref().unwrap_or(&substance);
            let (s, r, l3) = resolve_substance_route(db, subject_name, &route)?;
            let l3 = l3.with_context(|| {
                format!(
                    "找不到 L3 配置：substance={} route={}（无法做 peak 区间验证）",
                    s.id, r.id
                )
            })?;

            let opts = BuildPkOptions {
                override_ka_per_hour: ka_per_hour,
                override_ke_per_hour: ke_per_hour,
                override_half_life_hours: half_life_hours,
                estimate_ke_from_l3_total_max: estimate_ke_from_total,
            };

            let (curve, ke_source) = pk::curve_for_substance_route(
                db,
                s,
                r,
                Some(l3),
                dose,
                dt_minutes,
                total_hours,
                &opts,
            )?;

            let peak = pk::peak_range_hours(l3).with_context(
                || "该 L3 没有可解析的 duration.peak（units 必须为 minutes/hours/days）",
            )?;
            let t_max = curve
                .t_max_hours
                .with_context(|| "无法得到 t_max（曲线可能全为 NaN）")?;

            let passed = t_max >= peak.0 && t_max <= peak.1;
            let dist_to_range = if passed {
                0.0
            } else if t_max < peak.0 {
                peak.0 - t_max
            } else {
                t_max - peak.1
            };

            let risk_note = match ke_source {
                Some(_) => None,
                None => Some("ke 来源未知".to_string()),
            };

            println!(
                "{}",
                serde_json::json!({
                    "substance": {"id": s.id, "name": s.name},
                    "route": {"id": r.id, "name": r.name},
                    "dose": dose,
                    "params": {
                        "ka_per_hour": curve.params.ka_per_hour,
                        "ke_per_hour": curve.params.ke_per_hour,
                    },
                    "ke_source": ke_source.as_ref().map(|x| format!("{:?}", x)),
                    "duration_peak_hours": {"min": peak.0, "max": peak.1},
                    "t_max_hours": t_max,
                    "passed": passed,
                    "distance_hours": dist_to_range,
                    "note": risk_note
                })
            );
            Ok(())
        }
    }
}

fn resolve_substance_route<'a>(
    db: &'a Database,
    substance: &str,
    route: &str,
) -> Result<(
    &'a tvp_engine::db::L1Substance,
    &'a tvp_engine::db::L2Route,
    Option<&'a tvp_engine::db::L3Dosage>,
)> {
    let s = db
        .substance_by_name_or_id(substance)
        .with_context(|| format!("找不到 substance：{substance}"))?;
    let r = db
        .route_by_id_or_name(route)
        .with_context(|| format!("找不到 route：{route}"))?;
    let l3 = db.dosage(&s.id, &r.id);
    Ok((s, r, l3))
}

fn split_csv(s: &str) -> Vec<String> {
    s.split(',')
        .map(|x| x.trim().to_string())
        .filter(|x| !x.is_empty())
        .collect()
}
