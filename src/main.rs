use std::time::{Duration, Instant};

use anyhow::Result;
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, MouseEventKind};
use murmuration::render::{Camera, Renderer};
use murmuration::sim::params::DT;
use murmuration::sim::{World, analysis};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use ratatui::buffer::Buffer;
use ratatui::style::Color;

/// Starlings at dusk, in your terminal.
#[derive(Debug, Parser)]
#[command(name = "murmuration", version, about)]
struct Cli {
    /// Run headless for this many simulated seconds, printing flock statistics
    /// once per second, then exit.
    #[arg(long, value_name = "SECONDS")]
    analyze: Option<f32>,
    /// Run the forced-turn experiment headless, report the fitted wave speed
    /// and R², then exit.
    #[arg(long)]
    turns: bool,
    /// Run the agitation-wave experiment (falcon attack) headless and exit.
    #[arg(long)]
    agitation: bool,
    /// Copying threshold k for the zigzag wave (1–7).
    #[arg(long, default_value_t = 2)]
    zigzag_k: u32,
    /// A falcon hunts from the start.
    #[arg(long)]
    falcon: bool,
    /// Hold the display phase (no evening arc).
    #[arg(long)]
    no_arc: bool,
    /// Length of one evening, in minutes.
    #[arg(long, default_value_t = 15.0)]
    arc_minutes: f32,
    /// Number of birds.
    #[arg(long, default_value_t = 2000)]
    birds: usize,
    /// Random seed for the evening.
    #[arg(long)]
    seed: Option<u64>,
    /// Forced-turn experiment: birds disturbed.
    #[arg(long, default_value_t = 100)]
    turn_side: usize,
    /// Forced-turn experiment: imposed heading change [rad].
    #[arg(long, default_value_t = 0.8)]
    turn_angle: f32,
    /// Forced-turn experiment: seconds to simulate after the disturbance.
    #[arg(long, default_value_t = 2.0)]
    turn_window: f32,
    /// Forced-turn experiment: minimum local heading change to include a bird [rad].
    #[arg(long, default_value_t = 0.3)]
    turn_min: f32,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let seed = cli.seed.unwrap_or(0);

    if let Some(seconds) = cli.analyze {
        analyze(seconds, cli.birds, seed, cli.falcon);
        return Ok(());
    }
    if cli.turns {
        turns(cli.birds, seed, &cli);
        return Ok(());
    }
    if cli.agitation {
        agitation(cli.birds, seed, cli.zigzag_k);
        return Ok(());
    }

    let mut terminal = ratatui::init();
    let result = run(&mut terminal, seed, &cli);
    ratatui::restore();
    result
}

fn analyze(seconds: f32, birds: usize, seed: u64, with_falcon: bool) {
    let mut world = World::new(birds, seed);
    if with_falcon {
        world.spawn_falcon();
    }
    let steps = (seconds / DT).max(0.0) as usize;
    let per_second = (1.0 / DT).round() as usize;
    println!("# murmuration --analyze birds={birds} seed={seed} seconds={seconds}");
    let mut last = analysis::compute(&world);
    for s in 1..=steps {
        world.step(DT);
        if s % per_second == 0 {
            last = analysis::compute(&world);
            println!("t={:6.1}s {}", world.time, last.line());
        }
    }
    println!("# summary  {}", last.line());
}

fn turns(birds: usize, seed: u64, cli: &Cli) {
    let mut world = World::new(birds, seed);
    for _ in 0..(20.0 / DT) as usize {
        world.step(DT);
    }
    let config = murmuration::sim::turns::TurnConfig {
        side: cli.turn_side,
        angle: cli.turn_angle,
        window: cli.turn_window,
        min_turn: cli.turn_min,
    };
    let r = murmuration::sim::turns::measure(&mut world, config);
    println!("# murmuration --turns birds={birds} seed={seed} config={config:?}");
    println!(
        "wave speed = {:.2} m/s, R2 = {:.3}, samples = {}, turned = {:.0}%, \
         pol = {:.3}, shape = {:.2}, one piece = {}, t-span = {:.2} s, d-span = {:.0} m",
        r.speed,
        r.r2,
        r.samples,
        r.turned_fraction * 100.0,
        r.polarization_after,
        r.shape_ratio,
        r.stayed_connected,
        r.time_span,
        r.distance_span,
    );
}

