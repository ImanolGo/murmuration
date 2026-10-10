//! Blue-noise dithering of opacity into braille dots.
//!
//! Braille gives a 2 × 4 dot grid per cell. Each dot is switched on when its
//! opacity exceeds a threshold from a tiled 64 × 64 blue-noise texture, so thin
//! bird fringes break into individual dots without the screen-door pattern a
//! Bayer matrix gives.

use std::sync::LazyLock;

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

/// Side of the blue-noise texture [dots]. A power of two, so the tiling mask is
/// `SIZE - 1`.
pub const SIZE: usize = 64;

/// Standard deviation of the void-and-cluster filter [dots].
const SIGMA: f32 = 1.9;

/// The tiled blue-noise threshold texture. Index it with the dot's pixel
/// coordinates masked by `SIZE - 1`.
pub fn texture() -> &'static [[f32; SIZE]; SIZE] {
    &TEXTURE
}

/// The bit for the braille dot at cell offset `(dx, dy)`.
pub const fn dot_bit(dx: usize, dy: usize) -> u32 {
    match (dx, dy) {
        (0, 0) => 0x01,
        (0, 1) => 0x02,
        (0, 2) => 0x04,
        (0, 3) => 0x40,
        (1, 0) => 0x08,
        (1, 1) => 0x10,
        (1, 2) => 0x20,
        (1, 3) => 0x80,
        _ => 0,
    }
}

/// The braille character for a set of dot bits.
pub const fn braille(bits: u32) -> char {
    match char::from_u32(0x2800 + bits) {
        Some(c) => c,
        None => ' ',
    }
}

/// The tiled threshold texture, generated once.
static TEXTURE: LazyLock<[[f32; SIZE]; SIZE]> = LazyLock::new(generate);

/// Generates a blue-noise threshold map with Ulichney's void-and-cluster
/// algorithm. The seed is fixed, so the texture is the same every run.
#[allow(clippy::needless_range_loop)]
fn generate() -> [[f32; SIZE]; SIZE] {
    let kernel = kernel();
    let mut rng = ChaCha8Rng::seed_from_u64(0x00B1_0E00);
    let mut on = [[false; SIZE]; SIZE];
    for row in &mut on {
        for v in row {
            *v = rng.random::<bool>();
        }
    }
    let mut filter = filter_map(&on, &kernel);

    // Phase 0: swap the tightest cluster with the largest void until balanced.
    for _ in 0..SIZE * SIZE {
        let (cx, cy) = tightest(&filter, &on);
        let (vx, vy) = largest_void(&filter, &on);
        if filter[cy][cx] <= filter[vy][vx] {
            break;
        }
        on[cy][cx] = false;
        on[vy][vx] = true;
        add(&mut filter, &kernel, cx, cy, -1.0);
        add(&mut filter, &kernel, vx, vy, 1.0);
    }

    let prototype = on;
    let ones = prototype.iter().flatten().filter(|&&b| b).count();
    let mut rank = [[0u16; SIZE]; SIZE];

    // Phase 1: remove the prototype's ones, tightest first; the first removed
    // gets the highest rank (it is the last to switch on as the threshold
    // rises), so the low ranks form a sparse blue-noise pattern.
    for k in (0..ones).rev() {
        let (cx, cy) = tightest(&filter, &on);
        on[cy][cx] = false;
        rank[cy][cx] = k as u16;
        add(&mut filter, &kernel, cx, cy, -1.0);
    }

    // Phase 2: restore the prototype and add its zeros, largest void first.
    on = prototype;
    filter = filter_map(&on, &kernel);
    for k in ones..SIZE * SIZE {
        let (vx, vy) = largest_void(&filter, &on);
        on[vy][vx] = true;
        rank[vy][vx] = k as u16;
        add(&mut filter, &kernel, vx, vy, 1.0);
    }

    let n = (SIZE * SIZE) as f32;
    let mut texture = [[0.0f32; SIZE]; SIZE];
    for y in 0..SIZE {
        for x in 0..SIZE {
            texture[y][x] = (rank[y][x] as f32 + 0.5) / n;
        }
    }
    texture
}

/// The filtered map of a binary pattern.
#[allow(clippy::needless_range_loop)]
fn filter_map(on: &[[bool; SIZE]; SIZE], kernel: &[[f32; SIZE]; SIZE]) -> [[f32; SIZE]; SIZE] {
    let mut filter = [[0.0f32; SIZE]; SIZE];
    for y in 0..SIZE {
        for x in 0..SIZE {
            if on[y][x] {
                add(&mut filter, kernel, x, y, 1.0);
            }
        }
    }
    filter
}

