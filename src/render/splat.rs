//! Per-bird darkness splatted into an optical-depth buffer.
//!
//! Follows Costanzo, Hildenbrandt & Hemelrijk (2022): a bird contributes
//! according to how much of its wing faces the observer (`e = |up · d̂|`) and
//! how large it appears, and the flock's optical depth is the sum over birds.

use crate::sim::{World, flight};

use super::camera::Camera;

/// Starling wingspan [m], used as the apparent disc size.
const WINGSPAN: f32 = 0.4;

/// Overall darkness scale. Tuned so a settled flock reads as dense smoke.
const K_DARK: f32 = 25.0;

/// Distance over which a bird's contribution fades toward the sky [m].
const ATMOSPHERE: f32 = 700.0;

/// Adds each bird's opacity to `depth` (length `pixel_w · pixel_h`).
pub fn splat(depth: &mut [f32], pixel_w: usize, pixel_h: usize, world: &World, camera: &Camera) {
    let focal = camera.focal_px(pixel_h);
    for i in 0..world.len() {
        let Some((sx, sy, dist)) = camera.project(world.position[i], pixel_w, pixel_h) else {
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
        let exposure = up.dot(dhat).abs();
        let atmosphere = (1.0 - dist / ATMOSPHERE).clamp(0.25, 1.0);
        let a = K_DARK * (0.15 + 0.85 * exposure) * atmosphere;

        let rho = WINGSPAN * focal / dist;
        if rho < 1.0 {
            bilinear(depth, pixel_w, pixel_h, sx, sy, a * rho * rho);
        } else {
            gaussian(depth, pixel_w, pixel_h, sx, sy, a, (rho * 0.5).min(8.0));
        }
    }
}

/// Spreads `value` over the four pixels nearest to `(sx, sy)`.
fn bilinear(depth: &mut [f32], pixel_w: usize, pixel_h: usize, sx: f32, sy: f32, value: f32) {
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
            depth[y as usize * pixel_w + x as usize] += value * wx * wy;
        }
    }
}

/// Adds a small Gaussian disc of standard deviation `sigma` centred on `(sx, sy)`.
fn gaussian(
    depth: &mut [f32],
    pixel_w: usize,
    pixel_h: usize,
    sx: f32,
    sy: f32,
    peak: f32,
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
            depth[y as usize * pixel_w + x as usize] += peak * (-d2 * inv).exp();
        }
    }
}
