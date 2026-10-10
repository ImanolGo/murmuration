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

/// Persistent per-frame buffers for the renderer.
pub struct Renderer {
    depth: Vec<f32>,
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer {
    /// A renderer with no buffers allocated yet.
    pub fn new() -> Self {
        Self { depth: Vec::new() }
    }

    /// Draws `world` as seen by `camera` into `frame`.
    pub fn draw(&mut self, frame: &mut Frame, world: &World, camera: &Camera) {
        let area = frame.area();
        if area.is_empty() {
            return;
        }
        let pixel_w = area.width as usize * 2;
        let pixel_h = area.height as usize * 4;

        // Background: sky gradient and reeds.
        sky::draw(frame);

        // Optical depth of the birds.
        self.depth.clear();
        self.depth.resize(pixel_w * pixel_h, 0.0);
        splat::splat(&mut self.depth, pixel_w, pixel_h, world, camera);

        composite(frame, &self.depth, pixel_w);
    }
}

/// Turns the optical-depth buffer into braille dots and colours the cells.
fn composite(frame: &mut Frame, depth: &[f32], pixel_w: usize) {
    let area = frame.area();
    let buf = frame.buffer_mut();
    for row in 0..area.height {
        let t = row as f32 / area.height.saturating_sub(1).max(1) as f32;
        let sky_col = sky::dusk_colour(t);
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
            let fg = colour::scale(sky_col, 1.0 - (mean * 3.0).clamp(0.0, 1.0));
            let mut bytes = [0u8; 4];
            let symbol = dither::braille(bits).encode_utf8(&mut bytes);
            let cell = &mut buf[(col, row)];
            cell.set_symbol(symbol);
            cell.set_fg(fg);
        }
    }
}
