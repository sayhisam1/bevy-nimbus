//! Scene-wide cloud shadow ownership and GPU rendering.

mod render;

use bevy::{
    asset::RenderAssetUsages,
    core_pipeline::{Core3dSystems, schedule::Core3d},
    image::{Image, ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    prelude::*,
    render::{
        RenderApp, RenderStartup,
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
    },
};
use nimbus_volume::DENSITY_WGSL;

const SHADOW_MAP_SIZE: u32 = 256;
const SHADOW_MARCH_WGSL: &str = include_str!("shadow.wgsl");
const SHADOW_SHADER: &str = "nimbus_volume_shadow.wgsl";

/// Connects the shadow-source camera to a cloud-volume entity.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[require(Camera3d)]
pub struct NimbusCloudShadowSource {
    /// Cloud volume rendered into the scene-wide shadow map.
    #[entities]
    pub volume: Entity,
}

impl NimbusCloudShadowSource {
    /// Uses `volume` as this camera's cloud-shadow field.
    #[must_use]
    pub const fn new(volume: Entity) -> Self {
        Self { volume }
    }
}

/// Directional cloud transmittance map for the primary cloud light.
#[derive(Resource, Clone, Debug, ExtractResource)]
pub struct NimbusCloudShadowMap {
    /// True when exactly one source exists.
    pub enabled: bool,
    /// R-channel transmittance over an 8192-unit square centered on the world origin.
    pub texture: Handle<Image>,
}

/// Installs cloud-shadow resources and GPU rendering.
#[derive(Debug, Default)]
#[doc(hidden)]
pub struct NimbusCloudShadowPlugin;

impl Plugin for NimbusCloudShadowPlugin {
    fn build(&self, app: &mut App) {
        let shader = {
            let mut shaders = app.world_mut().resource_mut::<Assets<Shader>>();
            shaders.add(Shader::from_wgsl(
                format!("{DENSITY_WGSL}\n{SHADOW_MARCH_WGSL}"),
                SHADOW_SHADER,
            ))
        };
        app.add_plugins(ExtractResourcePlugin::<NimbusCloudShadowMap>::default())
            .add_systems(Startup, create_shadow_map)
            .add_systems(Update, update_shadow_map);
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app
            .insert_resource(render::ShadowShader(shader))
            .add_systems(RenderStartup, render::init_shadow_pipeline)
            .add_systems(
                Core3d,
                render::render_shadow_coverage.before(Core3dSystems::MainPass),
            );
    }
}

fn blank_map_image() -> Image {
    let texels = [255u8].repeat((SHADOW_MAP_SIZE * SHADOW_MAP_SIZE) as usize);
    let mut image = Image::new(
        Extent3d {
            width: SHADOW_MAP_SIZE,
            height: SHADOW_MAP_SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        texels,
        TextureFormat::R8Unorm,
        RenderAssetUsages::default(),
    );
    image.texture_descriptor.usage =
        TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST;
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::ClampToEdge,
        address_mode_v: ImageAddressMode::ClampToEdge,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        ..Default::default()
    });
    image
}

fn create_shadow_map(mut images: ResMut<Assets<Image>>, mut commands: Commands) {
    let texture = images.add(blank_map_image());
    commands.insert_resource(NimbusCloudShadowMap {
        enabled: false,
        texture,
    });
}

fn update_shadow_map(
    sources: Query<&NimbusCloudShadowSource>,
    mut map: ResMut<NimbusCloudShadowMap>,
) {
    let mut sources = sources.iter();
    map.enabled = sources.next().is_some();
    assert!(
        sources.next().is_none(),
        "Nimbus supports one NimbusCloudShadowSource"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shadow_map_is_high_resolution_and_does_not_repeat() {
        let image = blank_map_image();
        assert_eq!(image.texture_descriptor.size.width, 256);
        assert_eq!(image.texture_descriptor.size.height, 256);
        let ImageSampler::Descriptor(sampler) = image.sampler else {
            panic!("shadow map must use an explicit sampler");
        };
        assert_eq!(sampler.address_mode_u, ImageAddressMode::ClampToEdge);
        assert_eq!(sampler.address_mode_v, ImageAddressMode::ClampToEdge);
    }

    #[test]
    fn shadow_uv_maps_to_world_xz_not_xy() {
        assert!(SHADOW_MARCH_WGSL.contains("vec3<f32>(receiver_xz.x, 0.0, receiver_xz.y)",));
        assert!(!SHADOW_MARCH_WGSL.contains("vec3<f32>((in.uv - 0.5) * SHADOW_WORLD_SIZE, 0.0)",));
    }
}
