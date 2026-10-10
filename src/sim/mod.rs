//! The simulation: pure, headless and deterministic for a given seed.
//!
//! [`World`] owns the flock as a struct of arrays and advances it in fixed
//! integration steps. It knows nothing about the terminal.

pub mod analysis;
pub mod flight;
pub mod neighbours;
pub mod params;
pub mod social;
pub mod turns;

use glam::Vec3;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use neighbours::Grid;
use params::*;

/// The flock and its environment.
pub struct World {
    /// Simulated time [s].
    pub time: f32,
    /// Preferred flight height, the roost "zero level" [m].
    pub z0: f32,
    /// Bird positions [m].
    pub position: Vec<Vec3>,
    /// Bird velocities [m/s].
    pub velocity: Vec<Vec3>,
    /// Bank angle per bird [rad].
    pub roll: Vec<f32>,
    /// Adaptive interaction radius per bird [m].
    pub radii: Vec<f32>,
    /// Steering force held since each bird's last decision [N].
    pub steering: Vec<Vec3>,
    /// Simulated time of each bird's next decision [s].
    pub next_decision: Vec<f32>,
    /// Neighbour count `|N_i|` from each bird's last decision.
    pub neighbour_count: Vec<u32>,
    /// Centrality `C_i` from each bird's last decision.
    pub centrality: Vec<f32>,
    /// Per-bird random streams, so decisions are order-independent.
    rngs: Vec<ChaCha8Rng>,
    grid: Grid,
    /// The seed this world was created with.
    pub seed: u64,
}

impl World {
    /// Creates `n` birds in a compact, slightly flattened cloud near the roost,
    /// all moving together at cruise speed. (The paper starts them at random
    /// positions and orientations over the roost; the evening arc will spawn
    /// arriving groups instead.)
    pub fn new(n: usize, seed: u64) -> Self {
        let z0 = Z0;
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut position = Vec::with_capacity(n);
        let mut velocity = Vec::with_capacity(n);
        let mut roll = Vec::with_capacity(n);
        let mut radii = Vec::with_capacity(n);
        let mut next_decision = Vec::with_capacity(n);
        let mut rngs = Vec::with_capacity(n);
        for i in 0..n {
            let dir = social::random_unit(&mut rng);
            let rad = 18.0 * rng.random::<f32>().cbrt();
            position.push(Vec3::new(dir.x * rad, dir.y * rad, z0 + dir.z * rad * 0.4));
            velocity.push((Vec3::X + social::random_unit(&mut rng) * 0.05).normalize() * V0);
            roll.push(0.0);
            radii.push(5.0);
            next_decision.push(rng.random_range(0.0..DU));
            rngs.push(ChaCha8Rng::seed_from_u64(mix(seed, i as u64)));
        }
        let grid = Grid::new(
            GRID_CELL,
            Vec3::new(-R_ROOST - 60.0, -R_ROOST - 60.0, -40.0),
            Vec3::new(R_ROOST + 60.0, R_ROOST + 60.0, z0 + 200.0),
        );
        Self {
            time: 0.0,
            z0,
            position,
            velocity,
            roll,
            radii,
            steering: vec![Vec3::ZERO; n],
            next_decision,
            neighbour_count: vec![0; n],
            centrality: vec![0.0; n],
            rngs,
            grid,
            seed,
        }
    }

    /// Number of birds.
    pub fn len(&self) -> usize {
        self.position.len()
    }

    /// Whether the flock is empty.
    pub fn is_empty(&self) -> bool {
        self.position.is_empty()
    }

    /// Advances the world by one integration step.
    pub fn step(&mut self, dt: f32) {
        self.time += dt;
        self.grid.rebuild(&self.position);
        let time = self.time;
        let z0 = self.z0;

        // Decisions for every bird whose reaction is due.
        {
            let Self {
                position,
                velocity,
                radii,
                steering,
                next_decision,
                neighbour_count,
                centrality,
                rngs,
                grid,
                ..
            } = self;
            for i in 0..position.len() {
                if time >= next_decision[i] {
                    let d = social::decide(i, position, velocity, radii, grid, &mut rngs[i], z0);
                    steering[i] = d.force;
                    radii[i] = d.radius;
                    neighbour_count[i] = d.neighbours;
                    centrality[i] = d.centrality;
                    next_decision[i] += DU;
                }
            }
        }

        // Flight integration for every bird.
        {
            let Self {
                position,
                velocity,
                roll,
                steering,
                ..
            } = self;
            for i in 0..position.len() {
                flight::step_bird(
                    &mut position[i],
                    &mut velocity[i],
                    &mut roll[i],
                    steering[i],
                    dt,
                );
            }
        }
    }
}

/// SplitMix64, for deriving independent per-bird seeds from one master seed.
fn mix(seed: u64, i: u64) -> u64 {
    let mut z = seed.wrapping_add(i.wrapping_mul(0x9E37_79B9_7F4A_7C15));
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn determinism_for_a_seed() {
        let mut a = World::new(200, 7);
        let mut b = World::new(200, 7);
        for _ in 0..100 {
            a.step(DT);
            b.step(DT);
        }
        assert_eq!(a.position, b.position);
        assert_eq!(a.velocity, b.velocity);
    }

    #[test]
    fn different_seeds_differ() {
        let mut a = World::new(200, 1);
        let mut b = World::new(200, 2);
        for _ in 0..100 {
            a.step(DT);
            b.step(DT);
        }
        assert_ne!(a.position, b.position);
    }

    #[test]
    fn interaction_radius_relaxes() {
        let mut w = World::new(1000, 1);
        for _ in 0..2000 {
            w.step(DT);
        }
        let mean_r = w.radii.iter().sum::<f32>() / w.len() as f32;
        let mean_n = w.neighbour_count.iter().map(|&c| c as f32).sum::<f32>() / w.len() as f32;
        assert!(mean_r < 20.0, "mean R blew up: {mean_r}");
        assert!(mean_n < 40.0, "mean |N| blew up: {mean_n}");
    }
}
