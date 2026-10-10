//! The sky gradient (day → dusk → night), stars, and the reed-bed silhouette.

use ratatui::Frame;
use ratatui::style::Color;

use super::colour::{self, Depth};

/// Height, in rows, of the tallest reed clump.
const REED_MAX_ROWS: u16 = 4;

/// Colour of the reed silhouette: nearly black with a little green.
const REED: [u8; 3] = [12, 24, 16];

const STAR: [u8; 3] = [210, 214, 230];

/// Bright arrival sky, from the zenith (`t = 0`) to the horizon (`t = 1`).
const DAY_STOPS: &[(f32, [u8; 3])] = &[
    (0.00, [78, 140, 214]),
    (0.50, [156, 190, 224]),
    (0.80, [248, 206, 142]),
    (1.00, [255, 226, 158]),
];

/// Dusk sky (the display phase).
const DUSK_STOPS: &[(f32, [u8; 3])] = &[
    (0.00, [10, 16, 46]),
    (0.45, [72, 62, 122]),
    (0.72, [188, 96, 112]),
    (0.90, [236, 142, 72]),
    (1.00, [255, 196, 112]),
];

/// Night sky.
const NIGHT_STOPS: &[(f32, [u8; 3])] = &[
    (0.00, [2, 4, 12]),
    (0.55, [5, 7, 20]),
    (0.82, [10, 12, 28]),
    (1.00, [22, 20, 34]),
];

fn stops_colour(stops: &[(f32, [u8; 3])], t: f32) -> [u8; 3] {
    let t = t.clamp(0.0, 1.0);
    let mut lo = stops[0];
    let mut hi = stops[stops.len() - 1];
    for pair in stops.windows(2) {
        if t >= pair[0].0 && t <= pair[1].0 {
            lo = pair[0];
            hi = pair[1];
            break;
        }
    }
    let span = (hi.0 - lo.0).max(f32::EPSILON);
    let f = (t - lo.0) / span;
    let mix = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * f).round() as u8;
    [
        mix(lo.1[0], hi.1[0]),
        mix(lo.1[1], hi.1[1]),
        mix(lo.1[2], hi.1[2]),
    ]
}

fn blend(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    let t = t.clamp(0.0, 1.0);
    let m = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    [m(a[0], b[0]), m(a[1], b[1]), m(a[2], b[2])]
}

/// The sky colour at `t`, for evening brightness (`0` bright gold to `1` night).
pub fn sky_colour(t: f32, evening: f32) -> [u8; 3] {
    if evening < 0.5 {
        blend(
            stops_colour(DAY_STOPS, t),
            stops_colour(DUSK_STOPS, t),
            evening / 0.5,
        )
    } else {
        blend(
            stops_colour(DUSK_STOPS, t),
            stops_colour(NIGHT_STOPS, t),
            (evening - 0.5) / 0.5,
        )
    }
}

/// Number of reed rows in column `x`, swaying gently with `time`.
fn reed_rows(x: u16, width: u16, time: f32) -> u16 {
    let xf = f32::from(x);
    let x = u32::from(x);
    let width = u32::from(width).max(1);
    let base = 2
        + ((x.wrapping_mul(2_654_435_761) >> 28) % 2 + (x.wrapping_mul(40_503) / width) % 2) as u16;
    let sway = (time * 0.5 + xf * 0.25).sin();
    (base as f32 + sway * 0.6)
        .round()
        .clamp(1.0, REED_MAX_ROWS as f32) as u16
}

fn star(x: u16, y: u16) -> bool {
    let mut h = (u32::from(x)).wrapping_mul(73_856_093) ^ (u32::from(y)).wrapping_mul(19_349_663);
    h ^= h >> 13;
    h = h.wrapping_mul(1_274_126_177);
    h ^= h >> 16;
    h % 4096 < 6
}

/// Draws the sky, stars (at night) and reeds into `frame`.
pub fn draw(frame: &mut Frame, evening: f32, time: f32, depth: Depth) {
    let area = frame.area();
    if area.is_empty() {
        return;
    }
    let last = area.height.saturating_sub(1).max(1);
    let buf = frame.buffer_mut();
    for y in 0..area.height {
        let t = y as f32 / last as f32;
        let bg = colour::convert(sky_colour(t, evening), depth);
        for x in 0..area.width {
            let cell = &mut buf[(area.x + x, area.y + y)];
            if evening > 0.75 && y < area.height / 2 && star(x, y) {
                cell.set_symbol("·");
                cell.set_fg(colour::convert(STAR, depth));
            } else {
                cell.set_symbol(" ");
            }
            cell.set_bg(bg);
        }
    }
    let max_rows = REED_MAX_ROWS.min(area.height);
    let reed = colour::convert(REED, depth);
    for x in 0..area.width {
        let rows = reed_rows(x, area.width, time).min(max_rows);
        for k in 0..rows {
            let y = area.y + area.height - 1 - k;
            let cell = &mut buf[(area.x + x, y)];
            cell.set_symbol("█");
            cell.set_fg(reed);
            if depth == Depth::Mono {
                cell.set_bg(Color::Reset);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gradient_endpoints_match_stops() {
        assert_eq!(sky_colour(0.0, 0.0), [78, 140, 214]);
        assert_eq!(sky_colour(1.0, 0.5), [255, 196, 112]);
    }

    #[test]
    fn reeds_stay_within_the_band() {
        for x in 0..200 {
            for time in [0.0f32, 1.0, 5.0] {
                let rows = reed_rows(x, 200, time);
                assert!((1..=4).contains(&rows), "row count {rows} out of band");
            }
        }
    }
}
