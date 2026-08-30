//! Scene-wide cloud shadows projected onto a ground material.

mod common;

use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderType},
    shader::ShaderRef,
};
use bevy_nimbus::{NimbusCloudShadowMap, NimbusCloudShadowSource, NimbusCloudVolume};

type ShadowGroundMaterial = ExtendedMaterial<StandardMaterial, ShadowGround>;

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
struct ShadowGround {
    #[texture(100)]
    #[sampler(101)]
    shadow: Handle<Image>,
    #[uniform(102)]
    params: ShadowGroundParams,
}

#[derive(ShaderType, Debug, Clone)]
struct ShadowGroundParams {
    sun_strength: Vec4,
}

impl MaterialExtension for ShadowGround {
    fn fragment_shader() -> ShaderRef {
        "examples/shadow_ground.wgsl".into()
    }
}

fn main() {
    let mut app = common::app("bevy-nimbus — cloud shadows");
    app.add_plugins(MaterialPlugin::<ShadowGroundMaterial>::default())
        .add_systems(PostStartup, setup)
        .run();
}

#[expect(clippy::needless_pass_by_value, reason = "Bevy system parameters")]
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ShadowGroundMaterial>>,
    shadow_map: Res<NimbusCloudShadowMap>,
) {
    let volume = commands.spawn(NimbusCloudVolume).id();
    let camera = common::spawn_view(&mut commands, volume);
    commands
        .entity(camera)
        .insert(NimbusCloudShadowSource::new(volume));
    common::spawn_primary_light(&mut commands, volume);

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(4_000.0, 4_000.0))),
        MeshMaterial3d(materials.add(ExtendedMaterial {
            base: StandardMaterial {
                base_color: Color::srgb(0.28, 0.38, 0.22),
                perceptual_roughness: 0.95,
                ..default()
            },
            extension: ShadowGround {
                shadow: shadow_map.texture.clone(),
                params: ShadowGroundParams {
                    sun_strength: Vec3::new(0.5, 0.65, 0.55).normalize().extend(0.82),
                },
            },
        })),
    ));
}
