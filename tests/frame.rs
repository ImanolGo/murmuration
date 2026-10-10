//! A fixed-seed frame, snapshotted so the look can be tracked over time.

use murmuration::render::colour::Depth;
use murmuration::render::{Camera, Renderer};
use murmuration::sim::World;
use murmuration::sim::params::DT;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;

/// The braille/sky symbols of a buffer, row by row.
fn buffer_to_string(buf: &Buffer) -> String {
    let area = buf.area();
    let mut out = String::new();
    for y in 0..area.height {
        for x in 0..area.width {
            out.push_str(buf[(x, y)].symbol());
        }
        out.push('\n');
    }
    out
}

#[test]
fn fixed_seed_frame() {
    let mut world = World::new(2000, 42);
    for _ in 0..(30.0 / DT) as usize {
        world.step(DT);
    }

    let mut camera = Camera::new();
    for _ in 0..120 {
        camera.update(&world, 1.0 / 30.0);
    }

    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    let mut renderer = Renderer::new(Depth::TrueColor);
    terminal
        .draw(|frame| renderer.draw(frame, &world, &camera))
        .unwrap();

    insta::assert_snapshot!(buffer_to_string(terminal.backend().buffer()));
}

#[test]
fn all_colour_depths_render() {
    for depth in [Depth::TrueColor, Depth::Ansi256, Depth::Mono] {
        let mut world = World::new(200, 3);
        for _ in 0..200 {
            world.step(murmuration::sim::params::DT);
        }
        let mut camera = Camera::new();
        camera.update(&world, 1.0);
        let mut terminal = Terminal::new(TestBackend::new(40, 12)).unwrap();
        let mut renderer = Renderer::new(depth);
        terminal
            .draw(|frame| renderer.draw(frame, &world, &camera))
            .unwrap();
    }
}
