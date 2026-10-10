//! B1: the neighbour grid is unbounded, so distant groups still flock.

use glam::Vec3;
use murmuration::sim::World;
use murmuration::sim::params::{DT, V0, Z0};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

/// A group spawned 300 m out — outside the old ±210 m grid box — finds 3–10
/// neighbours within one second.
#[test]
fn distant_group_finds_neighbours() {
    let mut world = World::new(0, 0);
    let mut rng = ChaCha8Rng::seed_from_u64(0x300);
    let base = Vec3::new(300.0, 0.0, Z0);
    for _ in 0..50 {
        let pos = base
            + Vec3::new(
                rng.random_range(-15.0..15.0),
                rng.random_range(-15.0..15.0),
                rng.random_range(-5.0..5.0),
            );
        let toward = Vec3::new(-1.0, 0.0, 0.0)
            + Vec3::new(
                rng.random_range(-0.1..0.1),
                rng.random_range(-0.1..0.1),
                rng.random_range(-0.1..0.1),
            );
        world.push_bird(pos, toward.normalize() * V0, &mut rng);
    }
    for _ in 0..(1.0 / DT) as usize {
        world.step(DT);
    }
    let n = world.neighbour_count.iter().sum::<u32>() as f32 / world.len() as f32;
    assert!(
        (3.0..=10.0).contains(&n),
        "a group spawned 300 m out has {n} neighbours, target 3-10"
    );
}
