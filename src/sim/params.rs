//! Every model constant, with its source in a doc comment.
//!
//! Values and their names follow Table 1 of Hildenbrandt, Carere & Hemelrijk
//! (2010), "Self-organized aerial displays of thousands of starlings: a model"
//! (arXiv:0908.2677), unless noted.

/// Integration time step Δt [s]. Table 1.
pub const DT: f32 = 0.005;

/// Reaction time Δu [s]: steering is held for this long between decisions.
/// Table 1.
pub const DU: f32 = 0.05;

/// Cruise speed v0 [m/s]. Table 1.
pub const V0: f32 = 10.0;

/// Body mass m [kg]. Table 1.
pub const MASS: f32 = 0.08;

/// Lift-to-drag ratio C_L/C_D. Table 1.
pub const CL_CD: f32 = 3.3;

/// Standard gravity [m/s²].
pub const G: f32 = 9.81;

/// Default lift `L0 = m·g` [N]; cruise flight balances the weight. Table 1
/// prints 0.78 N, which is `m·g` rounded.
pub const L0: f32 = MASS * G;

/// Default drag, equal to default thrust, `D0 = T0 = L0/(C_L/C_D)` [N].
/// Table 1.
pub const D0: f32 = L0 / CL_CD;

/// Speed relaxation time τ [s]. Table 1.
pub const TAU: f32 = 1.0;

/// Banking weights w_βin, w_βout. Table 1.
pub const W_BETA_IN: f32 = 10.0;
/// See [`W_BETA_IN`].
pub const W_BETA_OUT: f32 = 1.0;

/// Bank angle limit [rad]. Not in the paper; a numerical safeguard so that a
/// bird cannot roll past vertical and lose all lift.
pub const MAX_BANK: f32 = 80.0 * std::f32::consts::PI / 180.0;

/// Speed is kept within `[V_MIN, V_MAX]`. PLAN.md.
pub const V_MIN: f32 = 0.5 * V0;
/// See [`V_MIN`].
pub const V_MAX: f32 = 2.0 * V0;

/// Topological range n_c: the number of interaction partners a bird strives
/// for. Table 1.
pub const N_C: f32 = 6.5;

/// Max perception radius R_max [m]. Table 1.
pub const R_MAX: f32 = 100.0;

/// Interpolation factor s for the adaptive radius, per reaction. Table 1
/// ("0.1 Δu"). See [`radius_step`] for why the update differs from Eq. 2.
pub const RADIUS_LERP: f32 = 0.1;

/// Radius of maximal separation, the "hard sphere" r_h [m]. Table 1.
pub const R_H: f32 = 0.2;

/// Separation radius r_sep [m]. Table 1.
pub const R_SEP: f32 = 4.0;

/// Standard deviation σ of the separation Gaussian [m]. Table 1.
pub const SIGMA: f32 = 1.37;

/// Weighting factor for the separation force [N]. Table 1 lists 1 N; raised to
/// 1.7 N to bring the mean nearest-neighbour distance into the 0.7–1.5 m band
/// the paper matches by tuning r_sep (our separation Gaussian is sharper).
pub const W_SEP: f32 = 1.7;

/// Weighting factor for the alignment force [N]. Table 1 lists 0.5 N; raised to
/// 1.0 N because Eq. 8 is applied to the *mean* neighbour heading (see
/// `social::decide`), which is a weaker signal than the printed sum.
pub const W_ALIGN: f32 = 1.0;

/// Weighting factor for the cohesion force [N]. Table 1.
pub const W_COH: f32 = 1.0;

/// Cosine of the rear blind angle for cohesion and alignment. The blind sector
/// is `2 × 45°`, so a neighbour is blind when `f·d̂ < −cos 45°`. Table 1.
pub const BLIND_COS: f32 = -std::f32::consts::FRAC_1_SQRT_2;

/// Weighting factor for the random force [N]. Table 1.
pub const W_XI: f32 = 0.01;

/// Roost boundary radius R_roost [m]. Table 1.
pub const R_ROOST: f32 = 150.0;

/// Weighting factor for the horizontal roost force [N/m]. Table 1 lists
/// 0.01 N/m; that is ~100× weaker than the social lateral forces, so the
/// banking stays saturated by neighbour noise and the flock never turns at the
/// roost boundary — it elongates and the adaptive radius diverges. Raised to
/// 1.0 N/m so the flock is actually held over the roost.
pub const W_ROOST_H: f32 = 1.0;

