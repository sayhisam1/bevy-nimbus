//! CPU-to-GPU cloud uniform data.

use bevy::{
    prelude::*,
    render::extract_resource::ExtractResource,
    render::render_resource::{ShaderType, UniformBuffer},
};

/// Maximum directional lights represented by the cloud GPU ABI.
pub const MAX_DIRECTIONAL_LIGHTS: usize = 4;

/// GPU representation of one directional cloud light.
#[derive(Clone, Copy, Debug, Default, ShaderType)]
#[doc(hidden)]
pub struct DirectionalLightUniform {
    pub direction_to_light: Vec3,
    pub strength: f32,
    pub color: Vec3,
}

#[derive(Resource, Clone, ExtractResource, ShaderType)]
#[doc(hidden)]
pub struct CloudsUniform {
    pub camera_pos: Vec3,
    pub elapsed_seconds: f32,
    pub camera_forward: Vec3,
    pub directional_light_count: u32,
    pub camera_right: Vec3,
    pub ambient_strength: f32,
    pub camera_up: Vec3,
    pub density: f32,
    pub ambient_color: Vec3,
    pub scale: f32,
    pub movement: Vec2,
    pub base_altitude: f32,
    pub thickness: f32,
    pub base_color: Vec3,
    pub view_steps: u32,
    pub directional_lights: [DirectionalLightUniform; MAX_DIRECTIONAL_LIGHTS],
    pub light_steps: u32,
    pub resolution_divisor: u32,
}

impl Default for CloudsUniform {
    fn default() -> Self {
        let mut directional_lights = [DirectionalLightUniform::default(); MAX_DIRECTIONAL_LIGHTS];
        directional_lights[0] = DirectionalLightUniform {
            direction_to_light: Vec3::NEG_X,
            strength: 1.0,
            color: Vec3::ONE,
        };
        Self {
            camera_pos: Vec3::ZERO,
            elapsed_seconds: 0.0,
            camera_forward: Vec3::NEG_Z,
            directional_light_count: 1,
            camera_right: Vec3::X * 0.577_350_26 * 1.777_777_8,
            ambient_strength: 0.1,
            camera_up: Vec3::Y * 0.577_350_26,
            density: 1.0,
            ambient_color: Vec3::ONE,
            scale: 1.0,
            movement: Vec2::ZERO,
            base_altitude: 1500.0,
            thickness: 1000.0,
            base_color: Vec3::ONE,
            view_steps: 48,
            directional_lights,
            light_steps: 4,
            resolution_divisor: 2,
        }
    }
}

/// Render-world storage for the current cloud uniform.
#[derive(Resource, Default)]
#[doc(hidden)]
pub struct CloudsUniformBuffer(pub UniformBuffer<CloudsUniform>);
