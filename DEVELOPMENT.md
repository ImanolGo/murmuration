# Development notes

Running record of what has been built, how it was validated, and — importantly
— where the implementation departs from [PLAN.md](PLAN.md) and from the cited
papers. When the plan and the StarDisplay paper (Hildenbrandt, Carere &
Hemelrijk 2010, arXiv:0908.2677) disagree, the paper wins; where the paper is
itself internally inconsistent, the deviation and the reason are recorded here.

## Status

| Milestone | State |
| --- | --- |
| M0 Skeleton | done |
| M1 Headless flock | done |
| M2 First light | done |
| M3 Turns | measurement done; linear wave not emerging (see below) |
| M4 Falcon | mechanics done; agitation wave propagates but is faster than the literature range (see below) |
| M5 The evening | done |
| M6 Polish & release | done (v0.1.0) |

Build and checks (must be green before every commit):

```sh
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## M1 — headless flock

`sim/` is pure, headless and deterministic for a given seed. `--analyze SECONDS
--birds N --seed N` steps the world headless and prints flock statistics once
per simulated second.

### Acceptance (2000 birds, seed 42, 120 s; averaged after a 30 s settle)

| Statistic | Target | Measured |
| --- | --- | --- |
| Mean topological neighbours | 6.5 ± 1 | 6.6 |
| Mean nearest-neighbour distance | 0.7–1.5 m | 0.79 m |
| Flock flatness `\|I1·ẑ\|` | > 0.85 | 1.00 |
| Nearest neighbours ahead/behind | < 0.10 | 0.055 |
| Largest single-link cluster at 5 m | ≥ 95 % | min 96.7 % |

Performance: 4000 birds simulate 20 s in 6.4 s on one core in release mode
(~3× real time). 2000 birds run at ~7× real time.

### Deviations from PLAN.md, following the paper

- **Alignment (Eq. 8).** The paper's equation sums the neighbours' forward
  vectors; applied literally it leaves a constant forward force even when a
  bird is already aligned, which accelerated the flock to ~14 m/s and inflated
  `|Ni|`. The paper's text says "average" and PLAN.md says "mean forward of
  neighbours", so the mean is used. The resulting misalignment vector is applied
  unnormalised: normalising it turned an almost-aligned bird's tiny, randomly
  directed misalignment into a full 1 N sideways kick — the "neighbour noise"
  that saturated the bank and drove the elongation.
- **Banking lift compensation.** Lift is divided by `cos β` (clamped), so a
  banked bird holds its height; this is the compensation PLAN.md describes. The
  paper instead adds a component `|al| sin β_in` to the lift, which is
  negligible for the Table 1 values and pumped energy into the flock. The
  `1/cos β` factor is the equivalent steady-state correction.

### Deviations where the paper is internally inconsistent

These are cases where following the printed equation does not produce the
behaviour the same paper reports, so a correction was made and is documented.

- **Adaptive interaction radius (Eq. 2).** As printed,
  `R_i ← (1−s)·R_i + s·(R_max − R_max·|Ni|/n_c)`, the equation *shrinks* `R_i`
  when `|Ni| = n_c` (the term in brackets is zero), and at the Table 1 values it
  is dynamically unstable. Verified numerically: with a density giving
  `N(5 m) = 6.5`, this update makes the mean `|Ni|` swing to 27 ± 27 instead of
  settling at 6.5. Replaced with a stable geometric relaxation toward
  `|Ni| = n_c`, `R ← R·(n_c/|Ni|)^(s/3)` (the cube root because `|N| ∝ R³`).
  See `params::radius_step`.
- **Horizontal roost force (Eq. 11).** Two problems. First, the printed equation
  multiplies `w_RoostH` (N/m) by a unit lateral vector, so it does not have the
  units of a force; the units say the force should grow with the distance beyond
  `R_roost`. Second, the lateral direction was fixed to the bird's right-hand
  side, so a bird already heading inward was pushed back out. The force is now
  scaled by `dist_h − R_roost` and the lateral direction is chosen to point
  inward (`sign = −(side·n)`). With the banking and alignment fixes in place the
  Table 1 weight (0.01 N/m) is used again. **Open:** the flock no longer mills at
  the boundary, but it still does not cross the roost — with `--probe` the
  centroid stays near 143 m and the mean signed tangential velocity is 10.3 m/s.
  The boundary force is radial for a tangentially moving bird, so it applies no
  torque and cannot remove the angular momentum the initial condition gives the
  flock. Making it cross needs a non-central (dissipative) roost force or a
  different initial condition; both were investigated and are recorded as a
  known gap in the M1 notes.
- **Banking (Eq. 19–21).** The paper's roll update,
  `β ← β + atan(w_βin·a_lat·Δt) − atan(w_βout·sin β·Δt)`, is applied once per
  5 ms integration step. With `w_βin = 10`, a 1 N lateral force adds ~0.56 rad in
  a single step while the roll-out removes ~0.005 rad, so the roll is a
  high-gain integrator pinned at the ±80° clamp; the paper's own birds fly level
  most of the time, so the printed form cannot be what was simulated. Replaced
  with a coordinated turn: the target bank is `β* = atan(a_lat/g)` and the roll
  follows it with a first-order lag (`τ_roll = 0.15 s`) and a maximum roll rate.
  See `flight::step_bird`.
- **Vertical roost force (Eq. 12).** The paper makes it proportional to the
  height difference with `w_RoostV = 0.2 N`, an uncapped spring: three metres off
  the preferred height already gives 0.6 N, most of the bird's weight, so the
  flock is squeezed into a sheet about 0.7 m thick. Read as a cap (Table 1 lists
  `w_RoostV` in N, and PLAN.md says "capped at `w_RV`") it is
  `w_RoostV · clamp((z0 − z)/h, −1, 1)` with `h = 15 m`. Measured with `--probe`,
  the height standard deviation rises from 0.7 m to 2.4 m and `I3/I1` falls from
  ~1000 to ~190.
- **Separation and alignment weights.** `W_ALIGN` is back at Table 1's 0.5 N;
  the earlier 1.0 N was compensating for normalising the alignment force, which
  is now proportional to the misalignment. `W_SEP` is still raised from Table 1's
  1.0 N to 1.7 N to put the nearest-neighbour distance in the 0.7–1.5 m band. The
  paper tunes these parameters too; the direction is the same, the value differs.
- **Bank angle clamp.** The paper does not clamp `β`; ±80° is added as a
  numerical safeguard so a bird cannot roll past vertical and lose all lift.

### Findings and fixes made along the way

- **Spatial grid.** A bug where birds leaving a fixed grid box corrupted
  neighbour queries was found and fixed by covering the roost plus margin. The
  grid is now stress-tested against brute force (`neighbours::grid_matches_bruteforce`).
- **Jacobi eigen-solver.** The 3×3 symmetric solver used for the inertia
  ellipsoid picked the wrong rotation branch for near-degenerate matrices,
  which made flatness read spuriously as 0. The rotation angle is now reduced to
  the small root; with the fix, flatness is consistently 1.00 for a flat flock.
- **Speed control.** Eq. 14's speed-relaxation term `f_τ` (Eq. 1) was initially
  omitted; adding it is needed for the flock to hold cruise speed.

### Part-D probe

`--probe` (2000 birds, seed 0, four simulated minutes, plus a rendered 120×36
frame) prints the measurements that catch a flock that has collapsed into a
ribbon: mean signed tangential velocity about the roost, crossings of the roost
centre, fraction of birds at the bank clamp, mean bank angle, height standard
deviation, the inertia ratios, the neighbour count of a group spawned 300 m out,
rendered coverage, the agitation-wave speed and the 256-colour mapping of a dark
grey. The numbers are recorded in the commit messages as the model is fixed.

### Known gaps (reported, not faked)

- **Roost crossing.** The inward roost force no longer pins the flock to the
  boundary, but the flock still orbits inside `R_roost` instead of passing back
  and forth over the centre. `--probe` (2000 birds, seed 0, four minutes): mean
  signed tangential velocity 10.2 m/s and the centroid never comes within 60 m
  of the centre. The boundary force is radial for a tangentially moving bird, so
  it applies no torque and cannot remove the angular momentum the initial
  condition gives the flock. Starting the flock with little angular momentum
  (heading inward, spread with random headings, or at the centre) was tried and
  rejected: with no continuous turning the flock stretches into a filament and
  splits, so it fails M1's "≥ 95% in one cluster" test (best case: a low-L start
  plus a central restoring component crossed the roost and kept the aspect
  ratios within the hard limits, but still fragmented, 50% in the largest
  cluster). Accepted as a gap for now; the model mills over the roost as the
  paper's own does.
- **Elongation.** The flock is frequently more elongated (inertia ratio `I2/I1`
  of several tens) than the paper's 1.9–3.6. The oblong-school instability is
  only partly destroyed by the turns in this implementation. This is a
  report-only criterion; it is expected to be revisited in M3 (turns).
- **rayon.** Not used yet: decisions run single-threaded and are fast enough at
  these flock sizes.

### Initial condition

For M1 the flock starts as a compact, slightly flattened cloud near the roost
edge, all moving together at cruise speed. It used to start at the roost centre,
which made the whole flock beeline ~150 m across the display area before the
roost force turned it into a milling flock — an artefact of the start, not the
rules. Starting at the edge (where the roost force acts at once) removes that.
The paper starts birds at random positions and orientations over the roost;
starting them unaligned this way did not self-organise into a single flock in my
implementation. The evening arc (M5) spawns arriving groups instead.

## M2 — first light

The live view now renders the flock. Pipeline (`src/render/`):

- `camera.rs` — a ground observer at 1.7 m, standing off from the roost so the
  flock spans ~60 % of a 50° vertical field; look target and distance are eased
  (time constants 2 s and 3 s). The stand-off uses the 90th percentile of
  distances from the centroid, so a few stragglers in an elongated flock do not
  push the camera so far that the bulk becomes tiny.
- `splat.rs` — each bird adds optical depth `a = k_dark·(0.15 + 0.85·|up·d̂|)·fade(depth)`
  (Costanzo et al. 2022), splatted bilinearly when its disc is sub-pixel or as a
  small Gaussian otherwise. `k_dark = 25` was chosen so a settled flock reads as
  dense smoke rather than a scatter of dots.
- `dither.rs` — a 2 × 4 Bayer matrix turns opacity into braille dots; thin
  fringes break into individual dots.
- `colour.rs` — cell foreground fades from the sky tint to near-black with the
  cell's mean opacity; the background is the sky gradient.
- `mod.rs` — `Renderer` owns the optical-depth buffer and composites a frame.

The live loop in `main.rs` advances the world in fixed 5 ms steps by wall-clock
time, eases the camera, and draws. It runs the display phase only (no arc yet).

### Validation

- A fixed-seed frame (2000 birds, seed 42, 30 s, 80 × 24) is snapshotted with
  `insta` in `tests/frame.rs`. It is `#[ignore]`d because libm differences
  between platforms flip individual dither dots, so the exact snapshot is only
  valid on the machine that made it (run `cargo test -- --ignored` locally to
  check it). CI instead runs `frame_renders_a_flock`, a portable check that the
  frame contains plenty of braille and a reed silhouette.