/// Weighting factor for the vertical roost force [N/m]. Table 1.
pub const W_ROOST_V: f32 = 0.2;

/// Preferred flight height above the roost, the "zero level" z0 [m]. The paper
/// calls it arbitrary; the evening arc varies it. Table 1 (implied).
pub const Z0: f32 = 100.0;

/// Reeds are at the bottom of the world; the arc lands birds below this.
pub const REED_TOP: f32 = 3.0;

/// Uniform grid cell size for neighbour search [m]. PLAN.md.
pub const GRID_CELL: f32 = 4.0;

// --- Escape and predation (PLAN.md "Escape"; Hemelrijk et al. 2015) ---

/// A bird is alarmed when the falcon is within this distance [m]. PLAN.md.
pub const R_THREAT: f32 = 30.0;

/// Weighting of the flee force away from the falcon [N]. PLAN.md.
pub const W_FLEE: f32 = 2.0;

/// Flash expansion triggers inside this distance [m]. PLAN.md.
pub const R_FLASH: f32 = 8.0;

/// ...and when the falcon is moving faster than this [m/s]. PLAN.md.
pub const V_FLASH: f32 = 20.0;

/// Strength of the flash-expansion impulse [N]. PLAN.md.
pub const W_FLASH: f32 = 12.0;

/// How long the flash expansion lasts [s]. PLAN.md.
pub const FLASH_DURATION: f32 = 0.3;

/// Peak zigzag bank angle [rad]. Hemelrijk et al. 2015 (target roll ±50°).
pub const ZIGZAG_MAX_ROLL: f32 = 50.0 * std::f32::consts::PI / 180.0;

/// Duration of one zigzag manoeuvre [s]. Hemelrijk et al. 2015.
pub const ZIGZAG_DURATION: f32 = 0.6;

/// A bird cannot start another zigzag for this long [s]. PLAN.md.
pub const ZIGZAG_COOLDOWN: f32 = 1.5;

/// Default number of neighbours whose zigzag starts a copy. PLAN.md (1–7).
pub const ZIGZAG_K: u32 = 2;

// --- Falcon (PLAN.md "Falcon") ---

/// Cruising speed of the falcon outside the flock [m/s]. PLAN.md.
pub const FALCON_CRUISE: f32 = 15.0;
/// Climb speed [m/s]. PLAN.md.
pub const FALCON_CLIMB: f32 = 15.0;
/// Stooping speed through the flock [m/s]. PLAN.md (25–35).
pub const FALCON_STOOP: f32 = 30.0;
/// Height above the flock the falcon climbs before stooping [m]. PLAN.md.
pub const FALCON_ATTACK_HEIGHT: f32 = 40.0;
/// How often the autonomous falcon attacks [s, min..max]. PLAN.md (5–20).
pub const FALCON_ATTACK_MIN: f32 = 5.0;
/// See [`FALCON_ATTACK_MIN`].
pub const FALCON_ATTACK_MAX: f32 = 20.0;

/// Advances an adaptive interaction radius toward `n_c` partners.
///
/// The paper's text describes the rule as "increased if `|N_i|` is smaller than
/// `n_c`, decreased if larger, unchanged if equal". Eq. 2 as printed,
/// `R_i ← (1−s)·R_i + s·(R_max − R_max·|N_i|/n_c)`,
/// does not do that: at `|N_i| = n_c` it shrinks `R_i` toward zero, and it is
/// dynamically unstable for the Table 1 values — I verified numerically that
/// it makes `|N_i|` swing between 0 and tens rather than settling at 6.5.
///
/// This is a stable form of the stated intent. Relaxing geometrically toward
/// the radius that would give `n_c` partners, assuming `|N| ∝ R³`, moves `R` by
/// a factor `(n_c/|N|)^(s/3)` per reaction: `|N| = n_c` is a fixed point, and
/// the rate stays bounded even for an almost isolated bird.
pub fn radius_step(r: f32, n: u32) -> f32 {
    let n = n.max(1) as f32;
    let factor = (N_C / n).powf(RADIUS_LERP / 3.0);
    (r * factor).clamp(R_H, R_MAX)
}
