//! Bind-group layouts shared by volumetric render routes.

use bevy::render::render_resource::{
    BindGroupLayoutDescriptor, BindGroupLayoutEntries, SamplerBindingType, ShaderStages,
    TextureSampleType,
    binding_types::{sampler, texture_2d, texture_3d, texture_depth_2d, uniform_buffer},
};
use nimbus_core::CloudsUniform;

use nimbus_motion::NimbusMotionUniform;

pub(super) fn volume() -> BindGroupLayoutDescriptor {
    BindGroupLayoutDescriptor::new(
        "nimbus_volume_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                uniform_buffer::<CloudsUniform>(true),
                texture_3d(TextureSampleType::Float { filterable: true }),
                texture_3d(TextureSampleType::Float { filterable: true }),
                texture_2d(TextureSampleType::Float { filterable: false }),
                sampler(SamplerBindingType::Filtering),
                texture_depth_2d(),
                uniform_buffer::<NimbusMotionUniform>(false),
                texture_2d(TextureSampleType::Float { filterable: true }),
                texture_2d(TextureSampleType::Float { filterable: false }),
                sampler(SamplerBindingType::Filtering),
            ),
        ),
    )
}

pub(super) fn composite() -> BindGroupLayoutDescriptor {
    BindGroupLayoutDescriptor::new(
        "nimbus_composite_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: false }),
                sampler(SamplerBindingType::Filtering),
                texture_depth_2d(),
            ),
        ),
    )
}
