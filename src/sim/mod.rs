//! The simulation: pure, headless and deterministic for a given seed.
//!
//! [`World`] owns the flock as a struct of arrays and advances it in fixed
//! integration steps. It knows nothing about the terminal.

pub mod analysis;
pub mod arc;
pub mod escape;
pub mod falcon;
pub mod flight;
pub mod neighbours;
pub mod params;
pub mod social;
pub mod turns;

use glam::Vec3;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use arc::Arc;
use escape::EscapeState;
use falcon::Falcon;
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
    /// Per-bird escape state (alarm, zigzag, flash expansion).
    pub escape: Vec<EscapeState>,
    /// The falcon, if one is hunting.
    pub falcon: Option<Falcon>,
    /// Number of neighbours whose zigzag starts a copy (1–7).
    pub zigzag_k: u32,
    /// The evening arc, if one is running.
    pub arc: Option<Arc>,
    /// Target flock size the arc aims for during arrival.
    pub target_birds: usize,
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
            escape: vec![EscapeState::default(); n],
            falcon: None,
            zigzag_k: ZIGZAG_K,
            arc: None,
            target_birds: n,
            rngs,
            grid,
            seed,
        }
    }

    /// Starts a new evening of `minutes` minutes, from an empty sky.
    pub fn enable_arc(&mut self, minutes: f32) {
        self.clear();
        self.arc = Some(Arc::new(minutes, self.seed));
    }

    /// Adds one bird with the given position and velocity.
    pub fn push_bird(&mut self, pos: Vec3, vel: Vec3, rng: &mut ChaCha8Rng) {
        let i = self.len();
        self.position.push(pos);
        self.velocity.push(vel);
        self.roll.push(0.0);
        self.radii.push(5.0);
        self.steering.push(Vec3::ZERO);
        self.next_decision
            .push(self.time + rng.random_range(0.0..DU));
        self.neighbour_count.push(0);
        self.centrality.push(0.0);
        self.escape.push(EscapeState::default());
        self.rngs
            .push(ChaCha8Rng::seed_from_u64(mix(self.seed, i as u64)));
    }

    /// Removes the bird at `i`, swapping the last bird into its place.
    fn remove_bird(&mut self, i: usize) {
        self.position.swap_remove(i);
        self.velocity.swap_remove(i);
        self.roll.swap_remove(i);
        self.radii.swap_remove(i);
        self.steering.swap_remove(i);
        self.next_decision.swap_remove(i);
        self.neighbour_count.swap_remove(i);
        self.centrality.swap_remove(i);
        self.escape.swap_remove(i);
        self.rngs.swap_remove(i);
    }

    /// Removes birds that have landed below the reeds near the roost.
    pub fn remove_landed(&mut self) {
        let mut i = 0;
        while i < self.len() {
            let p = self.position[i];
            let horizontal = (p.x * p.x + p.y * p.y).sqrt();
            if p.z < REED_TOP && horizontal < R_ROOST {
                self.remove_bird(i);
            } else {
                i += 1;
            }
        }
    }

    /// Removes every bird.
    pub fn clear(&mut self) {
        self.position.clear();
        self.velocity.clear();
        self.roll.clear();
        self.radii.clear();
        self.steering.clear();
        self.next_decision.clear();
        self.neighbour_count.clear();
        self.centrality.clear();
        self.escape.clear();
        self.rngs.clear();
    }

    /// Adds `count` birds near the flock centroid.
    pub fn add_birds(&mut self, count: usize, rng: &mut ChaCha8Rng) {
        let centre = if self.is_empty() {
            Vec3::new(0.0, 0.0, self.z0)
        } else {
            self.position.iter().copied().sum::<Vec3>() / self.len() as f32
        };
        for _ in 0..count {
            let pos = centre + social::random_unit(rng) * rng.random_range(0.0..30.0);
            let vel = social::random_unit(rng) * V0;
            self.push_bird(pos, vel, rng);
        }
    }

    /// Removes up to `count` birds.
    pub fn remove_birds(&mut self, count: usize) {
        for _ in 0..count {
            if self.is_empty() {
                break;
            }
            let last = self.len() - 1;
            self.remove_bird(last);
        }
    }

    /// Evening brightness: 0 bright gold, 1 night. Display dusk without an arc.
    pub fn evening(&self) -> f32 {
        match &self.arc {
            Some(arc) => arc.evening(),
            None => 0.5,
        }
    }

    /// Adds an autonomous falcon to the world.
    pub fn spawn_falcon(&mut self) {
        if self.falcon.is_none() {
            self.falcon = Some(Falcon::new(self.seed, self.z0));
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
        if let Some(mut arc) = self.arc.take() {
            arc.update(dt, self);
            self.arc = Some(arc);
        }
        let time = self.time;
        let z0 = self.z0;

        // The falcon hunts.
        if !self.is_empty() {
            let n = self.len() as f32;
            let centroid = self.position.iter().copied().sum::<Vec3>() / n;
            let extent = self
                .position
                .iter()
                .map(|p| (*p - centroid).length())
                .fold(1.0f32, f32::max);
            if let Some(falcon) = &mut self.falcon {
                falcon.update(dt, centroid, extent, time);
            }
        }

        // Alarm, the alarm-triggered manoeuvre, and flash expansion.
        match &self.falcon {
            Some(falcon) => {
                let fpos = falcon.position;
                let fspeed = falcon.speed();
                for i in 0..self.len() {
                    let dist = (self.position[i] - fpos).length();
                    let e = &mut self.escape[i];
                    if dist < R_THREAT {
                        if !e.zigzag_active && time >= e.cooldown_until {
                            e.zigzag_start = time;
                            e.zigzag_active = true;
                            e.zigzag_sign = if i % 2 == 0 { 1.0 } else { -1.0 };
                        }
                        e.alarmed = true;
                        if dist < R_FLASH && fspeed > V_FLASH {
                            e.flash_until = time + FLASH_DURATION;
                        }
                    } else {
                        e.alarmed = false;
                    }
                }
            }
            None => {
                for e in &mut self.escape {
                    e.alarmed = false;
                }
            }
        }

        self.grid.rebuild(&self.position);
        let k = self.zigzag_k;

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
                escape,
                falcon,
                ..
            } = self;
            for i in 0..position.len() {
                if time >= next_decision[i] {
                    let d = social::decide(
                        i,
                        position,
                        velocity,
                        radii,
                        grid,
                        &mut rngs[i],
                        z0,
                        falcon.as_ref(),
                        escape,
                        time,
                        k,
                    );
                    steering[i] = d.force;
                    radii[i] = d.radius;
                    neighbour_count[i] = d.neighbours;
                    centrality[i] = d.centrality;
                    if let Some(sign) = d.start_zigzag {
                        escape[i].zigzag_start = time;
                        escape[i].zigzag_active = true;
                        escape[i].zigzag_sign = sign;
                    }
                    next_decision[i] += DU;
                }
            }
        }

        // End finished zigzags and start their cooldown.
        for e in &mut self.escape {
            if e.zigzag_active && time - e.zigzag_start >= ZIGZAG_DURATION {
                e.zigzag_active = false;
                e.cooldown_until = time + ZIGZAG_COOLDOWN;
            }
        }

        // Flight integration for every bird.
        {
            let Self {
                position,
                velocity,
                roll,
                steering,
                escape,
                ..
            } = self;
            for i in 0..position.len() {
                let forced = escape::zigzag_roll(&escape[i], time);
                flight::step_bird(
                    &mut position[i],
                    &mut velocity[i],
                    &mut roll[i],
                    steering[i],
                    forced,
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