- `tests/perf.rs` (ignored by default) measures **69–72 fps** for 4000 birds at
  both 80 × 24 and 200 × 60 in release mode, well above the 30 fps target.
- A pseudo-terminal run confirms the live view enters the alternate screen,
  emits truecolor and braille, and restores the terminal on `q`.

### Findings

- The first render was too faint (`k_dark = 2.6`); with sub-pixel birds the mean
  opacity per dot was below the first Bayer threshold, so cells lit up a single
  dot and the flock looked like a cloud of dots. Raising `k_dark` fixed the
  density without scripting any shape — the form still comes from the birds.
- No plan-vs-paper conflicts arose in M2; the constants are presentation, not
  model, parameters.

## M3 — turns (partial)

`sim/turns.rs` implements the forced-turn experiment: after the flock settles,
the headings of the birds on one side are rotated gradually (0.5 s) by a set
angle, and for every bird we record the first time its heading changes *relative
to the flock's mean heading change* by more than a threshold — subtracting the
mean removes the whole-flock body turn and leaves only the front. The distance
from the disturbance is fitted against that arrival time (`--turns`
reports the speed, R², polarisation, shape ratio and connectivity).

The flock does turn as a coherent body: it stays in one piece, polarisation
stays ≈ 0.96, and its heading changes together rather than fragmenting.

