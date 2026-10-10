//! Forced-turn experiment and turning-wave measurement.
//!
//! A turn is imposed gradually on birds on one side of the flock. For every
//! bird we then record the time its heading changes fastest (the turn's arrival
//! at that bird) and plot it against the bird's initial distance from the
//! disturbance. A straight line means the turn sweeps across the flock at a
//! roughly constant speed (Attanasi et al. 2014).

use glam::Mat3;
use glam::Vec3;

use super::World;
use super::flight::forward;
use super::params::DT;

/// Duration over which the imposed turn is applied \[s\].
const TURN_SECS: f32 = 0.5;

/// Parameters of a forced-turn experiment.
#[derive(Debug, Clone, Copy)]
pub struct TurnConfig {
    /// Number of birds on the disturbed side.
    pub side: usize,
    /// Total imposed heading change \[rad\].
    pub angle: f32,
    /// Time simulated after the disturbance begins \[s\].
    pub window: f32,
    /// Only birds whose heading changed by at least this much are fitted.
    pub min_turn: f32,
}

impl Default for TurnConfig {
    fn default() -> Self {
        Self {
            side: 100,
            angle: 0.8,
            window: 2.0,
            min_turn: 0.3,
        }
    }
}

/// Outcome of a forced-turn experiment.
#[derive(Debug, Clone)]
pub struct TurnWave {
    /// Fitted propagation speed [m/s] (`1 / |slope|`), or `NaN` if flat.
    pub speed: f32,
    /// Coefficient of determination of the time-vs-distance fit.
    pub r2: f32,
    /// Birds used in the fit.
    pub samples: usize,
    /// Fraction of the flock that turned by at least `min_turn`.
    pub turned_fraction: f32,
    /// Heading coherence (mean unit forward) after the wave.
    pub polarization_after: f32,
    /// Ratio of `I2/I1` after the turn to `I2/I1` before; ~1 means shape kept.
    pub shape_ratio: f32,
    /// Whether the flock stayed in one cluster through the turn.
    pub stayed_connected: bool,
    /// Span of arrival times \[s\].
    pub time_span: f32,
    /// Span of distances from the disturbance \[m\].
    pub distance_span: f32,
    /// `(distance, arrival time)` for every fitted bird.
    pub points: Vec<(f32, f32)>,
}

/// Runs the experiment on `world` (which should already be settled).
pub fn measure(world: &mut World, config: TurnConfig) -> TurnWave {
    let n = world.len();
    let TurnConfig {
        side,
        angle,
        window,
        min_turn,
    } = config;
    if n < side + 8 {
        return empty();
    }

    let initial_heading: Vec<Vec3> = world.velocity.iter().map(|v| forward(*v)).collect();
    let initial_position: Vec<Vec3> = world.position.clone();
    let shape_before = super::analysis::compute(world).i2_i1;

    // The disturbed birds: a slab on one side.
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| {
        initial_position[a]
            .y
            .partial_cmp(&initial_position[b].y)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let perturbed = &order[..side];
    let origin = perturbed.iter().map(|&i| initial_position[i]).sum::<Vec3>() / side as f32;

    let steps = (window / DT).max(1.0) as usize;
    let force_steps = (TURN_SECS / DT).max(1.0) as usize;
    let turn_step = Mat3::from_rotation_z(angle / force_steps as f32);

    // Arrival time per bird = first time its heading change *relative to the
    // flock's mean heading change* exceeds `min_turn`. Subtracting the mean
    // change removes the whole-flock body turn (common mode) and leaves the
    // propagating front.
    let mut arrival = vec![f32::INFINITY; n];
    let mut disconnected = false;

    for s in 0..steps {
        if s < force_steps {
            for &i in perturbed {
                let speed = world.velocity[i].length();
                world.velocity[i] = (turn_step * world.velocity[i]).normalize() * speed;
            }
        }
        world.step(DT);

        let mut heading_change = vec![0.0f32; n];
        for i in 0..n {
            let a = forward(world.velocity[i]);
            let b = initial_heading[i];
            heading_change[i] = a.cross(b).dot(Vec3::Z).atan2(a.dot(b));
        }
        let mean_change = heading_change.iter().sum::<f32>() / n as f32;
        for i in 0..n {
            let local = heading_change[i] - mean_change;
            if arrival[i].is_infinite() && local.abs() > min_turn {
                arrival[i] = world.time;
            }
        }
        if s % 100 == 99 && super::analysis::compute(world).largest_cluster_fraction < 0.95 {
            disconnected = true;
        }
    }

    let mut xs = Vec::new();
    let mut ys = Vec::new();
    let mut turned = 0usize;
    for i in 0..n {
        if arrival[i].is_finite() {
            turned += 1;
            xs.push((initial_position[i] - origin).length());
            ys.push(arrival[i]);
        }
    }

    let (slope, r2) = linfit(&xs, &ys);
    let speed = if slope.abs() > 1e-6 {
        1.0 / slope.abs()
    } else {
        f32::NAN
    };

    let heading_sum: Vec3 = world.velocity.iter().map(|v| forward(*v)).sum();
    let shape_after = super::analysis::compute(world).i2_i1;

    TurnWave {
        speed,
        r2,
        samples: xs.len(),
        turned_fraction: turned as f32 / n as f32,
        polarization_after: heading_sum.length() / n as f32,
        shape_ratio: if shape_before > 1e-6 {
            shape_after / shape_before
        } else {
            f32::NAN
        },
        stayed_connected: !disconnected,
        time_span: span(&ys),
        distance_span: span(&xs),
        points: xs.iter().copied().zip(ys.iter().copied()).collect(),
    }
}

/// `max - min` of a slice, or `0` if empty.
fn span(v: &[f32]) -> f32 {
    if v.is_empty() {
        return 0.0;
    }
    v.iter().copied().fold(f32::NEG_INFINITY, f32::max)
        - v.iter().copied().fold(f32::INFINITY, f32::min)
}

/// Ordinary least-squares fit `y = a + b·x`; returns `(slope, r²)`.
fn linfit(xs: &[f32], ys: &[f32]) -> (f32, f32) {
    let n = xs.len() as f32;
    if n < 2.0 {
        return (f32::NAN, f32::NAN);
    }
    let mx = xs.iter().sum::<f32>() / n;
    let my = ys.iter().sum::<f32>() / n;
    let mut sxx = 0.0;
    let mut sxy = 0.0;
    for (x, y) in xs.iter().zip(ys) {
        sxx += (x - mx) * (x - mx);
        sxy += (x - mx) * (y - my);
    }
    if sxx < 1e-9 {
        return (f32::NAN, f32::NAN);
    }
    let slope = sxy / sxx;
    let intercept = my - slope * mx;
    let mut ss_res = 0.0;
    let mut ss_tot = 0.0;
    for (x, y) in xs.iter().zip(ys) {
        let pred = intercept + slope * x;
        ss_res += (y - pred) * (y - pred);
        ss_tot += (y - my) * (y - my);
    }
    let r2 = if ss_tot > 1e-9 {
        1.0 - ss_res / ss_tot
    } else {
        f32::NAN
    };
    (slope, r2)
}

fn empty() -> TurnWave {
    TurnWave {
        speed: f32::NAN,
        r2: f32::NAN,
        samples: 0,
        turned_fraction: 0.0,
        polarization_after: f32::NAN,
        shape_ratio: f32::NAN,
        stayed_connected: true,
        time_span: 0.0,
        distance_span: 0.0,
        points: Vec::new(),
    }
}
