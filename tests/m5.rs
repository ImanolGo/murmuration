//! M5: the evening arc.

use murmuration::sim::World;
use murmuration::sim::arc::Phase;
use murmuration::sim::params::DT;

/// A short evening runs arrival → display → roosting → night and loops without
/// the flock growing.
#[test]
fn m5_evening_loops_without_growth() {
    let target = 500;
    let mut world = World::new(target, 42);
    world.enable_arc(0.1); // 6-second evening
    let duration = world.arc.as_ref().map(|a| a.duration).unwrap_or(6.0);

    let mut max_birds = 0;
    let mut seen = [false; 4];
    let evenings = 3.0;
    let steps = (evenings * duration / DT) as usize;
    for _ in 0..steps {
        world.step(DT);
        max_birds = max_birds.max(world.len());
        if let Some(arc) = &world.arc {
            seen[match arc.phase() {
                Phase::Arrival => 0,
                Phase::Display => 1,
                Phase::Roosting => 2,
                Phase::Night => 3,
            }] = true;
        }
    }

    assert!(
        seen[0] && seen[1] && seen[2] && seen[3],
        "did not see all phases"
    );
    assert!(
        max_birds <= target,
        "flock grew past the target: {max_birds} > {target}"
    );
    assert!(world.len() <= target);
}

/// Birds actually arrive during the arrival phase.
#[test]
fn m5_birds_arrive() {
    let mut world = World::new(300, 7);
    world.enable_arc(0.1);
    for _ in 0..(1.0 / DT) as usize {
        world.step(DT);
    }
    assert!(!world.is_empty(), "no birds arrived during arrival");
}
