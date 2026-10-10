//! Drawing a frame from the world and camera.
//!
//! The pipeline is: paint the sky and reeds, splat every bird's opacity into an
//! optical-depth buffer, then dither that buffer into braille dots over the sky.

pub mod camera;
pub mod colour;
pub mod dither;
pub mod sky;
pub mod splat;

use ratatui::Frame;

pub use camera::Camera;

use crate::sim::World;
use colour::Depth;

/// Persistent per-frame buffers for the renderer.
pub struct Renderer {
    depth: Vec<f32>,
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
            colour,
        }
    }

    /// Draws `world` as seen by `camera` into `frame`.
    pub fn draw(&mut self, frame: &mut Frame, world: &World, camera: &Camera) {
        let area = frame.area();
        if area.is_empty() {
            return;
        }
        let pixel_w = area.width as usize * 2;
        let pixel_h = area.height as usize * 4;
        let evening = world.evening();
        let colour = self.colour;

        // Background: sky gradient, stars and reeds.
        sky::draw(frame, evening, world.time, colour);

        // Optical depth of the birds.
        self.depth.clear();
        self.depth.resize(pixel_w * pixel_h, 0.0);
        splat::splat(&mut self.depth, pixel_w, pixel_h, world, camera);

        composite(frame, &self.depth, pixel_w, evening, colour);
    }
}

/// Turns the optical-depth buffer into braille dots and colours the cells.
fn composite(frame: &mut Frame, depth: &[f32], pixel_w: usize, evening: f32, colour: Depth) {
    let area = frame.area();
    let buf = frame.buffer_mut();
    for row in 0..area.height {
        let t = row as f32 / area.height.saturating_sub(1).max(1) as f32;
        let sky_rgb = sky::sky_colour(t, evening);
        for col in 0..area.width {
            let mut bits = 0u32;
            let mut sum = 0.0f32;
            for dy in 0..4usize {
                for dx in 0..2usize {
                    let tau = depth[(row as usize * 4 + dy) * pixel_w + col as usize * 2 + dx];
                    let alpha = 1.0 - (-tau).exp();
                    sum += alpha;
                    if alpha > dither::threshold(dx, dy) {
                        bits |= dither::dot_bit(dx, dy);
                    }
                }
            }
            if bits == 0 {
                continue;
            }
            let mean = sum / 8.0;
            // Tint the background toward the ink by the cell's mean opacity (up
            // to 60 %), so dense parts read as smoke; the braille dots stay for
            // detail and edges.
            let tint = mean.min(0.6);
            let bg_rgb = colour::blend(sky_rgb, colour::INK, tint);
            let fg_rgb = colour::scale(sky_rgb, 1.0 - (mean * 3.0).clamp(0.0, 1.0));
            let fg = colour::convert(fg_rgb, colour);
            let mut bytes = [0u8; 4];
            let symbol = dither::braille(bits).encode_utf8(&mut bytes);
            let cell = &mut buf[(col, row)];
            cell.set_symbol(symbol);
            cell.set_fg(fg);
            cell.set_bg(colour::convert(bg_rgb, colour));
        }
    }
}
