//! True-volumetric Nubis cloud rendering for Bevy 0.19.
#![expect(
    clippy::needless_pass_by_value,
    clippy::too_many_arguments,
    reason = "cloud sampling math and Bevy system signatures require these"
)]
mod node;
mod pipeline;
mod render;
mod temporal;

use bevy::prelude::*;
use nimbus_core::CloudsUniform;
pub use nimbus_core::{
    CloudPreset, CloudSettings, NimbusAmbientLight, NimbusCloudView, NimbusCloudVolume,
    NimbusSampling,
};
use nimbus_lighting::{LIGHTING_WGSL, NimbusCloudShadowPlugin, NimbusLightingPlugin};
pub use nimbus_lighting::{
    NimbusCloudLight, NimbusCloudLightKind, NimbusCloudShadowMap, NimbusCloudShadowSource,
};
use nimbus_motion::MOTION_WGSL;
use nimbus_volume::{
    NubisTextures, load_nubis_textures, register_nubis_assets, validate_nubis_textures,
    volume_shader,
};

/// Installs volumetric rendering, temporal history, and cloud shadows.
#[derive(Debug, Default)]
pub struct NimbusPlugin;

impl Plugin for NimbusPlugin {
    fn build(&self, app: &mut App) {
        register_nubis_assets(app);
        app.add_plugins((NimbusLightingPlugin, NimbusCloudShadowPlugin));
        let handles = {
            let mut shaders = app.world_mut().resource_mut::<Assets<Shader>>();
            pipeline::CloudShaders {
                volume: shaders.add(Shader::from_wgsl(
                    volume_shader(LIGHTING_WGSL, MOTION_WGSL),
                    "nimbus_volume.wgsl",
                )),
                composite: shaders.add(Shader::from_wgsl(
                    pipeline::COMPOSITE_WGSL,
                    "nimbus_composite.wgsl",
                )),
            }
        };
        if let Some(render_app) = app.get_sub_app_mut(bevy::render::RenderApp) {
            render_app.insert_resource(handles);
        }
        app.init_resource::<CloudsUniform>()
            .add_observer(validate_inserted_view)
            .add_plugins((
                bevy::render::extract_component::ExtractComponentPlugin::<NimbusCloudView>::default(
                ),
                bevy::render::extract_resource::ExtractResourcePlugin::<CloudsUniform>::default(),
                bevy::render::extract_resource::ExtractResourcePlugin::<NubisTextures>::default(),
            ))
            .add_systems(Startup, load_nubis_textures)
            .add_systems(Update, validate_nubis_textures);
        node::install(app);
    }
}

#[expect(clippy::needless_pass_by_value, reason = "Bevy observer parameter")]
fn validate_inserted_view(
    insert: On<Insert, NimbusCloudView>,
    views: Query<(Has<bevy::camera::Hdr>, &Msaa), With<NimbusCloudView>>,
) {
    let Ok((hdr, msaa)) = views.get(insert.entity) else {
        return;
    };
    assert!(hdr, "NimbusCloudView requires Hdr");
    assert_eq!(
        *msaa,
        Msaa::Off,
        "NimbusCloudView requires Msaa::Off because Nimbus samples single-sample depth",
    );
}
