//! The command-line interface definition.
//!
//! It lives in the library rather than the binary so the `xtask` generator can
//! render the man page and the shell completions from the same clap definition
//! the binary parses. This is an implementation detail, hidden from the docs.

use clap::Parser;

/// Starlings at dusk, in your terminal.
#[derive(Debug, Parser)]
#[command(name = "murmuration", version, about)]
pub struct Cli {
    /// Run headless for this many simulated seconds, printing flock statistics
    /// once per second, then exit.
    #[arg(long, value_name = "SECONDS")]
    pub analyze: Option<f32>,
    /// Run the forced-turn experiment headless, report the fitted wave speed
    /// and R², then exit.
    #[arg(long)]
    pub turns: bool,
    /// Run the agitation-wave experiment (falcon attack) headless and exit.
    #[arg(long)]
    pub agitation: bool,
    /// Run the part-D probe (2000 birds, seed 0, four minutes) and print the
    /// metrics that catch the "looks wrong" failures, then exit.
    #[arg(long)]
    pub probe: bool,
    /// Copying threshold k for the zigzag wave (1–7).
    #[arg(long, default_value_t = 2)]
    pub zigzag_k: u32,
    /// Turning model: base (banked) or inertial (review C7).
    #[arg(long, value_enum, default_value_t = crate::sim::flight::Turning::Base)]
    pub turning: crate::sim::flight::Turning,
    /// A falcon hunts from the start.
    #[arg(long)]
    pub falcon: bool,
    /// Colour handling.
    #[arg(long, value_enum, default_value_t = crate::render::colour::Mode::Auto)]
    pub colors: crate::render::colour::Mode,
    /// Exit on any key press or mouse movement.
    #[arg(long)]
    pub screensaver: bool,
    /// Do not capture the mouse (no mouse falcon, no exit-on-move).
    #[arg(long)]
    pub no_mouse: bool,
    /// Hold the display phase (no evening arc).
    #[arg(long)]
    pub no_arc: bool,
    /// Length of one evening, in minutes.
    #[arg(long, default_value_t = 15.0)]
    pub arc_minutes: f32,
    /// Number of birds [default: from the terminal size, 4000–8000].
    #[arg(long)]
    pub birds: Option<usize>,
    /// Random seed for the evening.
    #[arg(long)]
    pub seed: Option<u64>,
    /// Forced-turn experiment: birds disturbed.
    #[arg(long, default_value_t = 100)]
    pub turn_side: usize,
    /// Forced-turn experiment: imposed heading change, in radians.
    #[arg(long, default_value_t = 0.8)]
    pub turn_angle: f32,
    /// Forced-turn experiment: seconds to simulate after the disturbance.
    #[arg(long, default_value_t = 2.0)]
    pub turn_window: f32,
    /// Forced-turn experiment: minimum local heading change to include a bird,
    /// in radians.
    #[arg(long, default_value_t = 0.3)]
    pub turn_min: f32,
}
