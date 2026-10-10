//! The evening: arrival, display, roosting and night.
//!
//! One evening lasts `--arc-minutes` (default 15). Groups of starlings arrive,
//! merge into a display flock, pour down into the reeds, and night falls before
//! the next evening starts. `--no-arc` holds the display phase forever.

use glam::Vec3;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use super::World;
use super::params::*;

/// Phase of the evening.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    /// First ~15 %: groups arrive from outside the view.
    Arrival,
    /// Up to ~70 %: the flock displays over the reeds.
    Display,
    /// Up to ~90 %: the flock pours down and lands.
    Roosting,
    /// The rest: night, then a new evening.
    Night,
}

/// The evening clock.
pub struct Arc {
    /// Length of one evening [s].
    pub duration: f32,
    /// Time within the current evening [s].
    pub time: f32,
    base_z0: f32,
    rng: ChaCha8Rng,
    spawn_timer: f32,
}

impl Arc {
    /// A new evening of `minutes` minutes.
    pub fn new(minutes: f32, seed: u64) -> Self {
        Self {
            duration: (minutes * 60.0).max(1.0),
            time: 0.0,
            base_z0: Z0,
            rng: ChaCha8Rng::seed_from_u64(seed ^ 0x00A2_C0FE),
            spawn_timer: 0.0,
        }
    }

    /// The current phase.
    pub fn phase(&self) -> Phase {
        let f = self.time / self.duration;
        if f < 0.15 {
            Phase::Arrival
        } else if f < 0.70 {
            Phase::Display
        } else if f < 0.90 {
            Phase::Roosting
        } else {
            Phase::Night
        }
    }

    /// How far into the evening we are, `0` (bright gold) to `1` (night).
    pub fn evening(&self) -> f32 {
        let f = (self.time / self.duration).clamp(0.0, 1.0);
        if f < 0.15 {
            f / 0.15 * 0.35
        } else if f < 0.70 {
            0.35 + (f - 0.15) / 0.55 * 0.25
        } else if f < 0.90 {
            0.60 + (f - 0.70) / 0.20 * 0.30
        } else {
            0.90 + (f - 0.90) / 0.10 * 0.10
        }
    }

    /// Advances the evening and its effect on the world.
    pub fn update(&mut self, dt: f32, world: &mut World) {
        self.time += dt;
        if self.time >= self.duration {
            self.time -= self.duration;
        }
        match self.phase() {
            Phase::Arrival => {
                world.z0 = self.base_z0;
                self.spawn_timer -= dt;
                while self.spawn_timer <= 0.0 && world.len() < world.target_birds {
                    let remaining = world.target_birds - world.len();
                    let group = self.rng.random_range(50..=300).min(remaining.max(1));
                    self.spawn_group(world, group);
                    self.spawn_timer += 0.5;
                }
                if self.spawn_timer < 0.0 {
                    self.spawn_timer = 0.0;
                }
            }
            Phase::Display => {}
            Phase::Roosting => {
                let f = ((self.time / self.duration) - 0.70) / 0.20;
                world.z0 = self.base_z0 + (REED_TOP - self.base_z0) * f.clamp(0.0, 1.0);
                world.remove_landed();
            }
            Phase::Night => {
                world.z0 = REED_TOP;
                world.clear();
            }
        }
    }

    fn spawn_group(&mut self, world: &mut World, count: usize) {
        let bearing = self.rng.random_range(0.0..std::f32::consts::TAU);
        let radius = self.rng.random_range(250.0..350.0);
        let height = self.base_z0 + self.rng.random_range(-20.0..60.0);
        let base = Vec3::new(radius * bearing.cos(), radius * bearing.sin(), height);
        // Each group heads for its own point over the roost, so groups merge
        // rather than all converging on the centre.
        let target = Vec3::new(
            self.rng.random_range(-90.0..90.0),
            self.rng.random_range(-90.0..90.0),
            self.base_z0,
        );
        for _ in 0..count {
            let pos = base
                + Vec3::new(
                    self.rng.random_range(-30.0..30.0),
                    self.rng.random_range(-30.0..30.0),
                    self.rng.random_range(-10.0..10.0),
                );
            let toward = target - pos;
            let dir = (toward.normalize_or_zero()
                + Vec3::new(
                    self.rng.random_range(-0.2..0.2),
                    self.rng.random_range(-0.2..0.2),
                    self.rng.random_range(-0.1..0.1),
                ))
            .normalize_or_zero();
            world.push_bird(pos, dir * V0, &mut self.rng);
        }
    }
}
