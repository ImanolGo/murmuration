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
        .draw(|frame| renderer.draw(frame, &world, &camera, 1.0 / 30.0))
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
        .draw(|frame| renderer.draw(frame, &world, &camera, 1.0 / 30.0))
        .unwrap();

    let buf = terminal.backend().buffer();
    let (mut braille, mut reeds) = (0u32, 0u32);
    for y in 0..24 {
        for x in 0..80 {
            if let Some(c) = buf[(x, y)].symbol().chars().next() {
                let u = c as u32;
                if (0x2800..=0x28FF).contains(&u)
                    && y < 24 - murmuration::render::sky::REED_MAX_ROWS
                {
                    braille += 1;
                }
                if (0x2801..=0x28ff).contains(&u) && buf[(x, y)].fg == Color::Rgb(12, 24, 16) {
                    reeds += 1;
                }
            }
        }
    }
    assert!(braille > 40, "too few braille cells for a flock: {braille}");
    // Reeds belong only to views near the horizon, not overhead close-ups.
    assert!(
        reeds == 0 || reeds > 50,
        "incomplete reed silhouette: {reeds}"
    );
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
            .draw(|frame| renderer.draw(frame, &world, &camera, 1.0 / 30.0))
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
        .draw(|frame| renderer.draw(frame, &world, &camera, 1.0 / 30.0))
        .unwrap();

    let buf = terminal.backend().buffer();
    let mut lit = 0u32;
    for y in 0..h.saturating_sub(murmuration::render::sky::REED_MAX_ROWS) {
        for x in 0..w {
            if let Some(c) = buf[(x, y)].symbol().chars().next() {
                let u = c as u32;
                if (0x2801..=0x28FF).contains(&u) && buf[(x, y)].fg != Color::Rgb(12, 24, 16) {
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

/// Dense cells tint their background toward the ink, and the dots are drawn in
/// a fixed dark ink rather than a scaled-down sky, so thin edges stay visible
/// (reviews C1 and C2).
#[test]
fn cells_use_dark_ink_and_tint() {
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
        .draw(|frame| renderer.draw(frame, &world, &camera, 1.0 / 30.0))
        .unwrap();

    let buf = terminal.backend().buffer();
    let mut tinted = false;
    let mut dots = 0u32;
    for y in 0..24 {
        let mut sky: Option<Color> = None;
        let mut lit = Vec::new();
        for x in 0..80 {
            let cell = &buf[(x, y)];
            let u = cell.symbol().chars().next().map(|c| c as u32).unwrap_or(0);
            if (0x2801..=0x28FF).contains(&u) {
                lit.push((cell.fg, cell.bg));
            } else if sky.is_none() {
                sky = Some(cell.bg);
            }
        }
        let Some(sky) = sky else { continue };
        let sky_sum = channel_sum(sky);
        for (fg, bg) in lit {
            assert!(
                channel_sum(fg) < sky_sum,
                "a dot at row {y} is not darker than the sky"
            );
            dots += 1;
            if bg != sky {
                tinted = true;
            }
        }
    }
    assert!(dots > 0, "no dots to check");
    assert!(tinted, "no dense cell tinted its background toward the ink");
}

/// Sum of an RGB colour's channels, for comparing brightness.
fn channel_sum(c: Color) -> u32 {
    match c {
        Color::Rgb(r, g, b) => u32::from(r) + u32::from(g) + u32::from(b),
        _ => 0,
    }
}

/// The exposure adapts so a small group and a large flock both read well
/// (review C3).
#[test]
fn exposure_adapts_to_flock_size() {
    let (w, h) = (120u16, 36u16);
    let mut exposures = Vec::new();
    for birds in [300usize, 5000] {
        let mut world = World::new(birds, 7);
        for _ in 0..(20.0 / DT) as usize {
            world.step(DT);
        }
        let mut camera = Camera::new();
        for _ in 0..300 {
            camera.update(&world, 1.0 / 30.0, aspect(w, h));
        }
        let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
        let mut renderer = Renderer::new(Depth::TrueColor);
        // Let the exposure settle (time constant 4 s).
        for _ in 0..180 {
            terminal
                .draw(|frame| renderer.draw(frame, &world, &camera, 1.0 / 30.0))
                .unwrap();
        }
        exposures.push(renderer.exposure());
        let buf = terminal.backend().buffer();
        let mut lit = 0u32;
        for y in 0..h.saturating_sub(murmuration::render::sky::REED_MAX_ROWS) {
            for x in 0..w {
                if let Some(c) = buf[(x, y)].symbol().chars().next() {
                    let u = c as u32;
                    if (0x2801..=0x28FF).contains(&u) && buf[(x, y)].fg != Color::Rgb(12, 24, 16) {
                        lit += 1;
                    }
                }
            }
        }
        let coverage = lit as f32 / (w as f32 * h as f32);
        assert!(
            coverage >= 0.05,
            "{birds} birds cover only {:.1}% after adaptation",
            coverage * 100.0
        );
    }
    assert!(
        exposures[0] > exposures[1],
        "the sparse flock should get a higher exposure: {exposures:?}"
    );
}

#[test]
fn foreground_is_fine_and_pause_is_visually_stable() {
    let mut world = World::new(0, 7);
    world.time = 12.0;
    let camera = Camera::new();
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    let mut renderer = Renderer::new(Depth::TrueColor);
    terminal
        .draw(|f| renderer.draw(f, &world, &camera, 0.0))
        .unwrap();
    let first = terminal.backend().buffer().clone();
    terminal
        .draw(|f| renderer.draw(f, &world, &camera, 0.0))
        .unwrap();
    assert_eq!(&first, terminal.backend().buffer());
    for y in 0..24 {
        for x in 0..80 {
            let cell = &first[(x, y)];
            assert_ne!(cell.symbol(), "█", "a reed became a building");
            if cell.fg == Color::Rgb(12, 24, 16) {
                assert!(y >= 21, "reeds intrude into the sky");
                assert!(
                    cell.symbol()
                        .chars()
                        .next()
                        .is_some_and(|c| (0x2801..=0x28ff).contains(&(c as u32)))
                );
            }
        }
    }

    let mut overhead = Camera::new();
    overhead.forward = glam::Vec3::Z;
    overhead.up = glam::Vec3::Y;
    // A new overhead view starts without foreground. A running view now fades
    // towards that state instead of snapping there (see the transition test).
    let mut renderer = Renderer::new(Depth::TrueColor);
    terminal
        .draw(|f| renderer.draw(f, &world, &overhead, 0.0))
        .unwrap();
    for cell in &terminal.backend().buffer().content {
        assert_ne!(
            cell.fg,
            Color::Rgb(12, 24, 16),
            "foreground landscape floats in an overhead close-up"
        );
    }
}

#[test]
fn foreground_fades_in_both_directions_and_freezes_when_paused() {
    let mut world = World::new(0, 7);
    world.time = 12.0;
    let horizon = Camera::new();
    let mut overhead = Camera::new();
    overhead.forward = glam::Vec3::Z;
    overhead.up = glam::Vec3::Y;
    let reeds = |buf: &Buffer| {
        buf.content
            .iter()
            .filter(|cell| {
                cell.symbol()
                    .chars()
                    .next()
                    .is_some_and(|c| (0x2801..=0x28ff).contains(&(c as u32)))
            })
            .count()
    };
    for depth in [Depth::TrueColor, Depth::Ansi256, Depth::Mono] {
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        let mut renderer = Renderer::new(depth);
        terminal
            .draw(|f| renderer.draw(f, &world, &horizon, 0.0))
            .unwrap();
        assert!(reeds(terminal.backend().buffer()) > 50);
        terminal
            .draw(|f| renderer.draw(f, &world, &overhead, 1.0 / 30.0))
            .unwrap();
        assert!(
            reeds(terminal.backend().buffer()) > 0,
            "foreground vanished in one frame"
        );
        let fading = terminal.backend().buffer().clone();
        terminal
            .draw(|f| renderer.draw(f, &world, &overhead, 0.0))
            .unwrap();
        assert_eq!(
            &fading,
            terminal.backend().buffer(),
            "pause did not freeze the fade"
        );
        for _ in 0..180 {
            terminal
                .draw(|f| renderer.draw(f, &world, &overhead, 1.0 / 30.0))
                .unwrap();
        }
        assert_eq!(
            reeds(terminal.backend().buffer()),
            0,
            "foreground persists overhead"
        );
        terminal
            .draw(|f| renderer.draw(f, &world, &horizon, 1.0 / 30.0))
            .unwrap();
        assert!(
            reeds(terminal.backend().buffer()) > 0,
            "foreground never fades back in"
        );
        if depth == Depth::TrueColor {
            assert!(
                !terminal
                    .backend()
                    .buffer()
                    .content
                    .iter()
                    .any(|cell| cell.fg == Color::Rgb(12, 24, 16)),
                "foreground jumped to full opacity"
            );
        }
    }
}
