//! Drawing a frame from the world and camera.
//!
//! The pipeline is: paint the sky and reeds, splat every bird's raw opacity into
//! an optical-depth buffer, adapt the exposure to the flock's density, then
//! dither the exposed buffer into braille dots over the sky.

pub mod camera;
pub mod colour;
pub mod dither;
pub mod sky;
pub mod splat;

use ratatui::Frame;

pub use camera::Camera;

use crate::sim::World;
use colour::Depth;

/// Exposure before the flock has been seen: the old fixed `k_dark`.
const DEFAULT_EXPOSURE: f32 = 25.0;

/// Optical depth that maps to 70 % opacity: `-ln(1 - 0.7)`.
const TARGET_TAU: f32 = 1.204;

/// Time constant of the exposure adaptation [s]. Slow enough that a sudden
/// blackening (a falcon attack) is still visible.
const EXPOSURE_TAU: f32 = 4.0;

/// Bounds on the adaptive exposure.
const EXPOSURE_MIN: f32 = 1.0;
const EXPOSURE_MAX: f32 = 400.0;

/// Ease foreground transitions without changing the plants' geometry [s].
const REED_FADE_TAU: f32 = 0.6;

/// Persistent per-frame buffers for the renderer.
pub struct Renderer {
    depth: Vec<f32>,
    dist: Vec<f32>,
    /// Optical depths of the flock's pixels, reused for the percentile.
    samples: Vec<f32>,
    exposure: f32,
    exposure_ready: bool,
    reed_opacity: Option<f32>,
    colour: Depth,
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new(Depth::TrueColor)
    }
}

impl Renderer {
    /// A renderer with no buffers allocated yet, for the given colour depth.
    pub fn new(colour: Depth) -> Self {
        Self {
            depth: Vec::new(),
            dist: Vec::new(),
            samples: Vec::new(),
            exposure: DEFAULT_EXPOSURE,
            exposure_ready: false,
            reed_opacity: None,
            colour,
        }
    }

    /// The current adaptive exposure (exposed for tests).
    pub fn exposure(&self) -> f32 {
        self.exposure
    }

    /// Draws `world` as seen by `camera` into `frame`, adapting the exposure
    /// over `dt` seconds.
    pub fn draw(&mut self, frame: &mut Frame, world: &World, camera: &Camera, dt: f32) {
        let area = frame.area();
        if area.is_empty() {
            return;
        }
        let pixel_w = area.width as usize * 2;
        let pixel_h = area.height as usize * 4;
        let evening = world.evening();
        let colour = self.colour;

        // Background: sky gradient, stars and reeds.
        sky::draw(frame, evening, colour, camera);

        // Raw optical depth of the birds, plus opacity · distance for
        // atmosphere.
        self.depth.clear();
        self.depth.resize(pixel_w * pixel_h, 0.0);
        self.dist.clear();
        self.dist.resize(pixel_w * pixel_h, 0.0);
        splat::splat(
            &mut self.depth,
            &mut self.dist,
            pixel_w,
            pixel_h,
            world,
            camera,
        );

        self.adapt_exposure(dt);

        composite(
            frame,
            &self.depth,
            &self.dist,
            pixel_w,
            evening,
            colour,
            self.exposure,
            camera,
        );
        let target = sky::reed_visibility(camera);
        let opacity = self.reed_opacity.get_or_insert(target);
        *opacity += (target - *opacity) * (1.0 - (-dt.max(0.0) / REED_FADE_TAU).exp());
        sky::draw_reeds_with_opacity(frame, world.time, colour, camera, *opacity);
    }

    /// Eases the exposure so the 80th percentile of the flock's optical depth
    /// maps to about 70 % opacity, so a small group and a huge flock both read
    /// well (review C3).
    fn adapt_exposure(&mut self, dt: f32) {
        self.samples.clear();
        self.samples
            .extend(self.depth.iter().copied().filter(|&d| d > 0.0));
        if self.samples.is_empty() {
            return;
        }
        let idx = ((self.samples.len() as f32 * 0.8) as usize).min(self.samples.len() - 1);
        let tau80 = {
            let (_, mid, _) = self.samples.select_nth_unstable_by(idx, |a, b| {
                a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
            });
            *mid
        };
        if tau80 > 1e-6 {
            let target = (TARGET_TAU / tau80).clamp(EXPOSURE_MIN, EXPOSURE_MAX);
            if !self.exposure_ready {
                self.exposure = target;
                self.exposure_ready = true;
            } else {
                self.exposure += (target - self.exposure) * (1.0 - (-dt / EXPOSURE_TAU).exp());
            }
            self.exposure = self.exposure.clamp(EXPOSURE_MIN, EXPOSURE_MAX);
        }
    }
}

/// Turns the optical-depth buffer into braille dots and colours the cells.
#[allow(clippy::too_many_arguments)]
fn composite(
    frame: &mut Frame,
    depth: &[f32],
    dist: &[f32],
    pixel_w: usize,
    evening: f32,
    colour: Depth,
    exposure: f32,
    camera: &Camera,
) {
    let area = frame.area();
    let buf = frame.buffer_mut();
    let tex = dither::texture();
    let mask = dither::SIZE - 1;
    for row in 0..area.height {
        let sky_rgb = sky::row_colour(row, area.height, evening, camera);
        for col in 0..area.width {
            let mut bits = 0u32;
            let mut sum = 0.0f32;
            let mut raw_tau_sum = 0.0f32;
            let mut dist_sum = 0.0f32;
            for dy in 0..4usize {
                for dx in 0..2usize {
                    let idx = (row as usize * 4 + dy) * pixel_w + col as usize * 2 + dx;
                    let raw = depth[idx];
                    let tau = exposure * raw;
                    let alpha = 1.0 - (-tau).exp();
                    sum += alpha;
                    raw_tau_sum += raw;
                    dist_sum += dist[idx];
                    let px = col as usize * 2 + dx;
                    let py = row as usize * 4 + dy;
                    if alpha > tex[py & mask][px & mask] {
                        bits |= dither::dot_bit(dx, dy);
                    }
                }
            }
            if bits == 0 {
                continue;
            }
            let mean = sum / 8.0;
            // Tint the background toward the ink by the cell's mean opacity (up
            // to 18 %), so dense parts read as smoke; the braille dots stay for
            // detail and edges.
            // A little tone helps dense flocks, but strong cell-wide shading
            // makes a rectangular mosaic instead of fine bird silhouettes.
            let tint = mean * mean * 0.18;
            let bg_rgb = colour::blend(sky_rgb, colour::INK, tint);
            // The ink is fixed; only atmospheric distance blends it back toward
            // the sky, so silhouettes stay dark near the observer.
            let mean_dist = if raw_tau_sum > 1e-6 {
                dist_sum / raw_tau_sum
            } else {
                0.0
            };
            // Silhouettes remain ink-dark at ordinary observing distances.
            // The old linear fade made a flock a few hundred metres away grey
            // on grey, despite auto-exposure restoring all its dither dots.
            let atmosphere = (-mean_dist / (splat::ATMOSPHERE * 3.0))
                .exp()
                .clamp(0.65, 1.0);
            let fg_rgb = colour::blend(sky_rgb, colour::INK, atmosphere);
            let fg = colour::convert(fg_rgb, colour);
            let mut bytes = [0u8; 4];
            let symbol = dither::braille(bits).encode_utf8(&mut bytes);
            let cell = &mut buf[(area.x + col, area.y + row)];
            cell.set_symbol(symbol);
            cell.set_fg(fg);
            cell.set_bg(colour::convert(bg_rgb, colour));
        }
    }
}
