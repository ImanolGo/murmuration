//! The sky gradient (day → dusk → night), stars, and the reed-bed silhouette.

use ratatui::Frame;
use ratatui::style::Color;

use super::colour::{self, Depth};
use super::{Camera, dither};

/// Height, in rows, of the tallest reed clump.
pub const REED_MAX_ROWS: u16 = 3;

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
    (0.00, [24, 34, 61]),
    (0.45, [97, 99, 135]),
    (0.72, [185, 132, 137]),
    (0.90, [231, 173, 129]),
    (1.00, [247, 205, 157]),
];

/// Night sky.
const NIGHT_STOPS: &[(f32, [u8; 3])] = &[
    (0.00, [2, 4, 12]),
    (0.55, [5, 7, 20]),
    (0.82, [10, 12, 28]),
    (1.00, [22, 20, 34]),
];

fn stops_colour(stops: &[(f32, [u8; 3])], t: f32) -> [f32; 3] {
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
    let mix = |a: u8, b: u8| a as f32 + (b as f32 - a as f32) * f;
    [
        mix(lo.1[0], hi.1[0]),
        mix(lo.1[1], hi.1[1]),
        mix(lo.1[2], hi.1[2]),
    ]
}

fn blend(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0);
    let m = |x: f32, y: f32| x + (y - x) * t;
    [m(a[0], b[0]), m(a[1], b[1]), m(a[2], b[2])]
}

/// The sky colour at `t`, for evening brightness (`0` bright gold to `1` night).
pub fn sky_colour(t: f32, evening: f32) -> [u8; 3] {
    sky_components(t, evening).map(|v| v.round() as u8)
}

