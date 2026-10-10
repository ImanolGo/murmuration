# Development notes

Running record of what has been built, how it was validated, and — importantly
— where the implementation departs from [PLAN.md](PLAN.md) and from the cited
papers. When the plan and the StarDisplay paper (Hildenbrandt, Carere &
Hemelrijk 2010, arXiv:0908.2677) disagree, the paper wins; where the paper is
itself internally inconsistent, the deviation and the reason are recorded here.

## Status

The milestone measurements below are historical. The visual/motion corrections
in **Visual review follow-up** supersede the earlier camera, cohesion, roost,
sky and foreground descriptions; they are explicit model adaptations, not a
claim to reproduce StarDisplay exactly.

| Milestone | State |
| --- | --- |
| M0 Skeleton | done |
| M1 Headless flock | done |
| M2 First light | done |
| M3 Turns | re-run after the review fixes; still not linear; `--turning inertial` built (see below) |
| M4 Falcon | done; agitation wave in the 3–25 m/s range (see below) |
| M5 The evening | done |
| M6 Polish & release | done (v0.1.0) |

Build and checks (must be green before every commit):

```sh
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## Visual review follow-up

### What was actually wrong

Recorded the live terminal with VHS and inspected rendered sequences, not just
flock statistics. The old output had a clipped, off-centre flock, strong
rectangular cell shading, conspicuous sky bands, and a blocky skyline that was
supposed to be a reed bed. Longer headless runs exposed the persistent orbit
and increasing elongation. A compressed full evening additionally showed many
unrelated incoming streams instead of a gathering flock.

### Motion and model adaptations

- **One source of lateral acceleration.** Lateral steering now requests bank;
  banked lift supplies the sideways force. Previously the integrator added the
  lateral steering directly *and* through lift, doubling steady turning and
  allowing sideways sliding before the bird rolled. Forward and vertical
  steering remain direct controls.
- **All-round, unit-direction cohesion.** Alignment still has the rear blind
  sector, but cohesion uses all the spatial neighbours, like separation. The
  old forward-only mean-direction force biased even interior birds forward and
  allowed long filaments. Cohesion uses the unit resultant scaled by centrality,
  with `W_COH = 0.65 N`; the full 1 N compressed nearest-neighbour spacing too
  much. This departs from StarDisplay's blind sector for cohesion.
- **Committed returns.** A bird beyond the roost boundary keeps an inward turn
  request until it faces inward (`f·n < -0.95`). The turn has a minimum effective
  overshoot of 25 m. Outward-heading birds in the outer 20% of the roost can
  anticipate it when two neighbours are returning. This behavioural memory
  prevents the old force balance at the circular boundary and reduces the
  slice-by-slice shearing of a wide flock. There is no global flock path, shape
  target or imposed heading animation. This is an extension of Eq. 11, not the
  original equation or evidence of the literature's turning-wave linearity.
- **Vertical preference.** The capped 0.2 N preference ramps over 40 m, rather
  than 15 m. It remains a preferred height, not a hard flight plane.
- **Arrival conditions.** Compact, similarly headed groups approach through a
  seeded corridor with bearing variation and nearby roost targets. Independent
  random bearings, 60 m-wide sparse groups and unrelated ±90 m targets created
  opposing streams that simply passed through each other. The corridor changes
  each evening. This changes initial conditions, not flock shapes.

### Drawing and camera

- Fit **both** projected dimensions. The previous wide-terminal branch ignored
  horizontal extent and hid elongation by zooming into a clipped ribbon.
- Feed forward the flock's translation when easing the gaze; the old 2 s lag
  watched empty sky behind moving birds and widened the lens to cover the error.
  Initialise framing immediately, open the lens in 0.6 s when necessary, and
  tighten it slowly (3 s). The observer remains at a fixed ground position.
- Keep dark silhouettes at normal observing distances. Reduce cell-wide tint
  to at most 18%, avoiding the old checkerboard/mosaic texture. Initialise
  exposure from the first visible flock and then adapt slowly. Pausing also
  freezes exposure.
- Blend unresolved bilinear footprints into resolved Gaussians continuously,
  with matching optical mass. The old one-pixel switch jumped by about 57% in
  darkness as a bird approached the camera.
- Sample sky colour by viewing elevation, with a quieter dusk palette. The
  horizon no longer travels up the screen with the flock. Half-blocks provide
  two gradient samples per cell, and fixed sub-one-RGB-level stochastic rounding
  softens colour banding without animated background noise.
- Replace solid, height-jumping columns with fine reed stems and seed heads;
  wind bends tips sideways. Draw them after the flock, so birds cannot overwrite
  the foreground. Reeds are omitted in high-sky close-ups, where a landscape
  pinned to the bottom would be physically inappropriate.

Follow-up: the original horizon visibility check (`bottom_ray.z > 0.15`) was a
hard switch, so tracking and zooming could make the entire reed band pop in and
out. Visibility now uses a smooth angular band (`z = 0.05–0.25`) plus a 0.6 s
temporal ease. Coloured output fades stems towards the actual background; mono
output progressively thins a fixed dot pattern. The wind geometry stays unchanged,
and pausing freezes the transition. Regression checks cover both fade directions,
the former angular cutoff, and truecolour/256-colour/mono output. Coverage metrics
exclude the entire foreground band, including partially faded stems.

### Reproducing the visual checks

`examples/preview.rs` runs the **actual renderer** at 30 simulated frames/s and
exports SVG frames every ten simulated seconds plus flock statistics. Braille
dots are plotted directly (the live VHS recordings also check the actual font).
It does not hand-draw or otherwise replace the simulation's output.

```sh
cargo run --release --example preview -- /tmp/opencode/display 0 2000 240
cargo run --release --example preview -- /tmp/opencode/falcon 7 4000 120 falcon 80 24
cargo run --release --example preview -- /tmp/opencode/evening 42 2000 120 arc
cargo run --release -- --probe --seed 0 --birds 2000
```

Regression checks cover lateral slip, both-axis framing, translational camera
tracking, continuous footprint mass, sky elevation, fine/stable foreground,
truecolour/256-colour/mono rendering, and roost crossing. Flock coverage excludes
reed dots, so the foreground cannot manufacture a passing coverage result.

### Current measurements and remaining limits

The original M1 hard criteria remain unchanged and pass (2000 birds, seed 42,
120 s, averaged after 30 s):

| Statistic | Measured |
| --- | --- |
| Interaction neighbours | 6.62 |
| Nearest-neighbour distance | 0.77 m |
| Horizontal flock-plane alignment | 0.899 |
| Height standard deviation | 2.06 m |
| Nearest-neighbour axial fraction | 0.094 |
| Largest 5 m single-link cluster | stays above the 95% hard target |

The four-minute seed-0 part-D test also passes the existing bank, thickness and
`I2/I1` checks, and now **asserts** roost crossing: the longest interval without
the centroid coming within 60 m is **29 s**, rather than an entire run spent
orbiting at the boundary.

The final `--probe` (seed 0, 2000 birds, 240 s) reports **0%** at the bank clamp,
mean bank **6.8°**, mean height standard deviation **2.14 m**, settled-frame
coverage **24.2%**, and agitation speed **7.6 m/s**. Its final-frame inertia
ratios are 2.6 / 5.9; these are a single frame, **not** the time averages below.
The release benchmark for 4000 birds measured **86.4 fps** at 80×24 and
**71.2 fps** at 200×60, even while the live recording ran alongside it.

The tightened incoming corridor was also checked in a 120 s compressed evening
(seed 42, 2000 birds): the largest cluster reaches 100% at t=10 s and remains
above 99% through t=50 s, instead of the old unrelated streams. Later turns can
still split it (about 79% at t=60–80 s), as noted below.

### Live recording validation

The pseudo-terminal smoke test exits cleanly and writes distinct saved frames at
2.7, 5.7 and 8.7 simulated seconds in an 11 s real-time run. This checks the
live loop independently of the headless example.

VHS 0.12.1 in this environment produced a stalled browser capture: 610 of 620
GIF frames were identical, despite the live simulation continuing normally.
Do not use that recording as evidence of motion quality. `docs/record.py` is a
browser-free fallback used for the updated README GIF. It drives the real binary
in a pseudo-terminal, captures its saved truecolour ANSI buffers, rasterises the
actual braille font, and preserves **simulation timestamps** in the GIF. A 3 s
smoke recording produced 20 distinct frames out of 20. The VHS recipe remains
available for environments where browser capture works.

```sh
cargo build --release
python3 docs/record.py
```

The optional recorder requires POSIX, Pillow, ffmpeg and a monospace font with
braille glyphs; none are runtime dependencies of the Rust application. It uses
fontconfig when available, or accepts `--font /path/to/font.ttf`, and rejects a
font that renders braille as missing-glyph boxes.

The refreshed README recording includes the smooth foreground fades, contains
**417 distinct frames out of 417**, and spans **44.04 simulated seconds**. Playback
and glyph checks confirm moving output rather than stalled frames or missing-font
boxes.

### Remaining limits

There are still real model limitations. Mean signed tangential velocity is
**5.89 m/s**, so the flock retains a preferred turning sense even though it no
longer traces the old fixed circle. Mean `I3/I1` is **26.4**, above the review's
hard suggestion of 20. Falcon attacks and arriving groups can produce separate
subflocks, and a short compressed evening is not equivalent to a 15-minute one:
birds have less physical time to merge and descend. The literature's linear
turning-wave result is still not claimed. These are reported limitations, not
passing assertions or effects hidden by camera clipping.

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

Performance: 4000 birds simulate 20 s in 5.7 s in release mode (~3.5× real
time, decisions in parallel with rayon). 2000 birds run faster still.

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
- **Separation and alignment weights.** Both are back at Table 1's values
  (`W_SEP = 1.0 N`, `W_ALIGN = 0.5 N`). The earlier raised values compensated for
  other errors: `W_ALIGN` for normalising the alignment force, and `W_SEP` for a
  separation Gaussian that was missing its factor of 2 (see the M1 findings).
- **Bank angle clamp.** The paper does not clamp `β`; ±80° is added as a
  numerical safeguard so a bird cannot roll past vertical and lose all lift.

### Findings and fixes made along the way

- **Spatial grid.** The grid was a dense array covering a fixed ±210 m box, so
  birds spawned outside it (the evening arc spawns groups 250–350 m out) found
  no neighbours and their interaction radius climbed to `R_max`. It is now an
  unbounded spatial hash: integer cell coordinates are hashed into a 2¹⁶-bucket
  table, the exact distance check is kept, and each bucket entry is matched to
  its own cell so a bird is never counted twice when two query cells collide.
  It is stress-tested against brute force
  (`neighbours::grid_matches_bruteforce`) and by a 300 m group that finds six
  neighbours in the first second (`b1::distant_group_finds_neighbours`).
- **Jacobi eigen-solver.** The 3×3 symmetric solver used for the inertia
  ellipsoid picked the wrong rotation branch for near-degenerate matrices,
  which made flatness read spuriously as 0. The rotation angle is now reduced to
  the small root; with the fix, flatness is consistently 1.00 for a flat flock.
- **Speed control.** Eq. 14's speed-relaxation term `f_τ` (Eq. 1) was initially
  omitted; adding it is needed for the flock to hold cruise speed.
- **Separation Gaussian (Eq. 4).** The code used `exp(−((d − r_h)/σ)²)`; the
  paper and PLAN.md have `exp(−(d − r_h)² / 2σ²)`. The missing factor of 2 made
  the curve too narrow, which `W_SEP` had been raised to 1.7 N to compensate for.
  With the correct curve and `W_SEP = 1.0 N`, the mean nearest-neighbour distance
  is 0.8–1.1 m, inside the 0.7–1.5 m target.

### Part-D probe

`--probe` (2000 birds, seed 0, four simulated minutes, plus a rendered 120×36
frame) prints the measurements that catch a flock that has collapsed into a
ribbon: mean signed tangential velocity about the roost, crossings of the roost
centre, fraction of birds at the bank clamp, mean bank angle, height standard
deviation, the inertia ratios, the neighbour count of a group spawned 300 m out,
rendered coverage, the agitation-wave speed and the 256-colour mapping of a dark
grey. The numbers are recorded in the commit messages as the model is fixed.

`tests/part_d.rs` runs the same 4-minute probe and asserts the targets the model
meets — bank clamp < 2 %, mean bank < 15°, height std ≥ 2 m, `I2/I1` < 10 — and
prints the open ones (mean tangential velocity, longest roost gap, `I3/I1`), so a
regression in the met targets fails the suite. The other part-D targets have
their own tests: `b1::distant_group_finds_neighbours`,
`frame::settled_flock_covers_the_screen`, `m4::m4_agitation_wave_propagates` and
`colour::dark_greys_stay_dark`.

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
- **Elongation.** The flock is still more elongated (`I2/I1` ≈ 6, `I3/I1` ≈ 190)
  than the paper's 1.9–3.6 / 4–8.4, and above the review's hard limits of 10 and
  20. The oblong-school instability is only partly destroyed by the turns in this
  implementation, and the fixed orbit (above) shears the flock along its path, so
  the flock projects as a long band rather than a compact blob. The camera (A5)
  frames the band's height to 60 % of the screen, which meets the coverage target
  but clips the ends.
- **rayon.** Decisions run in parallel with rayon: each reads a snapshot of the
  flock and writes only its own outputs, so the result is still deterministic
  for a seed (verified by `determinism_for_a_seed`). The default flock size is
  chosen from the terminal area, 4000–8000 birds (review C6).

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

- `camera.rs` — a ground observer at 1.7 m standing at a fixed spot 150 m from
  the roost centre (the bearing is changed by `c`). The gaze eases toward the
  flock (2 s) and the field of view eases (3 s) so the flock's projected 5–95 %
  bounding box fills 60 % of the binding (smaller) screen dimension. This
  replaced a stand-off that dollied back with the flock's 3D 90th-percentile
  radius, which kept a wide flat flock small and looked like zooming.
- `splat.rs` — each bird adds raw optical depth `a = (0.15 + 0.85·|up·d̂|)·fade(depth)`
  (Costanzo et al. 2022), splatted bilinearly when its disc is sub-pixel or as a
  small Gaussian otherwise. A second buffer accumulates `a · depth` for the
  atmospheric fade. The overall exposure is applied by the compositor. The wing
  exposure flickers with a per-bird wingbeat phase (`flap_phase`, advanced at
  10 Hz in the sim) by ±15 %, more while the bird accelerates — the living
  shimmer PLAN.md lists (review C5).
- `dither.rs` — a tiled 64 × 64 blue-noise threshold texture (generated with
  Ulichney's void-and-cluster algorithm, fixed seed) turns opacity into braille
  dots, so thin fringes break into individual dots without the screen-door
  pattern a 2 × 4 Bayer matrix gives (review C4).
- `colour.rs` — a fixed, slightly warm near-black ink (`INK`) for the birds, and
  `blend` for mixing two colours.
- `mod.rs` — `Renderer` owns the optical-depth buffer and an `opacity · distance`
  buffer and composites a frame. Each lit cell's *background* is tinted toward
  the ink by the cell's mean opacity (up to 60 %), so dense parts read as smoke;
  the dots are drawn in the ink and faded toward the sky only with atmospheric
  distance, so silhouettes stay dark near the observer (reviews C1 and C2). The
  exposure adapts slowly (time constant 4 s) so the 80th percentile of the
  flock's optical depth maps to ~70 % opacity, so a 300-bird group and a
  5000-bird flock both read well; it is slow enough that the blackening of an
  attack still shows (review C3).

The live loop in `main.rs` advances the world in fixed 5 ms steps by wall-clock
time, eases the camera, and draws. It runs the display phase only (no arc yet).

### Validation

- A fixed-seed frame (2000 birds, seed 42, 30 s, 80 × 24) is snapshotted with
  `insta` in `tests/frame.rs`. It is `#[ignore]`d because libm differences
  between platforms flip individual dither dots, so the exact snapshot is only
  valid on the machine that made it (run `cargo test -- --ignored` locally to
  check it). CI instead runs `frame_renders_a_flock`, a portable check that the
  frame contains plenty of braille and a reed silhouette.