However, **the fitted turning wave is not close to linear**, so the M3
acceptance bar is not met. Best result over a banking sweep (and the paper's
`w_βin = 10` is already the best of the sweep):

| config | speed | R² | birds turned | shape ratio |
| --- | --- | --- | --- | --- |
| side 100, angle 0.8, 2 s, threshold 0.3 | ~38 m/s | 0.47 | 13 % | 3.1 |
| side 100, angle 0.8, 2 s, threshold 0.1 | ~800 m/s | 0.00 | 90 % | 3.1 |
| side 300, angle 0.8, 3 s, threshold 0.3 | ~42 m/s | 0.25 | 53 % | 6.6 |
| banking `w_βin` 1 … 20 | — | ≤ 0.47 | — | 1.5–3.6 |

Lower thresholds are crossed by the flock's natural heading spread (~14° rms),
not the wave; higher thresholds catch only the near edge, so the front is
truncated before it crosses the flock. The turn also does not preserve the
flock's shape: `I2/I1` changes by a factor of ~3–7 over the manoeuvre.

**Why, best current explanation.** The alignment rule (Eq. 8) is symmetric: the
perturbed birds are pulled back toward their unperturbed neighbours as strongly
as those neighbours are pulled forward, so a local turn is damped instead of
sweeping. The clean, linear turning waves in the literature (Attanasi et al.
2014, Cavagna et al. 2015) come from *behavioural inertia* — a heading that
keeps changing once started — which this model does not have. PLAN.md lists the
inertial-spin model as a **separate experiment for after 0.1**
(`--turning inertial`), which is consistent with the base rules not producing
the wave on their own.

