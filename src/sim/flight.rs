//! Fixed-wing flight: lift, drag, thrust, gravity and banked turning.
//!
//! Follows the StarDisplay flight model. Turning comes from banked lift rather
//! than from rotating the heading directly.

use glam::Vec3;

use super::params::*;

/// Unit forward direction of a velocity, with a stable fallback.
pub fn forward(v: Vec3) -> Vec3 {
    if v.length_squared() > 1e-12 {
        v.normalize()
    } else {
        Vec3::X
    }
}

/// The bird's sideways direction: `normalize(f × ẑ)`, with a fallback when
/// `f` is near vertical.
pub fn side_of(f: Vec3) -> Vec3 {
    let s = f.cross(Vec3::Z);
    if s.length_squared() > 1e-8 {
        s.normalize()
    } else {
        Vec3::X
    }
}

/// The bird's body "up" for a bank angle `beta`, given forward `f`.
///
/// The body frame is `(f, side, up)`; banking rotates `up0 = side × f` about
/// `f` by `beta`, which simplifies to `up0·cos β + side·sin β`.
pub fn body_up(f: Vec3, beta: f32) -> Vec3 {
    let side = side_of(f);
    let up0 = side.cross(f);
    up0 * beta.cos() + side * beta.sin()
}

/// Advances one bird by `dt` under its held steering force. When
/// `forced_roll` is `Some`, that bank angle overrides the steering-driven
/// banking (used by the zigzag manoeuvre).
pub fn step_bird(
    p: &mut Vec3,
    v: &mut Vec3,
    roll: &mut f32,
    steering: Vec3,
    forced_roll: Option<f32>,
    dt: f32,
) {
    let f = forward(*v);
    let side = side_of(f);
    let speed = v.length().max(1e-6);

    // Bank into the lateral component of the steering force (Eq. 18–21).
    let al = steering.dot(side) / MASS;
    let beta_in = (W_BETA_IN * al * dt).atan();
    let beta_out = (W_BETA_OUT * roll.sin() * dt).atan();
    let beta = match forced_roll {
        Some(target) => target.clamp(-MAX_BANK, MAX_BANK),
        None => (*roll + beta_in - beta_out).clamp(-MAX_BANK, MAX_BANK),
    };
    *roll = beta;

    let up = body_up(f, beta);

    // Lift (Eq. 16) along the body up. The paper adds a small compensation for
    // the effective lift lost while banked; its magnitude is ambiguous there,
    // and experiments showed it pumps energy, so it is omitted for now (M3 will
    // revisit banking).
    let lift = (speed / V0).powi(2) * L0;
    // Drag (Eq. 16) against the direction of travel.
    let drag = (speed / V0).powi(2) * D0;
    // Default thrust along the forward direction. The speed-control force is
    // part of the steering force.
    let flight_force = up * lift - f * drag + f * D0 - Vec3::Z * MASS * G;

    // Euler integration (Eq. 22–23).
    *v += (steering + flight_force) / MASS * dt;
    let sp = v.length();
    if sp > V_MAX {
        *v *= V_MAX / sp;
    } else if sp < V_MIN && sp > 1e-6 {
        *v *= V_MIN / sp;
    }
    *p += *v * dt;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_cruise_holds_height() {
        let mut p = Vec3::new(0.0, 0.0, 100.0);
        let mut v = Vec3::X * V0;
        let mut roll = 0.0;
        for _ in 0..200 {
            step_bird(&mut p, &mut v, &mut roll, Vec3::ZERO, None, DT);
        }
        assert!((p.z - 100.0).abs() < 1.0, "sank or climbed: z = {}", p.z);
        assert!(
            (v.length() - V0).abs() < 0.5,
            "speed drifted: {}",
            v.length()
        );
    }

    #[test]
    fn banked_bird_turns() {
        // A sustained sideways steering force should curl the path.
        let mut p = Vec3::new(0.0, 0.0, 100.0);
        let mut v = Vec3::X * V0;
        let mut roll = 0.0;
        for _ in 0..400 {
            // Re-decide a small rightward force each step.
            step_bird(&mut p, &mut v, &mut roll, Vec3::Y * 0.5, None, DT);
        }
        assert!(roll.abs() > 0.05, "did not bank: beta = {roll}");
        assert!(v.y.abs() > 1.0, "did not turn: v = {v}");
    }
}
