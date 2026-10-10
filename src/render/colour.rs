//! Colour handling and the fallbacks for terminals without true colour.

use ratatui::style::Color;

/// Requested colour handling.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, clap::ValueEnum)]
pub enum Mode {
    /// Detect from the environment.
    #[default]
    Auto,
    /// 24-bit colour.
    Truecolor,
    /// Nearest of the 256 xterm colours.
    Ansi256,
    /// Plain braille, no colour.
    None,
}

/// Resolved colour depth.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Depth {
    /// 24-bit colour.
    TrueColor,
    /// 256 xterm colours.
    Ansi256,
    /// Terminal default colours.
    Mono,
}

impl Mode {
    /// Resolves a mode against the environment: `NO_COLOR` and `COLORTERM`.
    pub fn resolve(self) -> Depth {
        match self {
            Mode::Truecolor => Depth::TrueColor,
            Mode::Ansi256 => Depth::Ansi256,
            Mode::None => Depth::Mono,
            Mode::Auto => {
                if std::env::var_os("NO_COLOR").is_some() {
                    Depth::Mono
                } else if std::env::var("COLORTERM")
                    .map(|v| v.eq_ignore_ascii_case("truecolor") || v.eq_ignore_ascii_case("24bit"))
                    .unwrap_or(false)
                {
                    Depth::TrueColor
                } else {
                    Depth::Ansi256
                }
            }
        }
    }
}

/// Converts an RGB colour to a terminal colour at the given depth.
pub fn convert(rgb: [u8; 3], depth: Depth) -> Color {
    match depth {
        Depth::TrueColor => Color::Rgb(rgb[0], rgb[1], rgb[2]),
        Depth::Ansi256 => Color::Indexed(nearest_256(rgb)),
        Depth::Mono => Color::Reset,
    }
}

/// Scales an RGB colour's channels by `f` (clamped to `[0, 1]`).
pub fn scale(rgb: [u8; 3], f: f32) -> [u8; 3] {
    let f = f.clamp(0.0, 1.0);
    [
        (rgb[0] as f32 * f).round() as u8,
        (rgb[1] as f32 * f).round() as u8,
        (rgb[2] as f32 * f).round() as u8,
    ]
}

fn cube_level(n: i32) -> i32 {
    if n == 0 { 0 } else { 55 + 40 * n }
}

/// The index of the nearest xterm-256 colour.
fn nearest_256(rgb: [u8; 3]) -> u8 {
    let (r, g, b) = (rgb[0] as i32, rgb[1] as i32, rgb[2] as i32);
    let q = |c: i32| -> i32 {
        if c < 48 {
            0
        } else if c < 115 {
            1
        } else {
            (c - 35) / 40
        }
    };
    let (qr, qg, qb) = (q(r), q(g), q(b));
    let cube = [cube_level(qr), cube_level(qg), cube_level(qb)];
    let cube_index = (16 + 36 * qr + 6 * qg + qb) as u8;

    let gray = (r + g + b) / 3;
    let gray_level = ((gray - 8).clamp(0, 23) * 10 + 8).clamp(8, 238);
    let gray_index = (232 + ((gray - 8).clamp(0, 23))) as u8;

    let d2 = |a: [i32; 3], c: [i32; 3]| {
        let dr = a[0] - c[0];
        let dg = a[1] - c[1];
        let db = a[2] - c[2];
        dr * dr + dg * dg + db * db
    };
    if d2([r, g, b], [gray_level, gray_level, gray_level]) < d2([r, g, b], cube) {
        gray_index
    } else {
        cube_index
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_and_white_map_sensibly() {
        assert_eq!(nearest_256([0, 0, 0]), 16);
        assert_eq!(nearest_256([255, 255, 255]), 231);
    }

    #[test]
    fn no_color_is_mono() {
        assert_eq!(Mode::None.resolve(), Depth::Mono);
    }
}
