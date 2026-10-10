//! Deterministic, accelerated visual review using the real terminal renderer.
//! cargo run --release --example preview -- DIR SEED BIRDS SECONDS [falcon|arc] [COLS ROWS]
use std::{fmt::Write, fs};

use anyhow::Result;
use murmuration::{
    render::{Camera, Renderer, colour::Depth},
    sim::{World, analysis, params::DT},
};
use ratatui::{Terminal, backend::TestBackend, style::Color};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let dir = args
        .get(1)
        .map(String::as_str)
        .unwrap_or("/tmp/opencode/murmuration-preview");
    let seed = args.get(2).map(|s| s.parse()).transpose()?.unwrap_or(7);
    let birds = args.get(3).map(|s| s.parse()).transpose()?.unwrap_or(2000);
    let seconds: f32 = args.get(4).map(|s| s.parse()).transpose()?.unwrap_or(60.0);
    let mode = args.get(5).map(String::as_str).unwrap_or("display");
    fs::create_dir_all(dir)?;
    let mut world = World::new(birds, seed);
    if mode == "arc" {
        world.enable_arc(seconds / 60.0);
    }
    if mode == "falcon" {
        world.spawn_falcon();
    }
    let w = args
        .get(6)
        .map(|s| s.parse())
        .transpose()?
        .unwrap_or(120u16);
    let h = args.get(7).map(|s| s.parse()).transpose()?.unwrap_or(36u16);
    let mut terminal = Terminal::new(TestBackend::new(w, h))?;
    let mut camera = Camera::new();
    let mut renderer = Renderer::new(Depth::TrueColor);
    camera.update(&world, 1.0, w as f32 / (h as f32 * 2.0));
    let frames = (seconds * 30.0) as usize;
    let mut accumulator = 0.0;
    for frame in 0..=frames {
        if frame > 0 {
            accumulator += 1.0 / 30.0;
            while accumulator >= DT {
                world.step(DT);
                accumulator -= DT;
            }
            camera.update(&world, 1.0 / 30.0, w as f32 / (h as f32 * 2.0));
        }
        terminal.draw(|f| renderer.draw(f, &world, &camera, 1.0 / 30.0))?;
        if frame % 300 != 0 && frame != frames {
            continue;
        }
        let buf = terminal.backend().buffer();
        let mut svg = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\">",
            w as usize * 8,
            h as usize * 16,
            w as usize * 8,
            h as usize * 16
        );
        for y in 0..h {
            for x in 0..w {
                let cell = &buf[(x, y)];
                let rgb = |c| match c {
                    Color::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
                    _ => String::from("#111111"),
                };
                write!(
                    svg,
                    "<rect x=\"{}\" y=\"{}\" width=\"8\" height=\"16\" fill=\"{}\"/>",
                    x * 8,
                    y * 16,
                    rgb(cell.bg)
                )?;
                let symbol = cell.symbol().chars().next().unwrap_or(' ');
                let code = symbol as u32;
                if (0x2800..=0x28ff).contains(&code) {
                    for dy in 0..4 {
                        for dx in 0..2 {
                            if (code - 0x2800) & murmuration::render::dither::dot_bit(dx, dy) != 0 {
                                write!(
                                    svg,
                                    "<circle cx=\"{}\" cy=\"{}\" r=\"1.25\" fill=\"{}\"/>",
                                    x as usize * 8 + dx * 4 + 2,
                                    y as usize * 16 + dy * 4 + 2,
                                    rgb(cell.fg)
                                )?;
                            }
                        }
                    }
                } else if symbol == '▀' {
                    write!(
                        svg,
                        "<rect x=\"{}\" y=\"{}\" width=\"8\" height=\"8\" fill=\"{}\"/>",
                        x * 8,
                        y * 16,
                        rgb(cell.fg)
                    )?;
                } else if symbol != ' ' {
                    write!(
                        svg,
                        "<text x=\"{}\" y=\"{}\" font-family=\"DejaVu Sans Mono\" font-size=\"16\" fill=\"{}\">{symbol}</text>",
                        x * 8,
                        y * 16 + 13,
                        rgb(cell.fg)
                    )?;
                }
            }
        }
        svg.push_str("</svg>");
        fs::write(format!("{dir}/frame-{:04}.svg", frame / 30), svg)?;
        println!(
            "seed={seed} mode={mode} t={:.1} fov={:.1} {}",
            world.time,
            camera.fov.to_degrees(),
            analysis::compute(&world).line()
        );
    }
    Ok(())
}
