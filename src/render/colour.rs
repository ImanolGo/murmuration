//! Colour handling and the fallbacks for terminals without true colour.

use ratatui::style::Color;

/// Multiplies an RGB colour's channels by `f` (clamped to `[0, 1]`).
pub fn scale(c: Color, f: f32) -> Color {
    match c {
        Color::Rgb(r, g, b) => {
            let f = f.clamp(0.0, 1.0);
            Color::Rgb(
                (r as f32 * f).round() as u8,
                (g as f32 * f).round() as u8,
                (b as f32 * f).round() as u8,
            )
        }
        other => other,
    }
}