fn sky_components(t: f32, evening: f32) -> [f32; 3] {
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

/// Sample the sky by the elevation of the camera ray, not the screen row.
/// Looking up must not carry the orange horizon along with the flock.
pub fn row_colour(row: u16, height: u16, evening: f32, camera: &Camera) -> [u8; 3] {
    sky_at(row as f32 + 0.5, height, evening, camera)
}

fn sky_at(row: f32, height: u16, evening: f32, camera: &Camera) -> [u8; 3] {
    sky_at_components(row, height, evening, camera).map(|v| v.round() as u8)
}

fn sky_at_components(row: f32, height: u16, evening: f32, camera: &Camera) -> [f32; 3] {
    let y = 1.0 - 2.0 * row / height.max(1) as f32;
    let ray = (camera.forward + camera.up * y * (camera.fov * 0.5).tan()).normalize();
    let elevation = ray.z.clamp(-1.0, 1.0).asin().max(0.0);
    sky_components(1.0 - elevation / std::f32::consts::FRAC_PI_2, evening)
}

fn star(x: u16, y: u16) -> bool {
    let mut h = (u32::from(x)).wrapping_mul(73_856_093) ^ (u32::from(y)).wrapping_mul(19_349_663);
    h ^= h >> 13;
    h = h.wrapping_mul(1_274_126_177);
    h ^= h >> 16;
    h % 4096 < 6
}

/// Draws the sky and stars (at night) into `frame`.
pub fn draw(frame: &mut Frame, evening: f32, depth: Depth, camera: &Camera) {
    let area = frame.area();
    if area.is_empty() {
        return;
    }
    let buf = frame.buffer_mut();
    let texture = dither::texture();
    for y in 0..area.height {
        let lower = sky_at_components(y as f32 + 0.75, area.height, evening, camera);
        let upper = sky_at_components(y as f32 + 0.25, area.height, evening, camera);
        for x in 0..area.width {
            // Sub-one-level stochastic rounding breaks up 8-bit colour bands
            // without animated noise or a visible patterned sky texture.
            let noise =
                texture[(y as usize * 2) & (dither::SIZE - 1)][x as usize & (dither::SIZE - 1)];
            let bg = colour::convert(lower.map(|v| (v + noise).floor() as u8), depth);
            let fg = colour::convert(upper.map(|v| (v + 1.0 - noise).floor() as u8), depth);
            let cell = &mut buf[(area.x + x, area.y + y)];
            if evening > 0.75 && y < area.height / 2 && star(x, y) {
                cell.set_symbol("·");
                cell.set_fg(colour::convert(STAR, depth));
            } else {
                // Two colour samples per cell soften the visible row bands.
                cell.set_symbol(if depth == Depth::Mono { " " } else { "▀" });
                cell.set_fg(fg);
            }
            cell.set_bg(bg);
        }
    }
}

/// How much of the foreground belongs in this view. A soft angular band avoids
/// switching every stem on/off when camera tracking crosses a single threshold.
pub fn reed_visibility(camera: &Camera) -> f32 {
    let bottom_ray = (camera.forward - camera.up * (camera.fov * 0.5).tan()).normalize();
    let t = ((0.25 - bottom_ray.z) / 0.20).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Fine reed stems in a shallow foreground band. Wind bends their tips
/// sideways; it never changes their length into jumping building columns.
/// Draw last so birds cannot paint over the foreground.
pub fn draw_reeds(frame: &mut Frame, time: f32, depth: Depth, camera: &Camera) {
    draw_reeds_with_opacity(frame, time, depth, camera, reed_visibility(camera));
}

/// The renderer eases visibility over time as well as over viewing elevation.
pub(crate) fn draw_reeds_with_opacity(
    frame: &mut Frame,
    time: f32,
    depth: Depth,
    camera: &Camera,
    opacity: f32,
) {
    let area = frame.area();
    if area.is_empty() || opacity < 1.0 / 512.0 {
        return;
    }
    let rows = REED_MAX_ROWS.min(area.height) as usize;
    let width = area.width as usize * 2;
    let height = rows * 4;
    let mut dots = vec![0u32; area.width as usize * rows];
    for base in 0..width {
        let mut hash = (base as u32).wrapping_mul(2_654_435_761);
        hash ^= hash >> 16;
        hash = hash.wrapping_mul(2_246_822_519);
        hash ^= hash >> 13;
        if hash % 100 >= 32 {
            continue;
        }
        let length = 3 + (hash >> 27) as usize % (height.saturating_sub(3).max(1));
        let phase = (hash & 255) as f32 * 0.025;
        let lean = ((hash >> 16) & 15) as f32 / 7.5 - 1.0;
        let sway = lean * 1.8 + (time * 0.65 + phase).sin() * 0.85;
        for k in 0..length.min(height) {
            let bend = sway * (k as f32 / length as f32).powi(2);
            let x = (base as f32 + bend).round() as isize;
            if x < 0 || x >= width as isize {
                continue;
            }
            let x = x as usize;
            let y = height - 1 - k;
            dots[(y / 4) * area.width as usize + x / 2] |= dither::dot_bit(x % 2, y % 4);
            // A narrow seed head makes a reed, rather than a dotted fence post.
            if k + 2 >= length && x + 1 < width {
                dots[(y / 4) * area.width as usize + x.div_ceil(2)] |=
                    dither::dot_bit((x + 1) % 2, y % 4);
            }
        }
    }
    let texture = dither::texture();
    let buf = frame.buffer_mut();
    for y in 0..rows {
        for x in 0..area.width as usize {
            let mut bits = dots[y * area.width as usize + x];
            if depth == Depth::Mono {
                // Without colour, thin the fixed dot pattern progressively.
                // Thresholds stay fixed, so fading adds no animated noise.
                for dy in 0..4 {
                    for dx in 0..2 {
                        if opacity
                            < texture[(y * 4 + dy) & (dither::SIZE - 1)]
                                [(x * 2 + dx) & (dither::SIZE - 1)]
                        {
                            bits &= !dither::dot_bit(dx, dy);
                        }
                    }
                }
            }
            if bits != 0 {
                let row = area.height - rows as u16 + y as u16;
                let cell = &mut buf[(area.x + x as u16, area.y + row)];
                let background = match cell.bg {
                    Color::Rgb(r, g, b) => [r, g, b],
                    Color::Indexed(i) if i >= 232 => [8 + 10 * (i - 232); 3],
                    Color::Indexed(i) if i >= 16 => {
                        let n = i - 16;
                        [n / 36, (n / 6) % 6, n % 6].map(|v| if v == 0 { 0 } else { 55 + 40 * v })
                    }
                    _ => row_colour(row, area.height, 0.5, camera),
                };
                let reed = colour::convert(colour::blend(background, REED, opacity), depth);
                let mut bytes = [0u8; 4];
                cell.set_symbol(dither::braille(bits).encode_utf8(&mut bytes));
                cell.set_fg(reed);
                if depth == Depth::Mono {
                    cell.set_bg(Color::Reset);
                }
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
        assert_eq!(sky_colour(1.0, 0.5), [247, 205, 157]);
    }

    #[test]
    fn sky_follows_elevation_not_the_screen() {
        let mut camera = Camera::new();
        let horizon = row_colour(12, 24, 0.5, &camera);
        camera.forward = glam::Vec3::Z;
        camera.up = glam::Vec3::Y;
        let zenith = row_colour(12, 24, 0.5, &camera);
        assert!(horizon[0] > zenith[0] + 100);
    }

    #[test]
    fn reed_visibility_is_continuous_at_the_old_cutoff() {
        let mut camera = Camera::new();
        // With FOV zero the bottom ray is exactly the chosen gaze elevation.
        camera.fov = 0.0;
        camera.forward = glam::Vec3::new(0.0, (1.0f32 - 0.149f32.powi(2)).sqrt(), 0.149);
        let below = reed_visibility(&camera);
        camera.forward = glam::Vec3::new(0.0, (1.0f32 - 0.151f32.powi(2)).sqrt(), 0.151);
        let above = reed_visibility(&camera);
        assert!((below - above).abs() < 0.02);
        assert!(above > 0.45 && below < 0.55);
    }
}