This is reported rather than faked: no extra rule was added to manufacture a
wave, and the shape is not forced. The next step, if the wave is required for
0.1, is to build the deferred inertial variant and compare, rather than twist
the base model.

## M4 — falcon (mechanics done; agitation speed above range)

`sim/` (`escape.rs`, `falcon.rs`):

- **Falcon.** Autonomous state machine (cruise → climb above the flock → stoop
  through it → retreat → repeat every 5–20 s; speeds per PLAN.md), and a
  mouse-driven mode that chases the point under the cursor (a ray through the
  cursor onto the plane through the flock centroid). `f` releases the falcon.
- **Bird response.** Alarm within `r_threat = 30 m`; a flee force away from the
  falcon that falls off with distance; a flash-expansion impulse when the falcon
  is inside 8 m and faster than 20 m/s; and the zigzag manoeuvre (forced roll
  ±50° over 0.6 s, overriding banking), started by alarmed birds and copied by
  non-alarmed birds when at least `k` neighbours started one within `Δu`, with a
  1.5 s cooldown.
- The falcon is drawn a little larger and darker than a starling.

### Validation

- **Agitation wave** (`--agitation`, `tests/m4.rs`): with the copying rule
  seeded on a patch, the wave propagates and the arrival times are a clean front
  — high R² (0.77–0.97). The threshold weakens the wave as expected: `k = 1`
  reaches the whole flock, `k = 2` a few percent, `k ≥ 3` dies out in this dense
  flock.
- **Recorded session** (`--analyze 120 --birds 2000 --seed 42 --falcon`): the
  falcon attacks at roughly t = 35 s, 73 s and 98 s. Alarmed fraction peaks at
  66–76 % (blackening), mean |roll| rises to 0.9–1.1 rad (the zigzag wave), and
  the flock spread grows by ~30 % (flash expansion). Large splits are not
  required. Seed 42, times 35 s / 73 s / 98 s.

