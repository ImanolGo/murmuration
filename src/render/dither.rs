//! Ordered dithering of opacity into braille dots.
//!
//! Braille gives a 2 × 4 dot grid per cell. Each dot is switched on when its
//! opacity exceeds its threshold from a 2 × 4 Bayer matrix, so thin bird
//! fringes break into individual dots instead of grey mush.

/// Bayer thresholds, indexed `[row][col]`, values `0..8`.
const BAYER: [[u8; 2]; 4] = [[0, 4], [6, 2], [1, 5], [7, 3]];

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

/// The Bayer threshold in `[0, 1)` for cell offset `(dx, dy)`.
pub const fn threshold(dx: usize, dy: usize) -> f32 {
    (BAYER[dy][dx] as f32 + 0.5) / 8.0
}

/// The braille character for a set of dot bits.
pub const fn braille(bits: u32) -> char {
    match char::from_u32(0x2800 + bits) {
        Some(c) => c,
        None => ' ',
    }
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
}
