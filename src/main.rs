use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
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

    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
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
            last = murmuration::sim::analysis::compute(&world);
            println!("t={:6.1}s {}", world.time, last.line());
        }
    }
    println!("# summary  {}", last.line());
}

fn run(terminal: &mut ratatui::DefaultTerminal) -> Result<()> {
    loop {
        terminal.draw(murmuration::render::draw)?;
        if event::poll(Duration::from_millis(16))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
            && matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
        {
            return Ok(());
        }
    }
}