- `tests/perf.rs` (ignored by default) measures **89–92 fps** for 4000 birds at
  both 80 × 24 and 200 × 60 in release mode, well above the 30 fps target.
- `settled_flock_covers_the_screen` (2000 birds, seed 0, four minutes, 120 × 36)
  checks the framing: the flock covers 20 % of the cells, above the 15 % target.
  With the old dolly camera it was 1 %.
- A pseudo-terminal run confirms the live view enters the alternate screen,
  emits truecolor and braille, and restores the terminal on `q`.

### Findings

- The first render was too faint (`k_dark = 2.6`); with sub-pixel birds the mean
  opacity per dot was below the first Bayer threshold, so cells lit up a single
  dot and the flock looked like a cloud of dots. Raising `k_dark` fixed the
  density without scripting any shape — the form still comes from the birds.
- No plan-vs-paper conflicts arose in M2; the constants are presentation, not
  model, parameters.

## M3 — turns (re-run after the review fixes)

`sim/turns.rs` implements the forced-turn experiment: after the flock settles,
the headings of the birds on one side are rotated gradually (0.5 s) by a set
angle, and for every bird we record the first time its heading changes *relative
to the flock's mean heading change* by more than a threshold — subtracting the
mean removes the whole-flock body turn and leaves only the front. The distance
from the disturbance is fitted against that arrival time (`--turns`
reports the speed, R², polarisation, shape ratio and connectivity).

