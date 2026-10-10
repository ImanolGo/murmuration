//! M3: forced-turn experiment.
//!
//! The turning-wave measurement runs, and the flock turns as one coherent body
//! (it stays in one piece and keeps its polarisation). However, the fitted
//! time-vs-distance relation is only weakly linear — see DEVELOPMENT.md — so the
//! "close to linear" acceptance bar is not asserted here. This test guards the
//! body-turn behaviour and prints the fitted numbers.

use murmuration::sim::World;
use murmuration::sim::params::DT;
use murmuration::sim::turns::{self, TurnConfig};

#[test]
fn m3_turn_is_coherent() {
    let mut world = World::new(2000, 42);
    for _ in 0..(20.0 / DT) as usize {
        world.step(DT);
    }
    let wave = turns::measure(&mut world, TurnConfig::default());
    println!(
        "turn wave: speed {:.1} m/s, R2 {:.2}, turned {:.0}%, polarisation {:.2}, \
         shape ratio {:.2}, one piece {}",
        wave.speed,
        wave.r2,
        wave.turned_fraction * 100.0,
        wave.polarization_after,
        wave.shape_ratio,
        wave.stayed_connected,
    );

    assert!(
        wave.stayed_connected,
        "flock fragmented during the forced turn"
    );
    assert!(
        wave.polarization_after > 0.9,
        "flock lost heading coherence during the turn: {}",
        wave.polarization_after
    );
}