### Gap, reported not faked

The agitation **wave speed is 50–85 m/s, above the 3–25 m/s** PLAN.md cites from
Hemelrijk et al. 2015. The trend with `k` is right (higher threshold → slower,
weaker wave), but the absolute speed is too high and `k ≥ 3` does not propagate.
The likely cause is the flock density (NND ≈ 0.8 m, interaction radius ≈ 1.5 m)
combined with staggered decisions (each bird reacts every `Δu`, but at different
phases), so the front can advance several neighbour-hops per reaction time. The
`k = 1` speed of ~85 m/s is close to one interaction radius per `Δt`; hitting the
literature range would need a sparser flock or a longer effective reaction time,
which would change the M1 density. No rule was added to slow the wave by hand.

### Key bindings added

`f` releases the falcon; moving the mouse makes you the falcon (mouse mode sets
the falcon's target to the point under the cursor).

## M5 — the evening

`sim/arc.rs` runs one evening of `--arc-minutes` (default 15), then loops:

1. **Arrival** (~15 %): groups of 50–300 birds spawn at random bearings
   250–350 m out and fly toward the roost; `z0` reset to its base.
2. **Display** (to ~70 %): the flock mills over the reeds.
3. **Roosting** (to ~90 %): `z0` lowers steadily to the reed tops, so the flock
   pours down, and any bird below the reeds near the roost is removed (landed).
4. **Night**: stars come out; the flock is cleared, then the next evening begins.

The sky (`render/sky.rs`) blends a bright arrival palette → dusk → a near-black
night palette, adds stars after 75 % of the evening, and sways the reed
silhouette with time. `--no-arc` holds the display phase at dusk.

Remaining keys: `Space` pauses, `+`/`-` add/remove birds (and change the arc's
target), `c` moves the camera to a new bearing, `s` saves the current frame as
ANSI truecolour text (`murmuration-frame-<t>s.ansi`).

### Validation

- `tests/m5.rs`: a short (6 s) evening reaches Arrival, Display, Roosting and
  Night in turn, birds arrive during Arrival, and across three loops the flock
  never exceeds the target size — memory does not grow.
- A pseudo-terminal run of a 0.1-minute evening renders the flock, the `s` key
  writes the ANSI frame, and `q` restores the terminal.

No plan-vs-paper conflicts in M5.

## M6 — polish and release (v0.1.0)

- **Colour.** `--colors auto|truecolor|ansi256|none`; auto honours `NO_COLOR`
  and `COLORTERM` (truecolor/24bit), otherwise the nearest of the 256 xterm
  colours, otherwise plain braille on the terminal default.
- **Signals.** `SIGINT`/`SIGTERM`/`SIGHUP` set a flag through `signal-hook`, and
  the loop returns normally so the terminal is restored.
- **`--screensaver`.** Any key press or mouse movement exits.
- **Save.** `s` writes the current frame as ANSI truecolour text
  (`murmuration-frame-<t>s.ansi`, gitignored).

### `--analyze` for 10 minutes, 2000 birds, seed 42

Averaged over t = 60–600 s: mean `|N|` **6.55**, mean NND **0.83 m**, mean
flatness **1.000**, nearest-neighbour ahead/behind **0.065**, mean speed
**10.5 m/s**. The largest single-link cluster stayed above 95 % for the first
two minutes but fell as low as **80 %** at times over the full ten; the flock
grows more elongated over long runs (mean `I2/I1` ~ 1.3e3, `I3/I1` ~ 5e3). These
are the same two weak points flagged in M1/M3: occasional splitting and excess
elongation. The per-second statistics are stable rather than drifting, so a long
evening does not diverge.

### GIF

`docs/murmuration.tape` records `docs/murmuration.gif` with vhs: fixed seed 7,
`--no-arc`, a falcon session, 960×560 at 12 fps. Run `vhs docs/murmuration.tape`
to regenerate (needs `vhs`).

### Release

`README.md` updated, `CHANGELOG.md` added, tagged `v0.1.0`.
