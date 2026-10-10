# Changelog

All notable changes to this project. Format loosely follows
[Keep a Changelog](https://keepachangelog.com/); this project uses
[Semantic Versioning](https://semver.org/).

## [0.1.0] - 2026-10-10

First release. A starling murmuration in the terminal, built on the StarDisplay
model (Hildenbrandt, Carere & Hemelrijk 2010).

### Added

- **Headless simulation** (`sim/`): StarDisplay social forces (separation,
  cohesion with border centrality, alignment), fixed-wing flight with banked
  turning, the roost pull, noise, an adaptive-radius topological neighbourhood
  over a uniform grid, and staggered decisions. Deterministic for a seed.
  `--analyze` prints flock statistics.
- **Render** (`render/`): ground-observer perspective camera, per-bird optical
  depth from wing exposure (Costanzo et al. 2022), ordered-dithering into
  braille, and a day → dusk → night sky with stars and a swaying reed bed.
  True colour, 256-colour and plain-braille output (`--colors`, `NO_COLOR`).
- **Falcon** (`sim/falcon.rs`, `sim/escape.rs`): autonomous and mouse-driven
  hunter, alarm, flee, flash expansion, and the zigzag manoeuvre with neighbour
  copying. `--falcon`, `f`, mouse control; `--turns` and `--agitation` measure
  the waves.
- **The evening** (`sim/arc.rs`): arrival, display, roosting into the reeds, and
  night, looping over `--arc-minutes` (default 15). `--no-arc` holds the display.
- Keys: `Space`, `+`/`-`, `c`, `s` (save an ANSI frame), `q`/`Esc`; and clean
  shutdown on `SIGINT`/`SIGTERM`/`SIGHUP`.
- `--screensaver`: exit on any key or mouse movement.

### Known limitations

- The horizontal roost weight and the separation/alignment weights differ from
  Table 1 of the paper; the published adaptive-radius equation is unstable as
  printed. See [DEVELOPMENT.md](DEVELOPMENT.md) for the numbers and reasons.
- The turning wave is not close to linear (R² ≈ 0.47) and the agitation wave
  runs faster (50–85 m/s) than the 3–25 m/s the literature reports.
- The flock is more elongated over long runs than real murmurations.
- `rayon` is a dependency but decisions currently run single-threaded.
