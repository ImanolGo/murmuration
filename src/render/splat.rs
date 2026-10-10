//! Per-bird darkness splatted into an optical-depth buffer.
//!
//! Follows Costanzo, Hildenbrandt & Hemelrijk (2022): a bird contributes
//! according to how much of its wing faces the observer (`e = |up · d̂|`) and
//! how large it appears, and the flock's optical depth is the sum over birds.
//! A parallel buffer accumulates `opacity · distance`, so the compositor can
//! fade the ink toward the sky with atmospheric distance (review C2).

use crate::sim::params::FLAP_AMPLITUDE;
use crate::sim::{World, flight};

use super::camera::Camera;

/// Starling wingspan [m], used as the apparent disc size.
const WINGSPAN: f32 = 0.4;

/// Falcon wingspan [m].
const FALCON_WINGSPAN: f32 = 1.0;

/// Distance over which a bird's contribution fades toward the sky [m].
pub const ATMOSPHERE: f32 = 700.0;

/// Adds each bird's raw opacity to `depth` and `opacity · distance` to `dist`
/// (both length `pixel_w · pixel_h`). The exposure is applied by the compositor
/// so it can adapt to the flock's density (review C3).
pub fn splat(
    depth: &mut [f32],
    dist: &mut [f32],
    pixel_w: usize,
    pixel_h: usize,
    world: &World,
    camera: &Camera,
) {
    let focal = camera.focal_px(pixel_h);
    for i in 0..world.len() {
        let Some((sx, sy, z)) = camera.project(world.position[i], pixel_w, pixel_h) else {
            continue;
        };
        let margin = 8.0;
        if sx < -margin
            || sy < -margin
            || sx > pixel_w as f32 + margin
            || sy > pixel_h as f32 + margin
        {
            continue;
        }
        let dhat = (camera.eye - world.position[i]).normalize_or_zero();
        let up = flight::body_up(flight::forward(world.velocity[i]), world.roll[i]);
        // Wingbeat shimmer: the apparent wing exposure flickers with the flap,
        // more strongly while the bird accelerates.
        let flap = 1.0 + FLAP_AMPLITUDE * world.flap_amp[i] * world.flap_phase[i].sin();
        let wing = (up.dot(dhat).abs() * flap).clamp(0.0, 1.0);
        let atmosphere = (1.0 - z / ATMOSPHERE).clamp(0.25, 1.0);
        let a = (0.15 + 0.85 * wing) * atmosphere;

        let rho = WINGSPAN * focal / z;
        footprint(depth, dist, pixel_w, pixel_h, sx, sy, a, z, rho.min(16.0));
    }

    // The falcon, drawn a little larger and darker than a starling.
    if let Some(falcon) = &world.falcon
        && let Some((sx, sy, z)) = camera.project(falcon.position, pixel_w, pixel_h)
    {
        let atmosphere = (1.0 - z / ATMOSPHERE).clamp(0.25, 1.0);
        let a = 1.6 * atmosphere;
        let rho = FALCON_WINGSPAN * focal / z;
        footprint(depth, dist, pixel_w, pixel_h, sx, sy, a, z, rho.min(24.0));
    }
}

/// Blend continuously between unresolved and resolved birds. Both kernels
/// carry the same optical mass; the old switch at one pixel suddenly added
/// roughly 57% darkness as a bird approached the camera.
#[allow(clippy::too_many_arguments)]
fn footprint(
    depth: &mut [f32],
    dist: &mut [f32],
    w: usize,
    h: usize,
    x: f32,
    y: f32,
    a: f32,
    z: f32,
    rho: f32,
) {
    let mix = (rho - 1.0).clamp(0.0, 1.0);
    let mix = mix * mix * (3.0 - 2.0 * mix);
    let mass = a * rho * rho;
    if mix < 1.0 {
        let value = mass * (1.0 - mix);
        bilinear(depth, dist, w, h, x, y, value, value * z);
    }
    if mix > 0.0 {
        let sigma = rho * 0.5;
        let peak = mass * mix / (std::f32::consts::TAU * sigma * sigma);
        gaussian(depth, dist, w, h, x, y, peak, peak * z, sigma);
    }
}

/// Spreads `value` (and its `value · distance` counterpart) over the four
/// pixels nearest to `(sx, sy)`.
#[allow(clippy::too_many_arguments)]
fn bilinear(
    depth: &mut [f32],
    dist: &mut [f32],
    pixel_w: usize,
    pixel_h: usize,
    sx: f32,
    sy: f32,
    value: f32,
    dist_value: f32,
) {
    if value <= 0.0 {
        return;
    }
    let x0 = sx.floor();
    let y0 = sy.floor();
    let fx = sx - x0;
    let fy = sy - y0;
    for (dx, wx) in [(0.0f32, 1.0 - fx), (1.0, fx)] {
        for (dy, wy) in [(0.0f32, 1.0 - fy), (1.0, fy)] {
            let x = x0 + dx;
            let y = y0 + dy;
            if x < 0.0 || y < 0.0 || x >= pixel_w as f32 || y >= pixel_h as f32 {
                continue;
            }
            let idx = y as usize * pixel_w + x as usize;
            depth[idx] += value * wx * wy;
            dist[idx] += dist_value * wx * wy;
        }
    }
}

/// Adds a small Gaussian disc of standard deviation `sigma` centred on
/// `(sx, sy)`, to both buffers.
#[allow(clippy::too_many_arguments)]
fn gaussian(
    depth: &mut [f32],
    dist: &mut [f32],
    pixel_w: usize,
    pixel_h: usize,
    sx: f32,
    sy: f32,
    peak: f32,
    peak_dist: f32,
    sigma: f32,
) {
    let radius = (2.5 * sigma).ceil() as i32;
    let inv = 1.0 / (2.0 * sigma * sigma);
    let x0 = (sx.round() as i32 - radius).max(0);
    let x1 = (sx.round() as i32 + radius).min(pixel_w as i32 - 1);
    let y0 = (sy.round() as i32 - radius).max(0);
    let y1 = (sy.round() as i32 + radius).min(pixel_h as i32 - 1);
    for y in y0..=y1 {
        for x in x0..=x1 {
            let d2 = (x as f32 - sx).powi(2) + (y as f32 - sy).powi(2);
            let g = (-d2 * inv).exp();
            let idx = y as usize * pixel_w + x as usize;
            depth[idx] += peak * g;
            dist[idx] += peak_dist * g;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn footprint_does_not_pop_at_one_pixel() {
        let mut masses = Vec::new();
        for rho in [0.99, 1.01, 1.99, 2.01] {
            let mut depth = vec![0.0; 32 * 32];
            let mut dist = depth.clone();
            footprint(&mut depth, &mut dist, 32, 32, 16.3, 16.4, 1.0, 100.0, rho);
            let mass: f32 = depth.iter().sum();
            assert!((mass / (rho * rho) - 1.0).abs() < 0.03);
            masses.push(mass);
        }
        assert!(masses[1] / masses[0] < 1.05);
        assert!(masses[3] / masses[2] < 1.05);
    }
}
