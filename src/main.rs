use std::time::{Duration, Instant};

use anyhow::Result;
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use murmuration::render::{Camera, Renderer};
use murmuration::sim::params::DT;
use murmuration::sim::{World, analysis};

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
    /// Number of birds.
    #[arg(long, default_value_t = 2000)]
    birds: usize,
    /// Random seed for the evening.
    #[arg(long)]
    seed: Option<u64>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let seed = cli.seed.unwrap_or(0);

    if let Some(seconds) = cli.analyze {
        analyze(seconds, cli.birds, seed);
        return Ok(());
    }
    if cli.turns {
        turns(cli.birds, seed, &cli);
        return Ok(());
    }

    let mut terminal = ratatui::init();
    let result = run(&mut terminal, cli.birds, seed);
    ratatui::restore();
    result
}

fn analyze(seconds: f32, birds: usize, seed: u64) {
    let mut world = World::new(birds, seed);
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
    // Settle the flock before disturbing it.
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
    let d_max = r.points.iter().map(|p| p.0).fold(0.0f32, f32::max);
    let bins = 6usize;
    let mut sum = vec![0.0f32; bins];
    let mut count = vec![0usize; bins];
    for (d, t) in &r.points {
        let b = (((d / d_max.max(1e-6)) * bins as f32) as usize).min(bins - 1);
        sum[b] += t;
        count[b] += 1;
    }
    let mut line = String::new();
    for b in 0..bins {
        if count[b] > 0 {
            line += &format!(
                "{:.0}m:{:.2}s({}) ",
                (b as f32 + 0.5) / bins as f32 * d_max,
                sum[b] / count[b] as f32,
                count[b]
            );
        }
    }
    println!("binned distance:mean arrival  {line}");
}

fn run(terminal: &mut ratatui::DefaultTerminal, birds: usize, seed: u64) -> Result<()> {
    let mut world = World::new(birds, seed);
    let mut camera = Camera::new();
    let mut renderer = Renderer::new();
    camera.update(&world, 1.0);

    let mut accumulator = 0.0f32;
    let mut last = Instant::now();
    loop {
        let now = Instant::now();
        let dt = (now - last).as_secs_f32().min(0.25);
        last = now;

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

        terminal.draw(|frame| renderer.draw(frame, &world, &camera))?;

        if event::poll(Duration::from_millis(16))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
            && matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
        {
            return Ok(());
        }
    }
}