fn agitation(birds: usize, seed: u64, k: u32) {
    let mut world = World::new(birds, seed);
    world.zigzag_k = k;
    let r = murmuration::sim::escape::measure_agitation(&mut world, 4.0);
    println!("# murmuration --agitation birds={birds} seed={seed} k={k}");
    println!(
        "wave speed = {:.2} m/s, R2 = {:.3}, samples = {}, reached = {:.0}%, \
         expansion = {:.0}%",
        r.speed,
        r.r2,
        r.samples,
        r.peak_alarmed * 100.0,
        r.peak_expansion * 100.0,
    );
}

fn run(terminal: &mut ratatui::DefaultTerminal, seed: u64, cli: &Cli) -> Result<()> {
    let mut world = World::new(cli.birds, seed);
    world.zigzag_k = cli.zigzag_k;
    if !cli.no_arc {
        world.enable_arc(cli.arc_minutes);
    }
    if cli.falcon {
        world.spawn_falcon();
    }
    let mut ui_rng = ChaCha8Rng::seed_from_u64(seed ^ 0x5A_17);
    let mut camera = Camera::new();
    let mut renderer = Renderer::new();
    camera.update(&world, 1.0);

    let mut paused = false;
    let mut save_requested = false;
    let mut accumulator = 0.0f32;
    let mut last = Instant::now();

    loop {
        let now = Instant::now();
        let dt = (now - last).as_secs_f32().min(0.25);
        last = now;

        if !paused {
            accumulator += dt;
            let mut stepped = false;
            while accumulator >= DT {
                world.step(DT);
                accumulator -= DT;
                stepped = true;
            }
            if stepped {
                camera.update(&world, dt);
            }
        }

        let completed = terminal.draw(|frame| renderer.draw(frame, &world, &camera))?;
        if save_requested {
            save_requested = false;
            let path = format!("murmuration-frame-{:.1}s.ansi", world.time);
            std::fs::write(&path, frame_to_ansi(completed.buffer))?;
            println!("saved {path}");
        }

        if event::poll(Duration::from_millis(16))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Char('f') => world.spawn_falcon(),
                    KeyCode::Char(' ') => paused = !paused,
                    KeyCode::Char('s') => save_requested = true,
                    KeyCode::Char('c') => camera.bearing += 0.9,
                    KeyCode::Char('+') | KeyCode::Char('=') => {
                        world.target_birds += 250;
                        world.add_birds(250, &mut ui_rng);
                    }
                    KeyCode::Char('-') => {
                        world.target_birds = world.target_birds.saturating_sub(250);
                        world.remove_birds(250);
                    }
                    _ => {}
                },
                Event::Mouse(mouse)
                    if matches!(mouse.kind, MouseEventKind::Moved) && world.falcon.is_some() =>
                {
                    let size = terminal.size()?;
                    let plane = flock_centroid(&world);
                    if let Some(falcon) = &mut world.falcon {
                        falcon.mode = murmuration::sim::falcon::Mode::Mouse;
                        falcon.target = camera.cursor_to_plane(
                            mouse.column,
                            mouse.row,
                            size.width as usize * 2,
                            size.height as usize * 4,
                            plane,
                        );
                    }
                }
                _ => {}
            }
        }
    }
}

/// Mean position of the flock.
fn flock_centroid(world: &World) -> glam::Vec3 {
    if world.is_empty() {
        glam::Vec3::ZERO
    } else {
        world.position.iter().copied().sum::<glam::Vec3>() / world.len() as f32
    }
}

/// Serialises a buffer to text with ANSI truecolour escape codes.
fn frame_to_ansi(buf: &Buffer) -> String {
    let area = buf.area();
    let mut out = String::new();
    for y in 0..area.height {
        for x in 0..area.width {
            let cell = &buf[(x, y)];
            match (cell.fg, cell.bg) {
                (Color::Rgb(fr, fg, fb), Color::Rgb(br, bg, bb)) => {
                    out.push_str(&format!(
                        "\x1b[38;2;{fr};{fg};{fb}m\x1b[48;2;{br};{bg};{bb}m"
                    ));
                }
                _ => out.push_str("\x1b[0m"),
            }
            out.push_str(cell.symbol());
        }
        out.push_str("\x1b[0m\n");
    }
    out
}
