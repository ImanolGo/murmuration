//! A ground observer looking up at the flock.
//!
//! The eye sits at human height a slowly-eased horizontal distance from the
//! roost, chosen so the flock fills about 60 % of the screen. It looks toward
//! the flock centroid, smoothed over a couple of seconds.

use glam::Vec3;

use crate::sim::World;

/// Eye height of the observer [m].
pub const EYE_HEIGHT: f32 = 1.7;

/// Vertical field of view [rad].
pub const FOV_V: f32 = 50.0 * std::f32::consts::PI / 180.0;

/// Fraction of the vertical field the flock should span.
const FILL: f32 = 0.6;

/// A perspective camera on the ground.
pub struct Camera {
    /// Eye position [m].
    pub eye: Vec3,
    /// Unit view direction.
    pub forward: Vec3,
    /// Unit right direction.
    pub right: Vec3,
    /// Unit up direction.
    pub up: Vec3,
    /// Bearing around the roost [rad], changed by the `c` key.
    pub bearing: f32,
    distance: f32,
    smoothed_target: Vec3,
    ready: bool,
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

impl Camera {
    /// A camera south of the roost looking north.
    pub fn new() -> Self {
        Self {
            eye: Vec3::new(0.0, -200.0, EYE_HEIGHT),
            forward: Vec3::Y,
            right: Vec3::X,
            up: Vec3::Z,
            bearing: -std::f32::consts::FRAC_PI_2,
            distance: 200.0,
            smoothed_target: Vec3::ZERO,
            ready: false,
        }
    }

    /// Advances the camera by `dt` seconds to follow `world`.
    pub fn update(&mut self, world: &World, dt: f32) {
        if world.is_empty() {
            return;
        }
        let n = world.len() as f32;
        let centroid = world.position.iter().copied().sum::<Vec3>() / n;
        if !self.ready {
            self.smoothed_target = centroid;
            self.ready = true;
        }
        // Smooth the look target (time constant ~2 s).
        self.smoothed_target += (centroid - self.smoothed_target) * (1.0 - (-dt / 2.0).exp());

        // Characteristic radius of the flock, then the distance that makes it
        // span FILL of the vertical field of view. Use the 90th percentile of
        // distances so a few stragglers do not push the camera far away.
        let mut distances: Vec<f32> = world
            .position
            .iter()
            .map(|p| (*p - centroid).length())
            .collect();
        let index = ((distances.len() as f32 * 0.9) as usize).min(distances.len() - 1);
        distances.select_nth_unstable_by(index, |a, b| {
            a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
        });
        let radius = distances[index].max(1.0);
        let target_distance = radius / (0.5 * FILL * FOV_V).tan();
        let target_distance = target_distance.max(EYE_HEIGHT + 5.0);
        self.distance += (target_distance - self.distance) * (1.0 - (-dt / 3.0).exp());
        self.distance = self.distance.max(EYE_HEIGHT + 5.0);

        // Horizontal stand-off that puts the eye `distance` from the target.
        let height = self.smoothed_target.z - EYE_HEIGHT;
        let horizontal = (self.distance * self.distance - height * height)
            .max(1.0)
            .sqrt();
        self.eye = Vec3::new(
            horizontal * self.bearing.cos(),
            horizontal * self.bearing.sin(),
            EYE_HEIGHT,
        );

        let mut forward = self.smoothed_target - self.eye;
        if forward.length_squared() < 1e-6 {
            forward = Vec3::Y;
        }
        self.forward = forward.normalize();
        let mut right = self.forward.cross(Vec3::Z);
        if right.length_squared() < 1e-6 {
            right = Vec3::X;
        }
        self.right = right.normalize();
        self.up = self.right.cross(self.forward);
    }

    /// Focal length in pixels for a pixel grid `pixel_h` dots tall.
    pub fn focal_px(&self, pixel_h: usize) -> f32 {
        (pixel_h as f32 / 2.0) / (FOV_V / 2.0).tan()
    }

    /// Projects `p` to pixel coordinates `(x, y, depth)`, or `None` if it is
    /// behind the camera.
    pub fn project(&self, p: Vec3, pixel_w: usize, pixel_h: usize) -> Option<(f32, f32, f32)> {
        let rel = p - self.eye;
        let depth = rel.dot(self.forward);
        if depth <= 1.0 {
            return None;
        }
        let f = self.focal_px(pixel_h);
        let x = rel.dot(self.right);
        let y = rel.dot(self.up);
        Some((
            pixel_w as f32 / 2.0 + x / depth * f,
            pixel_h as f32 / 2.0 - y / depth * f,
            depth,
        ))
    }

    /// The point under the cursor, on the plane through `plane_point` facing the
    /// camera. `col`/`row` are terminal cell coordinates.
    pub fn cursor_to_plane(
        &self,
        col: u16,
        row: u16,
        pixel_w: usize,
        pixel_h: usize,
        plane_point: Vec3,
    ) -> Vec3 {
        let f = self.focal_px(pixel_h);
        let ndc_x = (col as f32 * 2.0 - pixel_w as f32 / 2.0) / f;
        let ndc_y = (row as f32 * 4.0 - pixel_h as f32 / 2.0) / f;
        let dir = (self.forward + self.right * ndc_x - self.up * ndc_y).normalize();
        let normal = self.forward;
        let denom = dir.dot(normal);
        if denom.abs() < 1e-6 {
            return plane_point;
        }
        let t = (plane_point - self.eye).dot(normal) / denom;
        self.eye + dir * t
    }
}