The flock turns as a coherent body: it stays in one piece and polarisation
stays ≈ 0.99. The original measurements were taken with the saturated banking
from review A2, so they were re-run after the fixes. The shape is now preserved
much better, but **the fitted turning wave is still not close to linear**:

| config (seed 42) | speed | R² | birds turned | shape ratio |
| --- | --- | --- | --- | --- |
| side 100, angle 0.8, 2 s, threshold 0.3 | 8.9 m/s | 0.45 | 5 % | 1.60 |
| side 300, angle 0.8, 3 s, threshold 0.3 | 24.6 m/s | 0.08 | 17 % | 1.56 |
| side 500, angle 1.2, 4 s, threshold 0.4 | 29.1 m/s | 0.31 | 31 % | 1.11 |
| side 100, angle 0.8, 2 s, threshold 0.1 | 76.7 m/s | 0.05 | 41 % | 1.60 |

Lower thresholds are still crossed by the flock's natural heading spread, and
higher thresholds catch only the near edge, so the front is truncated. The shape
is much better than before (shape ratio 1.1–1.6 versus 3.1–6.6), because the
bank no longer saturates.

**The `--turning inertial` experiment.** PLAN.md defers an inertial-spin variant
(Cavagna et al. 2015), so it is now built: `--turning inertial` gives the roll
*rate* momentum (a second time constant, `TAU_SPIN = 0.3 s`) on top of the
coordinated turn. It does not improve linearity — the best R² over the same
configs is 0.32 (default config 0.32 at 38 m/s), and longer momentum makes the
wave faster and less linear. So behavioural inertia as modelled here is not
enough to produce the literature's linear turning waves, and the base model's
turning wave remains a reported gap rather than a faked one.

