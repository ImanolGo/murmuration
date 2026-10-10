//! A fixed-seed frame, snapshotted so the look can be tracked over time.

use murmuration::render::colour::Depth;
use murmuration::render::{Camera, Renderer};
use murmuration::sim::World;
use murmuration::sim::params::DT;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::style::Color;

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

/// The pixel-grid aspect ratio of a `w`×`h` cell terminal.
fn aspect(w: u16, h: u16) -> f32 {
    (w as f32 * 2.0) / (h as f32 * 4.0)
}

#[test]
#[ignore = "platform-dependent: libm differences flip dither dots; run on your own machine"]
fn fixed_seed_frame() {
    let mut world = World::new(2000, 42);
    for _ in 0..(30.0 / DT) as usize {
        world.step(DT);
    }

    let mut camera = Camera::new();
    for _ in 0..120 {
        camera.update(&world, 1.0 / 30.0, aspect(80, 24));
    }

    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    let mut renderer = Renderer::new(Depth::TrueColor);
    terminal
        .draw(|frame| renderer.draw(frame, &world, &camera))
        .unwrap();

    insta::assert_snapshot!(buffer_to_string(terminal.backend().buffer()));
}

/// A portable frame check: the render contains plenty of braille (the flock)
/// and a reed silhouette. Exact dots are not compared, so this is stable across
/// platforms.
#[test]
fn frame_renders_a_flock() {
    let mut world = World::new(2000, 42);
    for _ in 0..(30.0 / DT) as usize {
        world.step(DT);
    }
    let mut camera = Camera::new();
    for _ in 0..120 {
        camera.update(&world, 1.0 / 30.0, aspect(80, 24));
    }
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    let mut renderer = Renderer::new(Depth::TrueColor);
    terminal
        .draw(|frame| renderer.draw(frame, &world, &camera))
        .unwrap();

    let buf = terminal.backend().buffer();
    let (mut braille, mut reeds) = (0u32, 0u32);
    for y in 0..24 {
        for x in 0..80 {
            if let Some(c) = buf[(x, y)].symbol().chars().next() {
                let u = c as u32;
                if (0x2800..=0x28FF).contains(&u) {
                    braille += 1;
                }
                if c == '█' {
                    reeds += 1;
                }
            }
        }
    }
    assert!(braille > 40, "too few braille cells for a flock: {braille}");
    assert!(reeds > 50, "no reed silhouette: {reeds}");
}

#[test]
fn all_colour_depths_render() {
    for depth in [Depth::TrueColor, Depth::Ansi256, Depth::Mono] {
        let mut world = World::new(200, 3);
        for _ in 0..200 {
            world.step(murmuration::sim::params::DT);
        }
        let mut camera = Camera::new();
        camera.update(&world, 1.0, aspect(40, 12));
        let mut terminal = Terminal::new(TestBackend::new(40, 12)).unwrap();
        let mut renderer = Renderer::new(depth);
        terminal
            .draw(|frame| renderer.draw(frame, &world, &camera))
            .unwrap();
    }
}

/// A settled flock fills a useful part of the screen (review A5, part D).
#[test]
fn settled_flock_covers_the_screen() {
    let mut world = World::new(2000, 0);
    for _ in 0..(240.0 / DT) as usize {
        world.step(DT);
    }
    let (w, h) = (120u16, 36u16);
    let mut camera = Camera::new();
    for _ in 0..300 {
        camera.update(&world, 1.0 / 30.0, aspect(w, h));
    }
    let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
    let mut renderer = Renderer::new(Depth::TrueColor);
    terminal
        .draw(|frame| renderer.draw(frame, &world, &camera))
        .unwrap();

    let buf = terminal.backend().buffer();
    let mut lit = 0u32;
    for y in 0..h {
        for x in 0..w {
            if let Some(c) = buf[(x, y)].symbol().chars().next() {
                let u = c as u32;
                if (0x2801..=0x28FF).contains(&u) {
                    lit += 1;
                }
            }
        }
    }
    let coverage = lit as f32 / (w as f32 * h as f32);
    assert!(
        coverage >= 0.15,
        "flock covers only {:.1}% of the screen",
        coverage * 100.0
    );
}

/// Dense cells tint their background toward the ink, so the flock reads as
/// smoke rather than stippling (review C1).
#[test]
fn dense_cells_tint_the_background() {
    let mut world = World::new(2000, 42);
    for _ in 0..(30.0 / DT) as usize {
        world.step(DT);
    }
    let mut camera = Camera::new();
    for _ in 0..120 {
        camera.update(&world, 1.0 / 30.0, aspect(80, 24));
    }
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    let mut renderer = Renderer::new(Depth::TrueColor);
    terminal
        .draw(|frame| renderer.draw(frame, &world, &camera))
        .unwrap();

    let buf = terminal.backend().buffer();
    let mut tinted = false;
    for y in 0..24 {
        let mut sky: Option<Color> = None;
        let mut lit = Vec::new();
        for x in 0..80 {
            let cell = &buf[(x, y)];
            let u = cell.symbol().chars().next().map(|c| c as u32).unwrap_or(0);
            if (0x2801..=0x28FF).contains(&u) {
                lit.push(cell.bg);
            } else if sky.is_none() {
                sky = Some(cell.bg);
            }
        }
        if let Some(sky) = sky
            && lit.iter().any(|bg| *bg != sky)
        {
            tinted = true;
        }
    }
    assert!(tinted, "no dense cell tinted its background toward the ink");
}