/// The toroidal Gaussian filter.
#[allow(clippy::needless_range_loop)]
fn kernel() -> [[f32; SIZE]; SIZE] {
    let mut k = [[0.0f32; SIZE]; SIZE];
    let two_sigma_sq = 2.0 * SIGMA * SIGMA;
    for y in 0..SIZE {
        for x in 0..SIZE {
            let dx = x.min(SIZE - x) as f32;
            let dy = y.min(SIZE - y) as f32;
            k[y][x] = (-(dx * dx + dy * dy) / two_sigma_sq).exp();
        }
    }
    k
}

/// Adds a Gaussian centred on `(cx, cy)` to the filter map.
fn add(
    filter: &mut [[f32; SIZE]; SIZE],
    kernel: &[[f32; SIZE]; SIZE],
    cx: usize,
    cy: usize,
    delta: f32,
) {
    for (y, row) in filter.iter_mut().enumerate() {
        let dy = (y + SIZE - cy) % SIZE;
        for (x, v) in row.iter_mut().enumerate() {
            let dx = (x + SIZE - cx) % SIZE;
            *v += delta * kernel[dy][dx];
        }
    }
}

/// The on-pixel with the highest filtered value (the tightest cluster).
#[allow(clippy::needless_range_loop)]
fn tightest(filter: &[[f32; SIZE]; SIZE], on: &[[bool; SIZE]; SIZE]) -> (usize, usize) {
    let mut best = (0, 0);
    let mut best_v = f32::NEG_INFINITY;
    for y in 0..SIZE {
        for x in 0..SIZE {
            if on[y][x] && filter[y][x] > best_v {
                best_v = filter[y][x];
                best = (x, y);
            }
        }
    }
    best
}

/// The off-pixel with the lowest filtered value (the largest void).
#[allow(clippy::needless_range_loop)]
fn largest_void(filter: &[[f32; SIZE]; SIZE], on: &[[bool; SIZE]; SIZE]) -> (usize, usize) {
    let mut best = (0, 0);
    let mut best_v = f32::INFINITY;
    for y in 0..SIZE {
        for x in 0..SIZE {
            if !on[y][x] && filter[y][x] < best_v {
                best_v = filter[y][x];
                best = (x, y);
            }
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn braille_all_dots() {
        assert_eq!(braille(0xFF), '⣿');
        assert_eq!(braille(0x00), '⠀');
    }

    #[test]
    fn dot_bits_are_unique() {
        let mut seen = [false; 8];
        for dy in 0..4 {
            for dx in 0..2 {
                let b = dot_bit(dx, dy);
                assert!(b.is_power_of_two());
                assert!(!seen[b.trailing_zeros() as usize]);
                seen[b.trailing_zeros() as usize] = true;
            }
        }
        assert!(seen.iter().all(|&s| s));
    }

    #[test]
    fn texture_is_a_permutation() {
        let mut seen = vec![false; SIZE * SIZE];
        for row in TEXTURE.iter() {
            for &t in row {
                assert!((0.0..1.0).contains(&t), "threshold {t} out of range");
                let v = (t * (SIZE * SIZE) as f32).floor() as usize;
                assert!(!seen[v], "duplicate threshold {v}");
                seen[v] = true;
            }
        }
        assert!(seen.iter().all(|&s| s));
    }

    #[test]
    fn texture_is_blue_noise() {
        // Below 10 % opacity the lit pixels should be spread out: far fewer
        // adjacent pairs than white noise would give.
        let mut low = [[false; SIZE]; SIZE];
        let mut count = 0;
        for (y, row) in low.iter_mut().enumerate() {
            for (x, v) in row.iter_mut().enumerate() {
                *v = texture()[y][x] < 0.1;
                count += usize::from(*v);
            }
        }
        let mut adjacent = 0;
        for y in 0..SIZE {
            for x in 0..SIZE {
                if low[y][x] {
                    if low[y][(x + 1) % SIZE] {
                        adjacent += 1;
                    }
                    if low[(y + 1) % SIZE][x] {
                        adjacent += 1;
                    }
                }
            }
        }
        // White noise would give `count · 2 · p` adjacent pairs; blue noise
        // should give far fewer.
        let white = count as f32 * 2.0 * count as f32 / (SIZE * SIZE) as f32;
        assert!(
            (adjacent as f32) < 0.5 * white,
            "texture is not blue noise: {adjacent} adjacent pairs, white-noise baseline {white:.0}"
        );
    }
}
