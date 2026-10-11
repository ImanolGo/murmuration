# Changelog

All notable changes to this project. Format loosely follows
[Keep a Changelog](https://keepachangelog.com/); this project uses
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.2.0] - 2026-10-11

### Added

- Prebuilt binaries for Linux, macOS and Windows, with shell and PowerShell
  installers, an `.msi` for Windows and `.tar.xz` archives for each platform.
- An `amd64` `.deb` (built with `cargo-deb`) that installs the binary plus a man
  page and bash/zsh/fish shell completions, all generated from the command-line
  definition.
- Automated releases: pushing a version tag builds the binaries, creates the
  GitHub Release, attaches the `.deb` and publishes to crates.io with Trusted
  Publishing (no stored token).

### Changed

- The minimum supported Rust version is now **1.95** (latest stable minus two);
  CI builds against it, and raising it is not a breaking change.
- Added package metadata (authors, keywords, categories), README badges and a
  security policy.
- The internal modules are hidden from the rendered documentation; docs.rs now
  shows the crate-level overview and stability note.

## [0.1.1] - 2026-10-10

### Fixed

- Fade foreground reeds smoothly with camera elevation and over time instead of
  popping the whole band on/off at a hard horizon cutoff; pause freezes the fade.

- Remove doubled lateral acceleration and add explicit, documented cohesion,
  and committed-return adaptations to prevent boundary milling and
  reduce filament formation.
- Frame both flock dimensions, compensate camera tracking lag and avoid slow
  zoom-out clipping during approaching passes.
- Keep silhouettes dark; reduce checkerboard shading and prevent footprint
  darkness jumps and exposure changes while paused.
- Use an elevation-aware, less saturated sky with finer, softly dithered colour
  gradients. Replace the jumping block skyline with fine reeds that bend in the
  wind and disappear from overhead close-ups.
- Spawn compact arriving groups along a shared approach corridor rather than
  unrelated opposing streams.

### Added

- Reproducible SVG visual-review example and regression checks for the motion,
  camera, footprint, foreground and roost-crossing defects.
- A browser-free live ANSI recorder and refreshed README demo, avoiding stalled
  browser captures and preserving simulation timing.

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
