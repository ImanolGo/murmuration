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
  neighbours", so the mean is used.
- **Banking lift compensation.** The paper states that rolling adds a component
  of magnitude `|al| sin β_in` to the lift. That magnitude is negligible for the
  Table 1 values and adding it pumped energy into the flock, so it is omitted.
  This is the term PLAN.md renders as a division of lift by `cos β`; the paper
  has no such division, so no `cos β` factor is used either.

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
- **Horizontal roost force (Eq. 11).** Table 1 gives `w_RoostH = 0.01 N/m`,
  ~100× weaker than the social lateral forces. Because the bank angle reacts to
  the *total* lateral steering force (Eq. 18), the social forces saturate the
  bank with neighbour noise, the flock never turns at the roost boundary, it
  elongates into a filament and the adaptive radius diverges (`|Ni|` into the
  hundreds). Raised to 1.0 N/m, which holds a sheet-like milling flock inside
  `R_roost`.
- **Separation and alignment weights.** Raised from Table 1's 1.0 N and 0.5 N to
  1.7 N and 1.0 N, to put the nearest-neighbour distance in the 0.7–1.5 m band
  and keep the flock coherent. The paper tunes these parameters too; the
  direction is the same, the value differs.
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

### Known gaps (reported, not faked)

- **Elongation.** The flock is frequently more elongated (inertia ratio `I2/I1`
  of several tens) than the paper's 1.9–3.6. The oblong-school instability is
  only partly destroyed by the turns in this implementation. This is a
  report-only criterion; it is expected to be revisited in M3 (turns).
- **rayon.** Not used yet: decisions run single-threaded and are fast enough at
  these flock sizes.

### Initial condition

For M1 the flock starts as a compact, slightly flattened cloud near the roost,
all moving together at cruise speed. The paper starts birds at random positions
and orientations over the roost; starting them unaligned this way did not
self-organise into a single flock in my implementation. The evening arc (M5)
will spawn arriving groups instead.

## M2 — first light

The live view now renders the flock. Pipeline (`src/render/`):

- `camera.rs` — a ground observer at 1.7 m, standing off from the roost so the
  flock spans ~60 % of a 50° vertical field; look target and distance are eased
  (time constants 2 s and 3 s).
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
  `insta` in `tests/frame.rs`; it shows a dense, soft-edged, stippled mass with
  no individual bird dots, which matches the character of dusk murmuration
  footage and the StarDisplay figures.
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
