use std::time::{Duration, Instant};

use anyhow::Result;
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, MouseEventKind};
use glam::Vec3;
use murmuration::render::colour::{self, Depth};
use murmuration::render::{Camera, Renderer};
use murmuration::sim::params::{DT, V0, Z0, ZIGZAG_K};
use murmuration::sim::{World, analysis, escape};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
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
    /// Run the part-D probe (2000 birds, seed 0, four minutes) and print the
    /// metrics that catch the "looks wrong" failures, then exit.
    #[arg(long)]
    probe: bool,
    /// Copying threshold k for the zigzag wave (1–7).
    #[arg(long, default_value_t = 2)]
    zigzag_k: u32,
    /// A falcon hunts from the start.
    #[arg(long)]
    falcon: bool,
    /// Colour handling.
    #[arg(long, value_enum, default_value_t = murmuration::render::colour::Mode::Auto)]
    colors: murmuration::render::colour::Mode,
    /// Exit on any key press or mouse movement.
    #[arg(long)]
    screensaver: bool,
    /// Do not capture the mouse (no mouse falcon, no exit-on-move).
    #[arg(long)]
    no_mouse: bool,
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
    // A fresh evening by default; pass --seed to repeat one.
    let seed = cli.seed.unwrap_or_else(rand::random);

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
    if cli.probe {
        probe(cli.birds, seed);
        return Ok(());
    }

    let mut terminal = ratatui::init();
    // Mouse capture is not part of ratatui's restore, so enable it here and
    // make sure it is turned off on every exit path.
    let mouse = !cli.no_mouse;
    if mouse {
        if let Err(err) =
            crossterm::execute!(std::io::stdout(), crossterm::event::EnableMouseCapture)
        {
            ratatui::restore();
            return Err(err.into());
        }
        install_mouse_panic_hook();
    }
    let result = run(&mut terminal, seed, &cli);
    if mouse {
        let _ = crossterm::execute!(std::io::stdout(), crossterm::event::DisableMouseCapture);
    }
    ratatui::restore();
    result
}

/// Turns mouse capture off if the process panics, then runs ratatui's hook
/// (which restores the terminal).
fn install_mouse_panic_hook() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = crossterm::execute!(std::io::stdout(), crossterm::event::DisableMouseCapture);
        hook(info);
    }));
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

/// The part-D probe: the measurements that catch the "looks wrong" failures.
///
/// Runs a 4-minute flock (2000 birds, seed 0), a rendered 120×36 frame, the
/// 300 m neighbour group, the agitation wave and the 256-colour dark grey.
fn probe(birds: usize, seed: u64) {
    let seconds = 240.0f32;
    let per_second = (1.0 / DT).round() as usize;
    let steps = (seconds / DT) as usize;

    let mut world = World::new(birds, seed);
    let mut tang = 0.0f64;
    let mut roll = 0.0f64;
    let mut clamp = 0.0f32;
    let mut height = 0.0f64;
    let mut samples = 0u32;
    let mut crossings = 0u32;
    let mut last_cross = 0.0f32;
    let mut max_gap = 0.0f32;
    let mut stats = analysis::compute(&world);

    for s in 1..=steps {
        world.step(DT);
        if s % per_second == 0 {
            stats = analysis::compute(&world);
            tang += stats.tangential as f64;
            roll += stats.mean_roll as f64;
            clamp = clamp.max(stats.clamp_fraction);
            height += stats.height_std as f64;
            samples += 1;
            if stats.centroid_radius < 60.0 {
                crossings += 1;
                max_gap = max_gap.max(world.time - last_cross);
                last_cross = world.time;
            }
        }
    }
    max_gap = max_gap.max(seconds - last_cross);
    let n = samples.max(1) as f64;

    println!("# murmuration --probe birds={birds} seed={seed} seconds={seconds}");
    println!(
        "rotation:    mean signed tangential {:6.2} m/s  (target |.| < 0.5)",
        tang / n
    );
    println!(
        "crossings:   {crossings} within 60 m; longest gap {max_gap:5.0} s  (target <= 120 s)"
    );
    println!(
        "bank:        clamp {:5.1}%  (target < 2%), mean |beta| {:4.1} deg  (target < 15)",
        clamp * 100.0,
        (roll / n).to_degrees()
    );
    println!(
        "thickness:   height std {:5.2} m  (target >= 2)",
        height / n
    );
    println!(
        "shape:       I2/I1 {:5.1}, I3/I1 {:5.1}",
        stats.i2_i1, stats.i3_i1
    );
    println!(
        "neighbours:  300 m group after 1 s {:5.1}  (target 3-10)",
        probe_neighbours_300m(seed)
    );
    println!(
        "coverage:    120x36 rendered {:5.1}%  (target >= 15%)",
        probe_coverage(&world, 120, 36) * 100.0
    );
    let wave = escape::measure_agitation(&mut World::new(birds, seed), 4.0);
    println!(
        "agitation:   k={ZIGZAG_K} speed {:5.1} m/s  (target 3-25)",
        wave.speed
    );
    println!(
        "256-colour:  [20,20,20] -> {:5}  (target 233/234)",
        match colour::convert([20, 20, 20], Depth::Ansi256) {
            Color::Indexed(i) => i.to_string(),
            other => format!("{other:?}"),
        }
    );
}

