//! Benchmarks (ignored by default).
//!
//! Run with:
//! `cargo test --release --test perf -- --ignored --nocapture`

use std::time::Instant;

use murmuration::render::colour::Depth;
use murmuration::render::{Camera, Renderer};
use murmuration::sim::World;
use murmuration::sim::params::DT;
use ratatui::Terminal;
use ratatui::backend::TestBackend;

/// One simulated frame at 30 fps is about seven 5 ms integration steps.
const STEPS_PER_FRAME: usize = 7;

#[test]
#[ignore = "benchmark"]
fn render_fps_4000_birds() {
    for (w, h) in [(80u16, 24u16), (200, 60)] {
        let mut world = World::new(4000, 7);
        let mut camera = Camera::new();
        let mut renderer = Renderer::new(Depth::TrueColor);
        let aspect = (w as f32 * 2.0) / (h as f32 * 4.0);
        camera.update(&world, 1.0, aspect);
        let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
        for _ in 0..STEPS_PER_FRAME {
            world.step(DT);
        }

        let frames = 120;
        let start = Instant::now();
        for _ in 0..frames {
            for _ in 0..STEPS_PER_FRAME {
                world.step(DT);
            }
            camera.update(&world, 1.0 / 30.0, aspect);
            terminal
                .draw(|f| renderer.draw(f, &world, &camera))
                .unwrap();
        }
        let elapsed = start.elapsed().as_secs_f64();
        println!(
            "{w}x{h}: {:.1} fps ({:.2} ms/frame)",
            frames as f64 / elapsed,
            1000.0 * elapsed / frames as f64
        );
    }
}
