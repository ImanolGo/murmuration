//! The sky gradient and the reed-bed silhouette.

use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

/// Height, in rows, of the tallest reed clump.
const REED_MAX_ROWS: u16 = 4;

/// Colour of the reed silhouette: nearly black with a little green.
const REED: Color = Color::Rgb(12, 24, 16);

/// Stops of the dusk gradient, from the zenith (`t = 0`) to the horizon
/// (`t = 1`).
const DUSK_STOPS: &[(f32, [u8; 3])] = &[
    (0.00, [10, 16, 46]),
    (0.45, [72, 62, 122]),
    (0.72, [188, 96, 112]),
    (0.90, [236, 142, 72]),
    (1.00, [255, 196, 112]),
];

/// The sky colour at `t`, where 0 is the zenith and 1 the horizon.
pub fn dusk_colour(t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let mut lo = DUSK_STOPS[0];
    let mut hi = DUSK_STOPS[DUSK_STOPS.len() - 1];
    for pair in DUSK_STOPS.windows(2) {
        if t >= pair[0].0 && t <= pair[1].0 {
            lo = pair[0];
            hi = pair[1];
            break;
        }
    }
    let span = (hi.0 - lo.0).max(f32::EPSILON);
    let f = (t - lo.0) / span;
    let mix = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * f).round() as u8;
    Color::Rgb(
        mix(lo.1[0], hi.1[0]),
        mix(lo.1[1], hi.1[1]),
        mix(lo.1[2], hi.1[2]),
    )
}

/// Number of reed rows in column `x`, for a view `width` cells wide.
fn reed_rows(x: u16, width: u16) -> u16 {
    let x = u32::from(x);
    let width = u32::from(width).max(1);
    // A cheap deterministic roughness: two incommensurate sawtooths.
    let a = (x.wrapping_mul(2_654_435_761) >> 28) % 2;
    let b = (x.wrapping_mul(40_503) / width) % 2;
    2 + (a + b) as u16
}

/// Draws the dusk sky and the reeds into `frame`.
pub fn draw(frame: &mut Frame) {
    let area = frame.area();
    if area.is_empty() {
        return;
    }
    let buf = frame.buffer_mut();
    paint_sky(buf, area);
    paint_reeds(buf, area);
}

fn paint_sky(buf: &mut Buffer, area: Rect) {
    let last = area.height.saturating_sub(1).max(1);
    for y in 0..area.height {
        let t = y as f32 / last as f32;
        let colour = dusk_colour(t);
        for x in 0..area.width {
            let cell = &mut buf[(area.x + x, area.y + y)];
            cell.set_symbol(" ");
            cell.set_bg(colour);
        }
    }
}

fn paint_reeds(buf: &mut Buffer, area: Rect) {
    let max_rows = REED_MAX_ROWS.min(area.height);
    for x in 0..area.width {
        let rows = reed_rows(x, area.width).min(max_rows);
        for k in 0..rows {
            let y = area.y + area.height - 1 - k;
            let cell = &mut buf[(area.x + x, y)];
            cell.set_symbol("█");
            cell.set_fg(REED);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gradient_endpoints_match_stops() {
        assert_eq!(dusk_colour(0.0), Color::Rgb(10, 16, 46));
        assert_eq!(dusk_colour(1.0), Color::Rgb(255, 196, 112));
    }

    #[test]
    fn gradient_is_clamped_outside_unit_range() {
        assert_eq!(dusk_colour(-5.0), dusk_colour(0.0));
        assert_eq!(dusk_colour(5.0), dusk_colour(1.0));
    }

    #[test]
    fn reeds_stay_within_the_band() {
        for x in 0..200 {
            let rows = reed_rows(x, 200);
            assert!((2..=4).contains(&rows), "row count {rows} out of band");
        }
    }
}