/// Mean neighbours of a group spawned 300 m out, one second later (B1).
fn probe_neighbours_300m(seed: u64) -> f32 {
    let mut world = World::new(0, seed);
    let mut rng = ChaCha8Rng::seed_from_u64(seed ^ 0x300);
    let base = Vec3::new(300.0, 0.0, Z0);
    for _ in 0..50 {
        let pos = base
            + Vec3::new(
                rng.random_range(-15.0..15.0),
                rng.random_range(-15.0..15.0),
                rng.random_range(-5.0..5.0),
            );
        let toward = Vec3::new(-1.0, 0.0, 0.0)
            + Vec3::new(
                rng.random_range(-0.1..0.1),
                rng.random_range(-0.1..0.1),
                rng.random_range(-0.1..0.1),
            );
        world.push_bird(pos, toward.normalize() * V0, &mut rng);
    }
    for _ in 0..(1.0 / DT) as usize {
        world.step(DT);
    }
    analysis::compute(&world).mean_neighbours
}

/// Fraction of a `w`×`h` terminal lit by the settled flock (part D).
fn probe_coverage(world: &World, w: u16, h: u16) -> f32 {
    let aspect = (w as f32 * 2.0) / (h as f32 * 4.0);
    let mut camera = Camera::new();
    for _ in 0..300 {
        camera.update(world, 1.0 / 30.0, aspect);
    }
    let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
    let mut renderer = Renderer::new(Depth::TrueColor);
    terminal
        .draw(|frame| renderer.draw(frame, world, &camera, 1.0 / 30.0))
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
    lit as f32 / (w as f32 * h as f32)
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
    let mut renderer = Renderer::new(cli.colors.resolve());
    let size = terminal.size()?;
    let mut aspect = (size.width as f32 * 2.0) / (size.height as f32 * 4.0);
    camera.update(&world, 1.0, aspect);

    // Terminate cleanly on SIGINT / SIGTERM / SIGHUP.
    let terminate = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    #[cfg(unix)]
    for signal in [
        signal_hook::consts::SIGINT,
        signal_hook::consts::SIGTERM,
        signal_hook::consts::SIGHUP,
    ] {
        signal_hook::flag::register(signal, std::sync::Arc::clone(&terminate))?;
    }

    let mut paused = false;
    let mut save_requested = false;
    let mut accumulator = 0.0f32;
    let mut last = Instant::now();

    loop {
        if terminate.load(std::sync::atomic::Ordering::Relaxed) {
            return Ok(());
        }
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
                let size = terminal.size()?;
                aspect = (size.width as f32 * 2.0) / (size.height as f32 * 4.0);
                camera.update(&world, dt, aspect);
            }
        }

        let completed = terminal.draw(|frame| renderer.draw(frame, &world, &camera, dt))?;
        if save_requested {
            save_requested = false;
            let path = format!("murmuration-frame-seed{seed}-{:.1}s.ansi", world.time);
            std::fs::write(&path, frame_to_ansi(completed.buffer))?;
            println!("saved {path}");
        }

        if event::poll(Duration::from_millis(16))? {
            let event = event::read()?;
            if cli.screensaver && matches!(event, Event::Key(_) | Event::Mouse(_)) {
                return Ok(());
            }
            match event {
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
                Event::Mouse(mouse) if matches!(mouse.kind, MouseEventKind::Moved) => {
                    // Moving the mouse makes you the falcon, spawning one on the
                    // first movement.
                    world.spawn_falcon();
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