## M4 — falcon (mechanics done)

`sim/` (`escape.rs`, `falcon.rs`):

- **Falcon.** Autonomous state machine (cruise → climb above the flock → stoop
  through it → retreat → repeat every 5–20 s; speeds per PLAN.md), and a
  mouse-driven mode that chases the point under the cursor (a ray through the
  cursor onto the plane through the flock centroid). `f` releases the falcon.
- **Bird response.** Alarm within `r_threat = 30 m`; a flee force away from the
  falcon that falls off with distance; a flash-expansion impulse when the falcon
  is inside 8 m and faster than 20 m/s; and the zigzag manoeuvre (forced roll
  ±50° over 0.6 s, overriding banking), started by alarmed birds and copied by
  non-alarmed birds when at least `k` neighbours are in a copyable zigzag, with a
  1.5 s cooldown. Alarmed birds roll away from the falcon, and a copying bird
  takes the roll direction of the majority of the neighbours it copies, so the
  dark band is coherent instead of a mix of opposite rolls (review B3).
- The falcon is drawn a little larger and darker than a starling.

### Validation

- **Agitation wave** (`--agitation`, `tests/m4.rs`): with the copying rule
  seeded on a patch, the wave propagates and the arrival times are a clean front
  — high R² (0.99 for `k = 1`–`2`). The threshold weakens the wave as expected:
  `k = 1` 18 m/s, `k = 2` 9.9 m/s (both reach the whole flock), `k = 3` 4.7 m/s
  (8 %), `k ≥ 4` dies out. `m4_agitation_wave_propagates` now asserts the default
  `k = 2` speed is in 3–25 m/s.
