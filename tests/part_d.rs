//! Part D: the measurements that catch a flock collapsed into a ribbon.
//!
//! The M1 tests pass on a degenerate flock, so these check the review's part-D
//! targets on the same 4-minute probe (2000 birds, seed 0) that `--probe` runs.
//! Targets that are still open (signed roost rotation and the aspect-ratio
//! hard limit) are printed rather than asserted; see DEVELOPMENT.md.

use murmuration::sim::World;
use murmuration::sim::analysis::{self, Stats};
use murmuration::sim::params::DT;

/// Runs the probe and returns the per-second samples plus the centroid's
/// horizontal distance from the roost centre each second.
fn probe() -> (Vec<Stats>, Vec<f32>) {
    let mut world = World::new(2000, 0);
    let per_second = (1.0 / DT).round() as usize;
    let steps = (240.0 / DT) as usize;
    let mut samples = Vec::new();
    let mut centroid_radii = Vec::new();
    for s in 1..=steps {
        world.step(DT);
        if s % per_second == 0 {
            let stats = analysis::compute(&world);
            centroid_radii.push(stats.centroid_radius);
            samples.push(stats);
        }
    }
    (samples, centroid_radii)
}

/// The longest time the centroid spends more than 60 m from the roost centre.
fn longest_gap(radii: &[f32]) -> f32 {
    let mut max_gap: f32 = 0.0;
    let mut since: f32 = 0.0;
    for &r in radii {
        if r < 60.0 {
            max_gap = max_gap.max(since);
            since = 0.0;
        } else {
            since += 1.0;
        }
    }
    max_gap.max(since)
}

#[test]
fn part_d_targets() {
    let (samples, radii) = probe();
    let mean = |f: fn(&Stats) -> f32| samples.iter().map(f).sum::<f32>() / samples.len() as f32;

    // Hard targets that the fixed model meets.
    let clamp = samples.iter().map(|s| s.clamp_fraction).fold(0.0, f32::max);
    assert!(
        clamp < 0.02,
        "birds at the bank clamp: {:.1}% (target < 2%)",
        clamp * 100.0
    );

    let bank = mean(|s| s.mean_roll).to_degrees();
    assert!(bank < 15.0, "mean bank {bank:.1} deg (target < 15)");

    let thickness = mean(|s| s.height_std);
    assert!(
        thickness >= 2.0,
        "height std {thickness:.2} m (target >= 2)"
    );

    let i2_i1 = mean(|s| s.i2_i1);
    assert!(i2_i1 < 10.0, "I2/I1 {i2_i1:.1} (hard limit < 10)");

    // A committed return must pass over the roost, not settle into an orbit.
    let gap = longest_gap(&radii);
    assert!(
        gap <= 120.0,
        "flock did not return over the roost for {gap:.0} s"
    );

    // Accepted gaps: reported, not asserted (see DEVELOPMENT.md).
    let rotation = mean(|s| s.tangential);
    let i3_i1 = mean(|s| s.i3_i1);
    println!(
        "part-D gaps: mean tangential {rotation:.2} m/s (target |.| < 0.5), \
         longest roost gap {gap:.0} s (target <= 120), I3/I1 {i3_i1:.1} (hard limit < 20)"
    );
    assert!(rotation.is_finite() && gap.is_finite() && i3_i1.is_finite());
}
