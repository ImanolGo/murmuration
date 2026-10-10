//! The hard acceptance tests for M1: a headless flock with no falcon.
//!
//! Targets are from PLAN.md and the papers it cites (StarDisplay for the
//! topological range, Ballerini et al. 2008b for the rest).

use murmuration::sim::World;
use murmuration::sim::analysis::{self, Stats};
use murmuration::sim::params::DT;

/// Steps a fresh world and returns a statistic sample every simulated second.
fn run_and_sample(birds: usize, seed: u64, seconds: f32) -> Vec<Stats> {
    let mut world = World::new(birds, seed);
    let steps = (seconds / DT) as usize;
    let per_sample = (1.0 / DT).round() as usize;
    let mut samples = Vec::new();
    for s in 1..=steps {
        world.step(DT);
        if s % per_sample == 0 {
            samples.push(analysis::compute(&world));
        }
    }
    samples
}

fn mean(samples: &[Stats], f: impl Fn(&Stats) -> f32) -> f32 {
    samples.iter().map(f).sum::<f32>() / samples.len() as f32
}

#[test]
fn m1_hard_criteria_2000_birds() {
    let samples = run_and_sample(2000, 42, 120.0);
    // Measure once the flock has settled (after the first 30 s).
    let settled = &samples[30..];

    let neighbours = mean(settled, |s| s.mean_neighbours);
    assert!(
        (5.5..=7.5).contains(&neighbours),
        "mean topological neighbours = {neighbours}, target 6.5 ± 1"
    );

    let nnd = mean(settled, |s| s.mean_nnd);
    assert!(
        (0.7..=1.5).contains(&nnd),
        "mean nearest-neighbour distance = {nnd} m, target 0.7–1.5 m"
    );

    let flatness = mean(settled, |s| s.flatness);
    assert!(flatness > 0.85, "mean |I1·ẑ| = {flatness}, target > 0.85");

    let aligned = mean(settled, |s| s.nn_aligned_fraction);
    assert!(
        aligned < 0.10,
        "fraction of nearest neighbours ahead/behind = {aligned}, isotropic baseline 0.10"
    );

    let min_cluster = settled
        .iter()
        .map(|s| s.largest_cluster_fraction)
        .fold(1.0f32, f32::min);
    assert!(
        min_cluster >= 0.95,
        "largest single-link cluster at 5 m fell to {min_cluster}"
    );
}