- **Recorded session** (`--analyze 120 --birds 2000 --seed 42 --falcon`): the
  falcon attacks at roughly t = 35 s, 73 s and 98 s. Alarmed fraction peaks at
  66–76 % (blackening), mean |roll| rises to 0.9–1.1 rad (the zigzag wave), and
  the flock spread grows by ~30 % (flash expansion). Large splits are not
  required. Seed 42, times 35 s / 73 s / 98 s.

### Agitation speed fixed (review B2)

The wave speed is now within the 3–25 m/s range PLAN.md cites from Hemelrijk et
al. 2015. The old 50–85 m/s came from the copying window: a bird copied any
neighbour whose zigzag had started within the last `Δu`, including one that had
copied 5 ms earlier, so the front could hop one neighbour per 5 ms integration
step. A bird now counts a neighbour only once its zigzag is at least `Δu` old
(and still active), which limits the front to about one neighbour distance per
reaction time. The review's window was `[Δu, 2Δu]`; the upper bound turned out to
kill the default `k = 2` wave (the copyable set became too thin), so only the
lower bound is used — it is the bound that limits the speed. Measured
(`--agitation`, 2000 birds, seed 42): `k = 1` 18 m/s, `k = 2` 9.9 m/s, `k = 3`
4.7 m/s, `k ≥ 4` dies out.

