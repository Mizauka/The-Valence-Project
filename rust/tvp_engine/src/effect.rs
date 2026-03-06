//! Effect timeline simulation ported from psychonautwiki-journal-android.
//!
//! Reference implementation:
//! - `FullTimelines.kt`
//! - `GroupDrawable.kt`
//! - `FullDurationRange.kt`
//!
//! This module intentionally models the *timeline/effect curve* used by the Journal app,
//! which is distinct from pharmacokinetic concentration models.

use crate::db::{L3Dosage, Range};
use anyhow::{bail, Result};

#[derive(Debug, Clone, Copy)]
pub struct DurationRangeSeconds {
    pub min_s: f64,
    pub max_s: f64,
}

impl DurationRangeSeconds {
    pub fn interpolate_at_value_seconds(&self, value: f64) -> f64 {
        let diff = self.max_s - self.min_s;
        self.min_s + diff * value
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TimelineDurationsSeconds {
    pub onset: DurationRangeSeconds,
    pub comeup: DurationRangeSeconds,
    pub peak: DurationRangeSeconds,
    pub offset: DurationRangeSeconds,
}

#[derive(Debug, Clone, Copy)]
pub struct EffectEvent {
    /// Start time relative to the graph start, in hours.
    pub start_h: f64,
    /// Optional end time (if the ingestion is spread evenly over a time range), in hours.
    pub end_h: Option<f64>,
    /// Non-normalised "height" (strength) used by the timeline.
    pub height: f64,
    /// Horizontal weight used for peak/offset interpolation.
    pub horizontal_weight: f64,
}

#[derive(Debug, Clone, Copy)]
struct LineSegment {
    start: Point,
    end: Point,
}

impl LineSegment {
    fn is_inside(&self, x: f64) -> bool {
        self.start.x <= x && x < self.end.x
    }

    fn height_at(&self, x: f64) -> f64 {
        let divider = self.end.x - self.start.x;
        if divider == 0.0 {
            return 0.0;
        }
        let m = (self.end.y - self.start.y) / divider;
        let b = self.start.y - m * self.start.x;
        m * x + b
    }
}

#[derive(Debug, Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}

#[derive(Debug, Clone, Copy)]
struct FinalPoint {
    x: f64,
    y: f64,
    is_ingestion_point: bool,
}

#[derive(Debug, Clone)]
pub struct EffectTimeline {
    /// Points are in seconds relative to `t0_h` (i.e., x is seconds).
    final_points: Vec<FinalPoint>,
    /// Maximum raw height across the timeline (used for normalisation).
    pub non_normalised_height: f64,
    /// End time of the drawn timeline, seconds relative to t0.
    pub end_of_line_s: f64,
}

impl EffectTimeline {
    /// Evaluate raw (non-normalised) height at x (seconds since t0).
    pub fn value_raw_at_seconds(&self, x_s: f64) -> f64 {
        if self.final_points.is_empty() {
            return 0.0;
        }
        if x_s < self.final_points[0].x || x_s > self.final_points[self.final_points.len() - 1].x {
            return 0.0;
        }

        // Find adjacent points. `final_points` is sorted by x.
        // Linear interpolation between nearest neighbors.
        for w in self.final_points.windows(2) {
            let a = w[0];
            let b = w[1];
            if a.x <= x_s && x_s <= b.x {
                let dx = b.x - a.x;
                if dx == 0.0 {
                    return a.y;
                }
                let t = (x_s - a.x) / dx;
                return a.y + (b.y - a.y) * t;
            }
        }
        0.0
    }

    /// Evaluate normalised height (like Journal draw code: y/referenceHeight).
    /// For a single group, referenceHeight == non_normalised_height.
    pub fn value_norm_at_seconds(&self, x_s: f64) -> f64 {
        let raw = self.value_raw_at_seconds(x_s);
        if self.non_normalised_height <= 0.0 {
            return 0.0;
        }
        raw / self.non_normalised_height
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct EffectSimulationResult {
    #[serde(rename = "timeH")]
    pub time_h: Vec<f64>,
    /// Normalised (max=1.0 for a single timeline) effect level.
    #[serde(rename = "level")]
    pub level: Vec<f64>,
    /// Raw (non-normalised) effect level.
    #[serde(rename = "levelRaw")]
    pub level_raw: Vec<f64>,
    /// AUC of `level` over hours.
    pub auc: f64,
    /// AUC of `levelRaw` over hours.
    #[serde(rename = "aucRaw")]
    pub auc_raw: f64,
    /// Non-normalised max height (referenceHeight).
    #[serde(rename = "maxRaw")]
    pub max_raw: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct SimulationGrid {
    pub start_h: f64,
    pub end_h: f64,
    pub steps: usize,
}

pub fn build_durations_seconds_from_l3(l3: &L3Dosage) -> Result<TimelineDurationsSeconds> {
    let onset = range_to_seconds(l3.onset.as_ref()).ok_or_else(|| {
        anyhow::anyhow!("L3.onset 缺失或单位无法解析（仅支持 minutes/hours/days）")
    })?;
    let comeup = range_to_seconds(l3.comeup.as_ref()).ok_or_else(|| {
        anyhow::anyhow!("L3.comeup 缺失或单位无法解析（仅支持 minutes/hours/days）")
    })?;
    let peak = range_to_seconds(l3.peak.as_ref()).ok_or_else(|| {
        anyhow::anyhow!("L3.peak 缺失或单位无法解析（仅支持 minutes/hours/days）")
    })?;
    let offset = range_to_seconds(l3.offset.as_ref()).ok_or_else(|| {
        anyhow::anyhow!("L3.offset 缺失或单位无法解析（仅支持 minutes/hours/days）")
    })?;
    Ok(TimelineDurationsSeconds {
        onset,
        comeup,
        peak,
        offset,
    })
}

pub fn simulate_timeline(
    events: &[EffectEvent],
    durations: TimelineDurationsSeconds,
    grid: SimulationGrid,
) -> Result<(EffectTimeline, EffectSimulationResult)> {
    if events.is_empty() {
        bail!("events 不能为空");
    }
    if grid.steps == 0 {
        bail!("grid.steps 必须 > 0");
    }
    if !(grid.end_h > grid.start_h) {
        bail!("grid.end_h 必须 > grid.start_h");
    }

    // The Journal model builds x in seconds relative to the timeline origin.
    // We support arbitrary grid.start_h by converting events to be relative to it.
    let events_rel: Vec<EffectEvent> = events
        .iter()
        .map(|e| EffectEvent {
            start_h: e.start_h - grid.start_h,
            end_h: e.end_h.map(|x| x - grid.start_h),
            height: e.height,
            horizontal_weight: e.horizontal_weight,
        })
        .collect();

    let timeline = build_full_timeline(&events_rel, durations)?;

    let steps = grid.steps;
    let mut time_h = Vec::with_capacity(steps + 1);
    let mut level = Vec::with_capacity(steps + 1);
    let mut level_raw = Vec::with_capacity(steps + 1);

    let dt_h = (grid.end_h - grid.start_h) / (steps as f64);
    for i in 0..=steps {
        let t_h = grid.start_h + (i as f64) * dt_h;
        let x_s = (t_h - grid.start_h) * 3600.0;
        let raw = timeline.value_raw_at_seconds(x_s);
        let norm = timeline.value_norm_at_seconds(x_s);
        time_h.push(t_h);
        level_raw.push(raw);
        level.push(norm);
    }

    let auc = auc_trapezoid_hours(&time_h, &level);
    let auc_raw = auc_trapezoid_hours(&time_h, &level_raw);

    Ok((
        timeline.clone(),
        EffectSimulationResult {
            time_h,
            level,
            level_raw,
            auc,
            auc_raw,
            max_raw: timeline.non_normalised_height,
        },
    ))
}

fn build_full_timeline(
    events: &[EffectEvent],
    durations: TimelineDurationsSeconds,
) -> Result<EffectTimeline> {
    // Port of FullTimelines.init
    let mut line_segments: Vec<LineSegment> = Vec::new();

    // Time-range ingestions (endTime != null) are converted to line segments via convolution.
    for e in events.iter().filter(|e| e.end_h.is_some()) {
        let end_h = e.end_h.unwrap();
        if end_h < e.start_h {
            bail!("event.end_h 必须 >= start_h");
        }
        let segments = time_range_line_segments(e, durations);
        line_segments.extend(segments);
    }

    // Point ingestions (endTime == null)
    for e in events.iter().filter(|e| e.end_h.is_none()) {
        let rel_start_s = e.start_h * 3600.0;
        let onset_and_comeup_weight = 0.5;

        let onset_end_x = rel_start_s + durations.onset.interpolate_at_value_seconds(onset_and_comeup_weight);
        let comeup_start = Point { x: onset_end_x, y: 0.0 };

        let comeup_end_x = onset_end_x + durations.comeup.interpolate_at_value_seconds(onset_and_comeup_weight);
        let peak_start = Point {
            x: comeup_end_x,
            y: e.height,
        };
        line_segments.push(LineSegment {
            start: comeup_start,
            end: peak_start,
        });

        let peak_end_x = comeup_end_x + durations.peak.interpolate_at_value_seconds(e.horizontal_weight);
        let peak_end = Point {
            x: peak_end_x,
            y: e.height,
        };
        line_segments.push(LineSegment {
            start: peak_start,
            end: peak_end,
        });

        let offset_end_x = peak_end_x + durations.offset.interpolate_at_value_seconds(e.horizontal_weight);
        let offset_end = Point { x: offset_end_x, y: 0.0 };
        line_segments.push(LineSegment {
            start: peak_end,
            end: offset_end,
        });
    }

    // Collect x coordinates from all segment endpoints (like Kotlin distinct()).
    let mut line_points_x: Vec<f64> = line_segments
        .iter()
        .flat_map(|s| [s.start.x, s.end.x])
        .collect();
    line_points_x.sort_by(|a, b| a.total_cmp(b));
    line_points_x.dedup();

    let mut points_to_consider: Vec<FinalPoint> = Vec::new();
    // Ingestion points (for point ingestions only)
    for e in events.iter().filter(|e| e.end_h.is_none()) {
        points_to_consider.push(FinalPoint {
            x: e.start_h * 3600.0,
            y: 0.0,
            is_ingestion_point: true,
        });
    }
    for x in line_points_x {
        points_to_consider.push(FinalPoint {
            x,
            y: 0.0,
            is_ingestion_point: false,
        });
    }

    // Compute summed heights at each x.
    let mut points_with_height: Vec<FinalPoint> = Vec::with_capacity(points_to_consider.len());
    for p in points_to_consider {
        let x = p.x;
        let sum_heights: f64 = line_segments
            .iter()
            .map(|seg| if seg.is_inside(x) { seg.height_at(x) } else { 0.0 })
            .sum();
        points_with_height.push(FinalPoint {
            x,
            y: sum_heights,
            is_ingestion_point: p.is_ingestion_point,
        });
    }

    points_with_height.sort_by(|a, b| a.x.total_cmp(&b.x));
    // Kotlin keeps duplicates if points_to_consider includes same x with different flags;
    // but later drawing iterates; for evaluation, keep only one per x (sum is identical).
    // Preserve ingestion point flag if any.
    let mut merged: Vec<FinalPoint> = Vec::new();
    for p in points_with_height {
        if let Some(last) = merged.last_mut() {
            if last.x == p.x {
                last.is_ingestion_point |= p.is_ingestion_point;
                // y should be equal; keep max defensively.
                if p.y > last.y {
                    last.y = p.y;
                }
                continue;
            }
        }
        merged.push(p);
    }

    let non_normalised_height = merged
        .iter()
        .map(|p| p.y)
        .fold(0.01, f64::max);

    let end_of_line_s = merged
        .iter()
        .map(|p| p.x)
        .fold(0.0, f64::max);

    Ok(EffectTimeline {
        final_points: merged,
        non_normalised_height,
        end_of_line_s,
    })
}

fn time_range_line_segments(event: &EffectEvent, durations: TimelineDurationsSeconds) -> Vec<LineSegment> {
    // Port of FullTimelines.getLineSegments(weightedLine)
    let start_x = event.start_h * 3600.0;
    let end_x = event.end_h.unwrap() * 3600.0;

    let range_in_seconds = end_x - start_x;
    let onset_s = durations.onset.interpolate_at_value_seconds(0.5);
    let comeup_s = durations.comeup.interpolate_at_value_seconds(0.5);

    let mut horizontal_weight_to_use = 0.5;
    if range_in_seconds < durations.peak.min_s {
        // if the range is short enough we use the same duration as for point ingestion
        horizontal_weight_to_use = event.horizontal_weight;
    }
    let peak_s = durations.peak.interpolate_at_value_seconds(horizontal_weight_to_use);
    let offset_s = durations.offset.interpolate_at_value_seconds(horizontal_weight_to_use);

    let points = get_sample_points(
        start_x,
        end_x,
        event.height,
        onset_s,
        comeup_s,
        peak_s,
        offset_s,
    );
    get_line_segments_from_points(&points)
}

fn get_line_segments_from_points(points: &[Point]) -> Vec<LineSegment> {
    if points.is_empty() {
        return Vec::new();
    }
    let mut result = Vec::with_capacity(points.len().saturating_sub(1));
    let mut prev = points[0];
    for &cur in points.iter().skip(1) {
        result.push(LineSegment { start: prev, end: cur });
        prev = cur;
    }
    result
}

fn get_sample_points(
    start_x: f64,
    end_x: f64,
    h_max: f64,
    onset: f64,
    comeup: f64,
    peak: f64,
    offset: f64,
) -> Vec<Point> {
    // Port of FullTimelines.getSamplePoints
    if start_x > end_x {
        return Vec::new();
    }
    let number_of_steps: i32 = 30;
    let start_sample_range = start_x + onset;
    let end_sample_range = end_x + onset + comeup + peak + offset;
    let step_size = (end_sample_range - start_sample_range) / (number_of_steps as f64);

    let mut points: Vec<Point> = Vec::with_capacity(number_of_steps as usize + 1);
    let first = Point {
        x: start_sample_range,
        y: 0.0,
    };
    points.push(first);

    for step in 1..number_of_steps {
        let x = start_sample_range + (step as f64) * step_size;
        let height = calculate_expression(x, start_x, end_x, h_max, onset, comeup, peak, offset);
        points.push(Point { x, y: height });
    }
    points.push(Point {
        x: end_sample_range,
        y: 0.0,
    });
    points
}

fn calculate_expression(
    x: f64,
    start_x: f64,
    end_x: f64,
    h_max: f64,
    onset: f64,
    comeup: f64,
    peak: f64,
    offset: f64,
) -> f64 {
    // Port of FullTimelines.calculateExpression
    fn clamp_min_max(v: f64, min_v: f64, max_v: f64) -> f64 {
        v.max(min_v).min(max_v)
    }

    let term1 = 2.0
        * comeup
        * offset
        * (
            clamp_min_max(-comeup - onset + x, start_x, end_x)
                - clamp_min_max(-comeup - onset - peak + x, start_x, end_x)
        );

    let term2 = 2.0
        * comeup
        * (
            clamp_min_max(-comeup - onset - peak + x, start_x, end_x)
                - clamp_min_max(-comeup - offset - onset - peak + x, start_x, end_x)
        )
        * (comeup + offset + onset + peak - x);

    let term3 = comeup
        * (
            clamp_min_max(-comeup - onset - peak + x, start_x, end_x).powi(2)
                - clamp_min_max(-comeup - offset - onset - peak + x, start_x, end_x).powi(2)
        );

    let term4 = 2.0
        * offset
        * (onset - x)
        * (
            -clamp_min_max(-onset + x, start_x, end_x)
                + clamp_min_max(-comeup - onset + x, start_x, end_x)
        );

    let term5 = offset
        * (
            -clamp_min_max(-onset + x, start_x, end_x).powi(2)
                + clamp_min_max(-comeup - onset + x, start_x, end_x).powi(2)
        );

    let numerator = 0.5 * h_max * (term1 + term2 + term3 + term4 + term5);
    let denominator = comeup * offset * (end_x - start_x);
    if denominator == 0.0 {
        return 0.0;
    }
    numerator / denominator
}

fn range_to_seconds(r: Option<&Range>) -> Option<DurationRangeSeconds> {
    let r = r?;
    let (min, max) = (r.min, r.max);
    if min <= 0.0 || max <= 0.0 || max < min {
        return None;
    }
    let u = r.units.to_ascii_lowercase();
    let mult = match u.as_str() {
        "minutes" | "minute" | "min" => 60.0,
        "hours" | "hour" | "h" => 3600.0,
        "days" | "day" => 86400.0,
        _ => return None,
    };
    Some(DurationRangeSeconds {
        min_s: min * mult,
        max_s: max * mult,
    })
}

pub fn auc_trapezoid_hours(time_h: &[f64], y: &[f64]) -> f64 {
    if time_h.len() != y.len() || time_h.len() < 2 {
        return 0.0;
    }
    let mut auc = 0.0;
    for i in 1..time_h.len() {
        let dt = time_h[i] - time_h[i - 1];
        let avg = 0.5 * (y[i] + y[i - 1]);
        auc += avg * dt;
    }
    auc
}
