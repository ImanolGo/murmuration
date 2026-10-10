# murmuration implementation plan

This document is for a coding agent implementing murmuration from scratch. Read the README first: it explains what the piece is for and the research behind it. Then read the StarDisplay paper (Hildenbrandt, Carere & Hemelrijk 2010, https://arxiv.org/abs/0908.2677) in full before starting M1. **Where this plan and the paper disagree about the model, the paper wins**; note the difference in your commit message.

Work through the milestones in order. Each ends with acceptance criteria; don't start the next until they pass, and commit at the end of each.

## Ground rules

- Rust, stable, edition 2024. `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass before every commit.
- Check crate APIs on docs.rs before using them.
- No `unwrap()` / `expect()` outside tests and `main` setup.
- **The simulation knows nothing about the terminal.** `sim/` is pure, deterministic for a given seed, and runs headless. Only `render/` and `main.rs` touch the terminal.
- **Scientific honesty:** behaviour comes from the bird rules. Never script a shape, a wave or an escape pattern directly. If something doesn't emerge, fix the rules or document that it doesn't.
- No network access, ever. No config or data files are required to run.

## Dependencies

```toml
[dependencies]
ratatui = "0.30"
crossterm = "0.29"
clap = { version = "4", features = ["derive"] }
anyhow = "1"
rand = "0.9"
rand_chacha = "0.9"
glam = "0.34"
rayon = "1"

[dev-dependencies]
insta = "1"
```

## Architecture

```
src/
  main.rs            CLI, terminal setup/teardown, main loop
  lib.rs
  sim/
    mod.rs           World: birds (struct of arrays), falcon, roost, time; step(dt)
    params.rs        every model constant, with its source in a doc comment
    neighbours.rs    uniform spatial grid, adaptive-radius topological neighbours
    social.rs        separation, cohesion (with centrality), alignment, roost, noise
    flight.rs        fixed-wing flight model: lift, drag, thrust, banking
    escape.rs        threat detection, flee, zigzag manoeuvre and its copying
    falcon.rs        autonomous hunter and mouse-driven hunter
    arc.rs           the evening: arrival, display, roosting, night
    analysis.rs      flock statistics for --analyze and tests
  render/
    mod.rs           draws a frame from World + Camera
    camera.rs        ground observer, perspective projection
    splat.rs         per-bird darkness into an optical-depth buffer
    dither.rs        buffer to braille dots (ordered dithering)
    sky.rs           sky gradient, sun, stars, reed bed silhouette
    colour.rs        truecolor, 256-colour and mono output
```

## Units and scales

SI throughout: metres, seconds, newtons, kilograms. Gravity +z is up, `g = 9.81`. The roost is centred at the origin; the reed bed top is at z = 3 m. A display flock of a few thousand birds spans roughly 50–300 m. The camera adapts its distance to keep the flock filling about 60% of the screen (see Rendering), so the world never needs rescaling for the terminal.

## The bird model (StarDisplay)

### State per bird (struct of arrays)

Position `r`, velocity `v` (forward unit `f = v/|v|`), roll angle `β`, adaptive radius `R`, the steering force `F` held since its last decision, its next decision time, and escape state (see Escape).

The body frame comes from `f` and `β`: `side = normalize(f × ẑ)` (fall back sensibly when `f` is near vertical), `up₀ = side × f`, and `up` is `up₀` rotated about `f` by `β`.

### Parameters (from the paper's Table 1; put each in `params.rs` with a citation)

| Name | Value |
| --- | --- |
| Integration step `Δt` | 5 ms (10 ms acceptable if profiling demands it) |
| Reaction time `Δu` | 50 ms |
| Cruise speed `v0` | 10 m/s |
| Mass `m` | 0.08 kg |
| Lift/drag `C_L/C_D` | 3.3 |
| Default lift `L0` | 0.78 N (= m·g, so cruise is level flight) |
| Default drag = thrust `D0 = T0` | 0.24 N (= L0 / 3.3) |
| Speed relaxation `τ` | 1 s |
| Banking weights `w_βin`, `w_βout` | 10, 1 |
| Topological range `n_c` | 6.5 |
| Max radius `R_max` | 100 m |
| Hard sphere `r_h` | 0.2 m |
| Separation range `r_sep`, Gaussian `σ` | 4 m, 1.37 m |
| Weights separation / alignment / cohesion | 1 / 0.5 / 1 N |
| Rear blind angle (cohesion, alignment only) | 2 × 45° |
| Centrality threshold `C_c` | 0.35 |
| Noise weight `w_ξ` | 0.01 N |
| Roost radius `R_roost` | 150 m |
| Roost weights horizontal / vertical | 0.01 N/m, 0.2 N |

### Decisions, every `Δu`, staggered

Each bird re-decides its steering once per `Δu`, at a random phase offset assigned at spawn, so the flock doesn't update in lockstep. Between decisions it keeps its last `F`. Decisions for all due birds can run in parallel with rayon, reading a snapshot of positions.

1. **Neighbours.** Query the spatial grid within `R_i`, sort by distance, keep the nearest 7. Update the radius as in the paper (verify the exact form there):
   `R_i ← (1 − s)·R_i + s·(R_max − R_max·|N_i| / n_c)`, clamped to (r_h, R_max].
2. **Separation** (all neighbours, no blind angle): the average of `−d̂_ij · g(d_ij)`, where `g = 1` inside `r_h` and a halved Gaussian `exp(−(d − r_h)² / 2σ²)` out to `r_sep`, times `w_s`.
3. **Centrality** `C_i`: length of the mean unit vector to neighbours within `2R_i`. It's near 0 inside the flock and around 0.5–0.75 at the border.
4. **Cohesion** (neighbours outside the rear blind angle): `w_c · C_i ·` unit vector toward their centre. Check the paper for how `C_c` is used.
5. **Alignment** (same neighbours as cohesion): `w_a ·` the unit vector of (mean forward of neighbours − own forward).
6. **Roost**, horizontal: beyond `R_roost`, a pull back toward the centre, stronger when heading outward. **Roost**, vertical: a pull toward the preferred height `z0` (set by the evening arc), proportional to the height difference and capped at `w_RV`. Take the exact forms from the paper.
7. **Noise:** `w_ξ ·` a random unit vector.
8. **Escape force** if alarmed (see Escape).

`F_i` is the sum of all of these.

### Flight, every `Δt`

- Split `F` into its forward part `F·f` and its lateral part `F·side`.
- **Roll:** `dβ/dt = w_βin·(F·side) − w_βout·β`, with `β` clamped to ±80°. Scale the rate so typical display turns use 20–60° of roll. Check the paper's banking section for the exact form and units.
- **Forces:**
  - lift `L0·(|v|/v0)²` along `up`, divided by `cos β` (clamped) so a banked bird holds its height, which is the paper's compensation
  - drag `D0·(|v|/v0)²` against `f`
  - thrust `T0 + (m/τ)·(v0 − |v|)` along `f`, plus the forward steering component
  - gravity `−m·g·ẑ`
- **Integrate:** `v += a·Δt`, `r += v·Δt`, and keep `|v|` within 0.5·v0 and 2·v0.

Turning therefore comes from banked lift, not from rotating `f` directly. This is the main visual difference from boids, so don't shortcut it.

### Spatial grid

A uniform hash grid with 4 m cells, rebuilt every `Δt` or every decision tick, whichever is cheaper overall. A query covers `ceil(R/4)` cells per axis, which is fine because `R` is usually a few metres. If sparse birds with large `R` become the bottleneck, try a k-d tree (`kiddo`) instead.

## Escape

### Falcon

- **Autonomous** (`--falcon` or `f`): it cruises at about 15 m/s outside the flock, picks a target (an edge bird, or the flock centre), climbs above the flock, then stoops at 25–35 m/s and passes through. It climbs away and repeats every 5–20 s at random. It's a simple agent and doesn't need to be realistic beyond looking like a bird of prey.
- **Mouse:** cast a ray from the camera through the cursor and intersect it with the plane through the flock centroid facing the camera. The falcon chases that point with a max speed and some lag, so it moves like a bird rather than a cursor.

### Bird response

- **Alarm:** the falcon is within `r_threat = 30 m`. Separation-style all-round awareness is fine; no blind angle for predators.
- **Flee:** an extra force away from the falcon, decreasing with distance, for as long as the bird is alarmed.
- **Flash expansion:** if the falcon is closer than 8 m and moving faster than 20 m/s, a strong impulse directly away from it for 0.3 s.
- **Zigzag** (Hemelrijk, van Zuidam & Hildenbrandt 2015): an alarmed bird starts a manoeuvre, a forced roll to one side then the other (target roll ±50°) over about 0.6 s, overriding banking from steering.
- **Copying:** a non-alarmed bird starts its own zigzag when at least `k` of its current topological neighbours started one within the last `Δu`. `k` is configurable from 1 to 7 (default 2). After a zigzag a bird can't start another for 1.5 s, so waves pass through rather than echo forever.

Blackening, waves, splits, flash expansion, cordons and vacuoles should all emerge from this. Don't add code for any of them directly.

## The evening (arc)

By default, one evening lasts `--arc-minutes` (default 15), then a new one starts:

1. **Arrival** (first ~15%). The sky is bright gold. Groups of 50–300 birds spawn outside the view at random bearings and heights, flying toward the roost.
2. **Display** (to ~70%). Groups merge into one or a few flocks over the reeds. The falcon appears now if enabled.
3. **Roosting** (to ~90%). The sky darkens. `z0` lowers steadily, so the flock pours down in streams. A bird that drops below the reed tops (z < 3 m) near the roost is removed (it has landed).
4. **Night** (the rest). Stars come out over the reeds, then the next evening begins.

`--no-arc` holds the display phase indefinitely at dusk, for screensaver use.

## Rendering

### Camera

- A ground observer at eye height 1.7 m, at a horizontal distance from the roost chosen so the flock's angular size fills about 60% of the screen. Ease this distance slowly so the view never jumps.
- The camera looks toward the flock centroid, smoothed with a time constant of about 2 s.
- Vertical field of view 50°, perspective projection.
- `c` moves the camera to a new bearing around the roost.

### Pixel grid

Braille gives 2 × 4 dots per cell. Use a buffer of `(cols·2) × (rows·4)` pixels. With the usual 1:2 cell shape these dots are close to square; make the aspect configurable for unusual fonts.

### Darkness per bird (Costanzo, Hildenbrandt & Hemelrijk 2022)

- `d̂` is the unit vector from the bird to the camera. Wing exposure is `e = |up · d̂|`: wings seen face-on when looking along their normal, edge-on when level with the eye.
- Apparent size in pixels: `ρ = 0.4 m · f_px / depth`, where 0.4 m is the wingspan.
- Contribution `a = k_dark · (0.15 + 0.85·e)`, scaled by the footprint area. If `ρ < 1` px, splat `a·ρ²` bilinearly into the four nearest pixels; otherwise splat a small Gaussian of radius `ρ`.
- Add `a` to an optical-depth buffer. Opacity is `α = 1 − exp(−τ)`.
- **Atmospheric perspective:** fade a bird's contribution slightly toward the sky by depth, so distant parts of the flock look paler.
- **Wingbeat (optional):** a per-bird flap phase modulating `e` by about ±15%, stronger while accelerating. It adds a living shimmer.

### Dots and colour

- **Ordered dithering:** each braille dot is set when `α` exceeds its threshold from a 2 × 4 Bayer matrix. Thin edges then break into individual birds, which is right.
- **Cell foreground:** blend from a sky-tinted grey to near-black by the cell's mean `α`.
- **Cell background:** the sky gradient for that row. Use a deep blue overhead, rose in the middle and orange at the horizon at dusk, all shifting with the arc.
- **Bottom rows:** a reed-bed silhouette in block characters that sways gently.
- **Colour depth:** truecolor by default. Fall back to the nearest of the 256 xterm colours when `COLORTERM` isn't `truecolor`/`24bit`. `NO_COLOR` or `--colors none` means plain braille.

## Analysis and validation

`murmuration --analyze <seconds> --seed N --birds N` runs headless and prints flock statistics once per simulated second, plus a summary. The same functions back the tests.

| Statistic | Target | Source | Test? |
| --- | --- | --- | --- |
| Mean interaction neighbours | 6.5 ± 1 | StarDisplay | hard test |
| Flock stays one piece (single-link clusters at 5 m) with no falcon, 120 s, 2000 birds | ≥ 95% of birds in the largest cluster | — | hard test |
| Mean nearest-neighbour distance | 0.7–1.5 m | Ballerini et al. 2008b | hard test |
| Inertia ellipsoid ratios I2/I1, I3/I1 | roughly 1.9–3.6 and 4–8.4 | Ballerini et al. 2008b | report; soft test with wide bounds |
| Flock plane vs ground, \|I1·ẑ\| | > 0.85 | Ballerini et al. 2008b | hard test |
| Neighbours ahead/behind vs beside | fewer along the direction of motion than an isotropic baseline | Ballerini et al. 2008b | hard test |
| Border vs interior density | real flocks are denser at the border; StarDisplay isn't | Ballerini 2008b, Hildenbrandt 2010 | report only |
| Turning wave | a turn forced on 20 birds at one side spreads roughly linearly with time | Attanasi et al. 2014 | report the fitted speed |
| Agitation wave speed | 3–25 m/s for k in 1–7 | Hemelrijk et al. 2015 | hard test for the default k |

To measure wave speed, record for each bird the first time its roll (for zigzags) or heading (for turns) crosses a threshold, plot that against its distance from the origin of the disturbance, and fit a line.

## Milestones

### M0: Skeleton
`cargo new`, dependencies, CI on Linux, macOS and Windows. The binary opens the alternate screen, draws the dusk sky gradient with the reed silhouette, and exits on `q`.
**Done when:** CI is green and the terminal is restored cleanly, including on panic.

### M1: Headless flock
`sim/` with flight, social forces, roost, noise, the grid and staggered decisions. `--analyze` and the analysis functions.
**Done when:** the hard tests for neighbours, fragmentation, nearest-neighbour distance, flatness and neighbour anisotropy pass for 2000 birds at a fixed seed. Also, 4000 birds simulate faster than real time on one core in release mode.

### M2: First light
Camera, splatting, dithering, sky colours. The live view runs the display phase only.
**Done when:** the flock reads as a murmuration rather than a cloud of dots, judged by eye against a few real murmuration videos and noted in the commit. There's an insta snapshot of a fixed-seed frame, and 30 fps holds with 4000 birds in an 80×24 and a 200×60 terminal.

### M3: Turns
Implement the turning-wave measurement and tune banking until turns sweep across the flock and the flock turns as a body.
**Done when:** the fitted turning wave is close to linear (report R²) and the flock keeps its shape through turns.

### M4: Falcon
Autonomous and mouse falcon, alarm, flee, flash expansion, zigzag with copying.
**Done when:** the agitation-wave test passes, and in a recorded session at least blackening, waves and flash expansion are visible, listed in the commit with the seed and time. Splits, cordons and vacuoles are welcome but not required.

### M5: The evening
Arc phases, arriving groups, roosting into the reeds, night with stars, `--no-arc`, the remaining keys, `s` to save a frame as ANSI text.
**Done when:** a 15-minute evening plays from arrival to night and loops without growing memory.

### M6: Polish and release
- Colour fallbacks and `NO_COLOR`.
- Signal handling (SIGHUP, SIGTERM, SIGINT) runs the normal shutdown.
- `--screensaver`.
- Run `--analyze` for 10 minutes and add its summary to DEVELOPMENT.md.
- Record a GIF with vhs (fixed seed, `--no-arc`, a falcon pass) for the README.
- Update the README to match what was built, add CHANGELOG.md, tag `v0.1.0`.

## Experiments for after 0.1 (behind flags, don't build yet)

- `--turning inertial`: give heading change momentum, after the inertial spin model of Cavagna et al. 2015 / Attanasi et al. 2014, and compare turning-wave linearity with M3.
- `--cohesion projection`: replace cohesion with steering toward the edges of each bird's visual field (Pearce et al. 2014), and check whether the flock settles at about 30–40% opacity on its own.
- A fixed real location's sunset time, so the evening matches the user's actual dusk.
