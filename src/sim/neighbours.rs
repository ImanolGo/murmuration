//! A uniform spatial hash grid for neighbour queries.
//!
//! Birds are bucketed into cubic cells of a fixed size; a query walks only the
//! cells its radius can touch. Rebuilding is O(N) and clearing touches only the
//! cells that were occupied.

use glam::Vec3;

use super::params::GRID_CELL;

/// A uniform grid of cell size `cell`, covering `[min, max]`.
pub struct Grid {
    origin: Vec3,
    inv_cell: f32,
    dims: [i32; 3],
    /// First bird index per cell, or −1. Length `dims.x·dims.y·dims.z`.
    head: Vec<i32>,
    /// Next bird in the same cell, or −1. Length N.
    next: Vec<i32>,
    /// Cells occupied in the current build (so rebuild clears only those).
    touched: Vec<u32>,
}

impl Grid {
    /// Builds an empty grid of the given cell size covering `[min, max]`.
    pub fn new(cell: f32, min: Vec3, max: Vec3) -> Self {
        let dims = [
            (((max.x - min.x) / cell).ceil() as i32).max(1),
            (((max.y - min.y) / cell).ceil() as i32).max(1),
            (((max.z - min.z) / cell).ceil() as i32).max(1),
        ];
        let cells = (dims[0] as usize) * (dims[1] as usize) * (dims[2] as usize);
        Self {
            origin: min,
            inv_cell: 1.0 / cell,
            dims,
            head: vec![-1; cells],
            next: Vec::new(),
            touched: Vec::new(),
        }
    }

    /// A grid covering `positions` with a margin of `margin` metres.
    pub fn around(cell: f32, positions: &[Vec3], margin: f32) -> Self {
        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);
        for &p in positions {
            min = min.min(p);
            max = max.max(p);
        }
        if positions.is_empty() {
            min = Vec3::ZERO;
            max = Vec3::splat(cell);
        }
        Self::new(cell, min - Vec3::splat(margin), max + Vec3::splat(margin))
    }

    /// Rebuilds the buckets from `positions`.
    pub fn rebuild(&mut self, positions: &[Vec3]) {
        for &c in &self.touched {
            self.head[c as usize] = -1;
        }
        self.touched.clear();
        self.next.resize(positions.len(), -1);
        for (i, &p) in positions.iter().enumerate() {
            let cell = self.index_for(p);
            if self.head[cell] == -1 {
                self.touched.push(cell as u32);
            }
            self.next[i] = self.head[cell];
            self.head[cell] = i as i32;
        }
    }

    fn coord_unclamped(&self, p: Vec3) -> [i32; 3] {
        let f = (p - self.origin) * self.inv_cell;
        [f.x.floor() as i32, f.y.floor() as i32, f.z.floor() as i32]
    }

    fn coord_clamped(&self, p: Vec3) -> [i32; 3] {
        let c = self.coord_unclamped(p);
        [
            c[0].clamp(0, self.dims[0] - 1),
            c[1].clamp(0, self.dims[1] - 1),
            c[2].clamp(0, self.dims[2] - 1),
        ]
    }

    fn linear(&self, c: [i32; 3]) -> usize {
        (c[0] + c[1] * self.dims[0] + c[2] * self.dims[0] * self.dims[1]) as usize
    }

    fn index_for(&self, p: Vec3) -> usize {
        self.linear(self.coord_clamped(p))
    }

    /// Calls `f(j, distance)` for every `j` whose position is within `radius`
    /// of `p`. Includes `j == i` when querying a bird against itself.
    pub fn for_each_within(
        &self,
        p: Vec3,
        radius: f32,
        positions: &[Vec3],
        mut f: impl FnMut(usize, f32),
    ) {
        let lo = self.coord_unclamped(p - Vec3::splat(radius));
        let hi = self.coord_unclamped(p + Vec3::splat(radius));
        let lo = [lo[0].max(0), lo[1].max(0), lo[2].max(0)];
        let hi = [
            hi[0].min(self.dims[0] - 1),
            hi[1].min(self.dims[1] - 1),
            hi[2].min(self.dims[2] - 1),
        ];
        if lo[0] > hi[0] || lo[1] > hi[1] || lo[2] > hi[2] {
            return;
        }
        let r2 = radius * radius;
        for cz in lo[2]..=hi[2] {
            for cy in lo[1]..=hi[1] {
                for cx in lo[0]..=hi[0] {
                    let mut j = self.head[self.linear([cx, cy, cz])];
                    while j >= 0 {
                        let jj = j as usize;
                        let d2 = (positions[jj] - p).length_squared();
                        if d2 <= r2 {
                            f(jj, d2.sqrt());
                        }
                        j = self.next[jj];
                    }
                }
            }
        }
    }
}

/// Convenience constructor with the default cell size.
pub fn default_grid(positions: &[Vec3]) -> Grid {
    Grid::around(GRID_CELL, positions, 20.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_only_birds_within_radius() {
        let pos = vec![
            Vec3::ZERO,
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(3.0, 0.0, 0.0),
            Vec3::new(10.0, 0.0, 0.0),
        ];
        let mut grid = Grid::around(1.0, &pos, 1.0);
        grid.rebuild(&pos);
        let mut found = Vec::new();
        grid.for_each_within(Vec3::ZERO, 2.5, &pos, |j, _| found.push(j));
        found.sort_unstable();
        assert_eq!(found, vec![0, 1]);
    }

    #[test]
    fn rebuild_clears_old_cells() {
        let first = vec![Vec3::ZERO, Vec3::new(50.0, 0.0, 0.0)];
        let second = vec![Vec3::new(50.0, 0.0, 0.0)];
        let mut grid = Grid::around(1.0, &first, 1.0);
        grid.rebuild(&first);
        grid.rebuild(&second);
        let mut found = Vec::new();
        grid.for_each_within(Vec3::ZERO, 5.0, &second, |j, _| found.push(j));
        assert!(found.is_empty());
    }

    #[test]
    fn grid_matches_bruteforce() {
        let mut state = 12345u64;
        let mut rnd = move || {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((state >> 33) as f32 / (1u64 << 31) as f32) - 1.0
        };
        let pos: Vec<Vec3> = (0..800)
            .map(|_| Vec3::new(rnd() * 30.0, rnd() * 30.0, rnd() * 10.0))
            .collect();
        let mut grid = Grid::around(4.0, &pos, 5.0);
        grid.rebuild(&pos);
        for i in 0..pos.len() {
            for &r in &[1.0f32, 3.0, 8.0, 20.0] {
                let mut got = Vec::new();
                grid.for_each_within(pos[i], r, &pos, |j, _| got.push(j));
                got.sort_unstable();
                let mut want: Vec<usize> = (0..pos.len())
                    .filter(|&j| (pos[j] - pos[i]).length() <= r)
                    .collect();
                want.sort_unstable();
                assert_eq!(got, want, "i={i} r={r}");
            }
        }
    }
}