### Key bindings added

`f` releases the falcon; moving the mouse makes you the falcon (mouse mode sets
the falcon's target to the point under the cursor). Mouse capture is enabled
around the run and turned off on every exit path — normal return, signal and
panic — and the first mouse movement spawns the falcon, so mouse control no
longer requires an existing falcon. `--no-mouse` leaves the mouse alone.

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
ANSI truecolour text (`murmuration-frame-seed<seed>-<t>s.ansi`). Without
`--seed` the seed is drawn at random and printed by `--analyze`, so a nice
evening can be reproduced from the saved file name. New birds added with `+`
take the flock's mean heading plus a little noise, instead of a random
direction (review B7).

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
- **256-colour dark greys (review B6).** `nearest_256` built the grey *level*
  from `(gray - 8)` but the grey *index* from the same un-divided value, so a
  dark grey mapped to the bright end of the ramp (value 20 became near-white;
  macOS Terminal.app does not set `COLORTERM`, so it uses this fallback). The
  index is now `232 + (gray - 8) / 10` with the matching level; `[20, 20, 20]`
  maps to 233 and dark birds stay dark.
- **Signals.** `SIGINT`/`SIGTERM`/`SIGHUP` set a flag through `signal-hook`, and
  the loop returns normally so the terminal is restored.
- **`--screensaver`.** Any key press or mouse movement exits.
- **Mouse (review B4).** `ratatui::init()` does not enable mouse capture, so the
  mouse falcon and exit-on-move silently did nothing. Mouse capture is now
  enabled after init and disabled on every exit path (return, signal, panic).
  Moving the mouse spawns a mouse-driven falcon on the first movement.
  `--no-mouse` disables capture.
- **Save.** `s` writes the current frame as ANSI truecolour text
  (`murmuration-frame-seed<seed>-<t>s.ansi`, gitignored).
- **Seed (review B5).** Without `--seed` a random seed is drawn for each run, so
  every evening differs; `--analyze` prints it and it is in the saved file name,
  so a good one can be replayed.

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
