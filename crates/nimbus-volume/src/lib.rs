//! Nubis texture loading and GPU access for true-volumetric Nimbus clouds.

use bevy::{
    asset::{LoadState, embedded_asset, load_embedded_asset},
    image::{
        ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler,
        ImageSamplerDescriptor,
    },
    prelude::*,
    render::{extract_resource::ExtractResource, render_asset::RenderAssets, texture::GpuImage},
};

#[doc(hidden)]
pub const DENSITY_WGSL: &str = include_str!("density.wgsl");
const VOLUME_MARCH_WGSL: &str = include_str!("volume.wgsl");

/// Composes the shared Nubis density model with the production view marcher.
#[must_use]
#[doc(hidden)]
pub fn volume_shader(lighting_wgsl: &str, motion_wgsl: &str) -> String {
    format!("{DENSITY_WGSL}\n{lighting_wgsl}\n{motion_wgsl}\n{VOLUME_MARCH_WGSL}")
}

/// Linear, repeat-sampled Nubis density inputs.
#[derive(Resource, Clone, Debug, ExtractResource)]
#[doc(hidden)]
pub struct NubisTextures {
    pub shape: Handle<Image>,
    pub detail: Handle<Image>,
    pub weather: Handle<Image>,
}

fn configure_noise(settings: &mut ImageLoaderSettings) {
    settings.is_srgb = false;
    settings.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        address_mode_w: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Linear,
        ..default()
    });
}

/// Registers the canonical Nubis density textures embedded in this crate.
#[doc(hidden)]
pub fn register_nubis_assets(app: &mut App) {
    embedded_asset!(app, "assets/nubis/shape.ktx2");
    embedded_asset!(app, "assets/nubis/detail.ktx2");
    embedded_asset!(app, "assets/nubis/weather.ktx2");
}

/// Loads the canonical embedded Nubis density textures.
#[expect(clippy::needless_pass_by_value, reason = "Bevy system parameters")]
#[doc(hidden)]
pub fn load_nubis_textures(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(NubisTextures {
        shape: load_embedded_asset!(
            asset_server.as_ref(),
            "assets/nubis/shape.ktx2",
            configure_noise
        ),
        detail: load_embedded_asset!(
            asset_server.as_ref(),
            "assets/nubis/detail.ktx2",
            configure_noise
        ),
        weather: load_embedded_asset!(
            asset_server.as_ref(),
            "assets/nubis/weather.ktx2",
            configure_noise
        ),
    });
}

/// Panics if any required embedded Nubis texture fails to load.
///
/// # Panics
/// Panics with the underlying Bevy asset error after a required texture enters
/// [`LoadState::Failed`].
#[expect(clippy::needless_pass_by_value, reason = "Bevy system parameters")]
#[doc(hidden)]
pub fn validate_nubis_textures(textures: Res<NubisTextures>, asset_server: Res<AssetServer>) {
    for (name, id) in [
        ("shape", textures.shape.id().untyped()),
        ("detail", textures.detail.id().untyped()),
        ("weather", textures.weather.id().untyped()),
    ] {
        if let LoadState::Failed(error) = asset_server.load_state(id) {
            panic!("failed to load embedded Nubis {name} texture: {error}");
        }
    }
}

#[must_use]
#[doc(hidden)]
pub fn gpu_nubis_textures<'a>(
    textures: &NubisTextures,
    images: &'a RenderAssets<GpuImage>,
) -> Option<[&'a GpuImage; 3]> {
    Some([
        images.get(&textures.shape)?,
        images.get(&textures.detail)?,
        images.get(&textures.weather)?,
    ])
}
