//! Per-view temporal history for quarter-resolution volumetric clouds.

use bevy::{
    math::UVec2,
    prelude::*,
    render::{
        render_resource::{
            AddressMode, Extent3d, FilterMode, Sampler, SamplerDescriptor, Texture,
            TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
            TextureViewDescriptor, UniformBuffer,
        },
        renderer::RenderDevice,
    },
};
use nimbus_core::CloudsUniform;
use nimbus_motion::{NimbusMotionHistory, NimbusMotionUniform};

struct HistoryTextures {
    _colors: [Texture; 2],
    color_views: [TextureView; 2],
    _guides: [Texture; 2],
    guide_views: [TextureView; 2],
    read_index: usize,
    size: UVec2,
}

impl HistoryTextures {
    fn new(device: &RenderDevice, size: UVec2) -> Self {
        let create = |label, format| {
            let texture = device.create_texture(&TextureDescriptor {
                label: Some(label),
                size: Extent3d {
                    width: size.x,
                    height: size.y,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format,
                usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let view = texture.create_view(&TextureViewDescriptor::default());
            (texture, view)
        };
        let color_a = create("nimbus_volume_history_color_a", TextureFormat::Rgba16Float);
        let color_b = create("nimbus_volume_history_color_b", TextureFormat::Rgba16Float);
        let guide_a = create("nimbus_volume_history_guide_a", TextureFormat::Rg16Float);
        let guide_b = create("nimbus_volume_history_guide_b", TextureFormat::Rg16Float);
        Self {
            _colors: [color_a.0, color_b.0],
            color_views: [color_a.1, color_b.1],
            _guides: [guide_a.0, guide_b.0],
            guide_views: [guide_a.1, guide_b.1],
            read_index: 0,
            size,
        }
    }

    pub(crate) fn read_views(&self) -> (&TextureView, &TextureView) {
        (
            &self.color_views[self.read_index],
            &self.guide_views[self.read_index],
        )
    }

    pub(crate) fn write_views(&self) -> (&TextureView, &TextureView) {
        (
            &self.color_views[1 - self.read_index],
            &self.guide_views[1 - self.read_index],
        )
    }

    fn swap(&mut self) {
        self.read_index = 1 - self.read_index;
    }
}

pub(crate) struct VolumeViewState {
    textures: HistoryTextures,
    history_sampler: Sampler,
    motion_history: NimbusMotionHistory,
    pub(crate) temporal_uniform: UniformBuffer<NimbusMotionUniform>,
}

impl VolumeViewState {
    pub(crate) fn new(device: &RenderDevice, size: UVec2) -> Self {
        Self {
            textures: HistoryTextures::new(device, size),
            history_sampler: device.create_sampler(&SamplerDescriptor {
                label: Some("nimbus_volume_history_sampler"),
                address_mode_u: AddressMode::ClampToEdge,
                address_mode_v: AddressMode::ClampToEdge,
                address_mode_w: AddressMode::ClampToEdge,
                mag_filter: FilterMode::Linear,
                min_filter: FilterMode::Linear,
                ..default()
            }),
            motion_history: NimbusMotionHistory::default(),
            temporal_uniform: UniformBuffer::default(),
        }
    }

    pub(crate) fn resize_if_needed(&mut self, device: &RenderDevice, size: UVec2) {
        if self.textures.size != size {
            self.textures = HistoryTextures::new(device, size);
            self.motion_history.invalidate();
        }
    }

    pub(crate) fn begin_frame(&mut self, clouds: &CloudsUniform) {
        self.temporal_uniform
            .set(self.motion_history.begin_frame(clouds));
    }

    pub(crate) fn history_sampler(&self) -> &Sampler {
        &self.history_sampler
    }

    pub(crate) fn read_views(&self) -> (&TextureView, &TextureView) {
        self.textures.read_views()
    }

    pub(crate) fn write_views(&self) -> (&TextureView, &TextureView) {
        self.textures.write_views()
    }

    pub(crate) fn complete_frame(&mut self) {
        self.textures.swap();
    }
}
