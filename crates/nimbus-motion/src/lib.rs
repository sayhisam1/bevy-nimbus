//! Temporal motion history and shader reprojection for Nimbus clouds.

use bevy::{prelude::*, render::render_resource::ShaderType};
use nimbus_core::CloudsUniform;

/// WGSL camera and cloud-motion reprojection shared with the volume marcher.
#[doc(hidden)]
pub const MOTION_WGSL: &str = include_str!("motion.wgsl");

/// Previous-frame camera and time state consumed by GPU reprojection.
#[derive(Clone, Copy, Default, ShaderType)]
#[doc(hidden)]
pub struct NimbusMotionUniform {
    previous_camera_pos: Vec3,
    history_blend: f32,
    previous_camera_forward: Vec3,
    history_valid: u32,
    previous_camera_right: Vec3,
    frame: u32,
    previous_camera_up: Vec3,
    previous_elapsed_seconds: f32,
}

#[derive(Clone)]
struct MotionKey {
    camera_pos: Vec3,
    camera_forward: Vec3,
    camera_right: Vec3,
    camera_up: Vec3,
    elapsed_seconds: f32,
    density: f32,
    movement: Vec2,
    scale: f32,
    base_altitude: f32,
    thickness: f32,
    base_color: Vec3,
}

impl MotionKey {
    fn from_clouds(clouds: &CloudsUniform) -> Self {
        Self {
            camera_pos: clouds.camera_pos,
            camera_forward: clouds.camera_forward,
            camera_right: clouds.camera_right,
            camera_up: clouds.camera_up,
            elapsed_seconds: clouds.elapsed_seconds,
            density: clouds.density,
            movement: clouds.movement,
            scale: clouds.scale,
            base_altitude: clouds.base_altitude,
            thickness: clouds.thickness,
            base_color: clouds.base_color,
        }
    }

    fn camera_moved(&self, current: &Self) -> bool {
        self.camera_pos != current.camera_pos
            || self.camera_forward != current.camera_forward
            || self.camera_right != current.camera_right
            || self.camera_up != current.camera_up
    }

    fn rejects(&self, current: &Self) -> bool {
        let direction_changed =
            self.camera_forward.dot(current.camera_forward) < 10.0_f32.to_radians().cos();
        let focal_ratio = (self.camera_up.length() / current.camera_up.length().max(0.0001))
            .ln()
            .abs();
        !current.camera_pos.is_finite()
            || !current.camera_forward.is_finite()
            || current.camera_pos.distance(self.camera_pos) > 64.0
            || direction_changed
            || focal_ratio > 0.1
            || (current.density - self.density).abs() > 0.05
            || current.movement != self.movement
            || (current.scale - self.scale).abs() > f32::EPSILON
            || (current.base_altitude - self.base_altitude).abs() > f32::EPSILON
            || (current.thickness - self.thickness).abs() > f32::EPSILON
            || current.base_color != self.base_color
            || current.elapsed_seconds < self.elapsed_seconds
            || current.elapsed_seconds - self.elapsed_seconds > 0.25
    }
}

/// Tracks the state required to derive cloud motion between frames.
#[derive(Default)]
#[doc(hidden)]
pub struct NimbusMotionHistory {
    previous: Option<MotionKey>,
    frame: u32,
}

impl NimbusMotionHistory {
    /// Derives the next GPU motion uniform and advances history.
    pub fn begin_frame(&mut self, clouds: &CloudsUniform) -> NimbusMotionUniform {
        let current = MotionKey::from_clouds(clouds);
        let valid = self
            .previous
            .as_ref()
            .is_some_and(|previous| !previous.rejects(&current));
        let moving = self
            .previous
            .as_ref()
            .is_some_and(|previous| previous.camera_moved(&current));
        let previous = self.previous.as_ref().unwrap_or(&current);
        let uniform = NimbusMotionUniform {
            previous_camera_pos: previous.camera_pos,
            history_blend: if moving { 0.82 } else { 0.92 },
            previous_camera_forward: previous.camera_forward,
            history_valid: u32::from(valid),
            previous_camera_right: previous.camera_right,
            frame: self.frame,
            previous_camera_up: previous.camera_up,
            previous_elapsed_seconds: previous.elapsed_seconds,
        };
        self.previous = Some(current);
        self.frame = self.frame.wrapping_add(1);
        uniform
    }

    /// Invalidates previous-frame reprojection state.
    pub fn invalidate(&mut self) {
        self.previous = None;
    }
}

#[cfg(test)]
#[allow(
    clippy::float_cmp,
    reason = "these tests assert exact contract constants"
)]
mod tests {
    use super::NimbusMotionHistory;
    use nimbus_core::CloudsUniform;

    #[test]
    fn history_becomes_valid_and_detects_camera_motion() {
        let mut history = NimbusMotionHistory::default();
        let clouds = CloudsUniform::default();
        let first = history.begin_frame(&clouds);
        let second = history.begin_frame(&clouds);
        let mut moved = clouds;
        moved.camera_pos.x += 1.0;
        let third = history.begin_frame(&moved);
        moved.movement.x += 1.0;
        let changed_field = history.begin_frame(&moved);

        assert_eq!(first.history_valid, 0);
        assert_eq!(second.history_valid, 1);
        assert_eq!(second.history_blend, 0.92);
        assert_eq!(third.history_valid, 1);
        assert_eq!(third.history_blend, 0.82);
        assert_eq!(changed_field.history_valid, 0);
    }
}
