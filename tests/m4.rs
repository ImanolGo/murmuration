//! M4: falcon, escape and the agitation (zigzag) wave.

use murmuration::sim::World;
use murmuration::sim::escape;

/// The zigzag copying wave propagates and its arrival times are roughly linear
/// in distance (a clean front).
#[test]
fn m4_agitation_wave_propagates() {
    let mut world = World::new(2000, 42);
    world.zigzag_k = 2;
    let wave = escape::measure_agitation(&mut world, 3.0);
    println!(
        "k=2 agitation: speed {:.1} m/s, R2 {:.2}, reached {:.0}%, expansion {:.0}%",
        wave.speed,
        wave.r2,
        wave.peak_alarmed * 100.0,
        wave.peak_expansion * 100.0,
    );
    assert!(
        wave.samples > 50,
        "zigzag wave did not propagate: only {} birds",
        wave.samples
    );
    assert!(
        wave.r2 > 0.5,
        "agitation wave not a clean front: R2 = {}",
        wave.r2
    );
    assert!(
        (3.0..=25.0).contains(&wave.speed),
        "agitation wave speed = {:.1} m/s, target 3-25 m/s",
        wave.speed
    );
}

/// A higher copying threshold should reach fewer birds (a weaker/slower wave).
#[test]
fn m4_threshold_weakens_the_wave() {
    let mut loose = World::new(2000, 42);
    loose.zigzag_k = 1;
    let a = escape::measure_agitation(&mut loose, 3.0);

    let mut strict = World::new(2000, 42);
    strict.zigzag_k = 5;
    let b = escape::measure_agitation(&mut strict, 3.0);

    assert!(
        a.samples > b.samples,
        "k=1 reached {} birds, k=5 reached {}",
        a.samples,
        b.samples
    );
}

/// Copied zigzags take the majority direction, so the dark band is coherent
/// rather than a mix of opposite rolls (review B3).
#[test]
fn copied_zigzags_share_a_direction() {
    let mut world = World::new(2000, 42);
    world.zigzag_k = 2;
    escape::measure_agitation(&mut world, 3.0);

    let signs: Vec<f32> = world
        .escape
        .iter()
        .filter(|e| e.zigzag_start.is_finite())
        .map(|e| e.zigzag_sign)
        .collect();
    assert!(signs.len() > 50, "too few zigzags: {}", signs.len());
    let positive = signs.iter().filter(|&&s| s > 0.0).count();
    let fraction = positive as f32 / signs.len() as f32;
    assert!(
        fraction > 0.8,
        "copied zigzags are not coherent: {:.0}% roll one way",
        fraction * 100.0
    );
}
