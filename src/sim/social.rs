//! The social and environmental forces a bird sums into its steering force.
//!
//! Separation, centrality and cohesion, alignment and the roost pull follow the
//! StarDisplay equations (Eq. 4–14). A random force stands in for perception
//! and control noise.

use glam::Vec3;
use rand::Rng;
use rand_chacha::ChaCha8Rng;

use super::escape::{self, EscapeState};
use super::falcon::Falcon;
use super::flight::{forward, side_of};
use super::neighbours::Grid;
use super::params::*;

/// The result of one bird's decision: its new steering force and interaction
/// radius, plus the neighbourhood statistics used for analysis.
pub struct Decision {
    /// Steering force held until the next decision [N].
    pub force: Vec3,
    /// Adaptive interaction radius for the next interval [m].
    pub radius: f32,
    /// Number of neighbours within the current radius `|N_i|`.
    pub neighbours: u32,
    /// Centrality `C_i` (Eq. 7): near 0 inside the flock, ~0.5–0.75 at the edge.
    pub centrality: f32,
    /// Sign of a zigzag to start now (+1 / −1), if the copying rule fired.
    pub start_zigzag: Option<f32>,
}

/// Computes bird `i`'s steering force and next interaction radius.
#[allow(clippy::too_many_arguments)]
pub fn decide(
    i: usize,
    positions: &[Vec3],
    velocities: &[Vec3],
    radii: &[f32],
    grid: &Grid,
    rng: &mut ChaCha8Rng,
    z0: f32,
    falcon: Option<&Falcon>,
    escape: &[EscapeState],
    time: f32,
    zigzag_k: u32,
) -> Decision {
    let p = positions[i];
    let r = radii[i];
    let f_i = forward(velocities[i]);

    let mut sep = Vec3::ZERO;
    let mut coh = Vec3::ZERO;
    let mut align = Vec3::ZERO;
    let mut n_all = 0u32;
    let mut n_star = 0u32;
    let mut n_align = 0u32;
    let mut zigzag_neighbours = 0u32;

    // One pass over the topological neighbourhood (Eq. 3).
    grid.for_each_within(p, r, positions, |j, dist| {
        if j == i {
            return;
        }
        n_all += 1;
        let dhat = (positions[j] - p) / dist.max(1e-6);

        // Separation over all neighbours, no blind angle (Eq. 4).
        let g = if dist <= R_H {
            1.0
        } else {
            (-((dist - R_H) / SIGMA).powi(2)).exp()
        };
        sep -= dhat * g;

        // Cohesion and alignment use the reduced neighbourhood N*_i (Eq. 6).
        if f_i.dot(dhat) >= BLIND_COS {
            align += forward(velocities[j]);
            n_align += 1;
            if dist > R_H {
                coh += dhat;
                n_star += 1;
            }
        }

        // Neighbours that started a zigzag within the last reaction time.
        if time - escape[j].zigzag_start <= DU {
            zigzag_neighbours += 1;
        }
    });

    let mut force = Vec3::ZERO;
    if n_all > 0 {
        force += sep * (W_SEP / n_all as f32);
    }

    // Centrality is measured over a wider radius, 2R_i (Eq. 7).
    let mut csum = Vec3::ZERO;
    let mut n_g = 0u32;
    grid.for_each_within(p, 2.0 * r, positions, |j, dist| {
        if j == i {
            return;
        }
        csum += (positions[j] - p) / dist.max(1e-6);
        n_g += 1;
    });
    let centrality = if n_g > 0 {
        csum.length() / n_g as f32
    } else {
        0.0
    };

    // Cohesion, stronger at the border through C_i (Eq. 5).
    if n_star > 0 {
        force += coh * (W_COH * centrality / n_star as f32);
    }

    // Alignment (Eq. 8): proportional to the misalignment from the mean
    // neighbour heading. Normalising this made a nearly-aligned bird receive a
    // full-strength kick in a random direction, which was the "neighbour noise"
    // that saturated the bank.
    let a = if n_align > 0 {
        align / n_align as f32 - f_i
    } else {
        Vec3::ZERO
    };
    force += a * W_ALIGN;

    // Horizontal roost pull (Eq. 11): a lateral steering force, stronger when
    // heading outward, that turns the bird back toward the roost. `w_RoostH` is
    // in N/m, so the force grows with the distance beyond the boundary. The
    // lateral direction is the one whose component points inward.
    let horiz = Vec3::new(p.x, p.y, 0.0);
    let dist_h = horiz.length();
    if dist_h > R_ROOST {
        let n = horiz / dist_h;
        let side = side_of(f_i);
        let inward_side = side * side.dot(-n);
        let outward = 0.5 + 0.5 * f_i.dot(n);
        force += inward_side * (W_ROOST_H * (dist_h - R_ROOST) * outward);
    }

    // Vertical roost pull toward the preferred height (Eq. 12), capped at
    // `W_ROOST_V` so it is not an arbitrarily stiff spring. It ramps up over
    // `H_ROOST_V` metres and is constant beyond.
    force += Vec3::Z * (W_ROOST_V * ((z0 - p.z) / H_ROOST_V).clamp(-1.0, 1.0));

    // Speed control: relax toward the cruise speed (Eq. 1, part of Eq. 14).
    let speed = velocities[i].length();
    force += f_i * ((MASS / TAU) * (V0 - speed));

    // Random force (Eq. 13).
    force += random_unit(rng) * W_XI;

    // Flee and flash expansion.
    force += escape::force(p, falcon, escape[i].flash_until, time);

    // Copy a zigzag: a non-alarmed bird starts one when at least `zigzag_k` of
    // its neighbours started one within the last reaction time.
    let start_zigzag = if !escape[i].alarmed
        && !escape[i].zigzag_active
        && time >= escape[i].cooldown_until
        && zigzag_neighbours >= zigzag_k
    {
        Some(if rng.random::<bool>() { 1.0 } else { -1.0 })
    } else {
        None
    };

    Decision {
        force,
        radius: radius_step(r, n_all),
        neighbours: n_all,
        centrality,
        start_zigzag,
    }
}

/// A uniformly distributed unit vector.
pub fn random_unit(rng: &mut ChaCha8Rng) -> Vec3 {
    let z: f32 = rng.random_range(-1.0..1.0);
    let theta: f32 = rng.random_range(0.0..std::f32::consts::TAU);
    let r = (1.0 - z * z).sqrt();
    Vec3::new(r * theta.cos(), r * theta.sin(), z)
}
