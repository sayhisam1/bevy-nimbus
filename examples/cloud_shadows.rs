//! World-space cloud shadows on a ground plane, raised panels, and buildings.

mod common;

use bevy::{
    asset::embedded_asset,
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderType},
    shader::ShaderRef,
};
use bevy_nimbus::{
    CloudPreset, CloudSettings, NimbusCloudShadowMap, NimbusCloudShadowSource, NimbusCloudVolume,
};

const SUN_DIRECTION_TO_LIGHT: Vec3 = Vec3::new(0.48, 0.76, 0.43);
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
    direction_amount: Vec4,
}

impl MaterialExtension for ShadowGround {
    fn fragment_shader() -> ShaderRef {
        "embedded://cloud_shadows/shadow_ground.wgsl".into()
    }
}

fn main() {
    let mut app = common::app_with_clear(
        "bevy-nimbus — world-space cloud shadows",
        Color::srgb(0.24, 0.55, 0.76),
    );
    embedded_asset!(app, "examples", "shadow_ground.wgsl");
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
    let volume = commands
        .spawn((
            NimbusCloudVolume,
            CloudPreset::Scattered,
            CloudSettings {
                density: 0.64,
                scale: 0.68,
                movement: Vec2::new(35.0, 12.0),
                ..default()
            },
        ))
        .id();
    let camera = common::spawn_view_at(
        &mut commands,
        volume,
        Transform::from_xyz(-650.0, 720.0, -1_450.0)
            .looking_at(Vec3::new(100.0, 440.0, 650.0), Vec3::Y),
    );
    commands
        .entity(camera)
        .insert(NimbusCloudShadowSource::new(volume));
    common::spawn_primary_light_with(
        &mut commands,
        volume,
        SUN_DIRECTION_TO_LIGHT.normalize(),
        Color::srgb(1.0, 0.95, 0.80),
        25_000.0,
    );

    let tan = shadow_material(&mut materials, &shadow_map, Color::srgb(0.58, 0.48, 0.30));
    let white = shadow_material(&mut materials, &shadow_map, Color::srgb(0.78, 0.76, 0.67));
    let dark = shadow_material(&mut materials, &shadow_map, Color::srgb(0.035, 0.10, 0.14));
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(4_000.0, 4_000.0))),
        MeshMaterial3d(tan),
    ));

    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    // Bright service roads make shadow motion and scale easy to read.
    for z in [-850.0, -250.0, 350.0, 950.0] {
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(white.clone()),
            Transform::from_xyz(0.0, 2.0, z).with_scale(Vec3::new(3_100.0, 4.0, 26.0)),
        ));
    }
    // Raised panel rows prove that sampling is world-space rather than screen-space.
    for z in [-700.0, -440.0, -180.0, 80.0, 340.0, 600.0, 860.0] {
        for x in [-1_050.0, -720.0, -390.0, -60.0, 270.0, 600.0, 930.0] {
            commands.spawn((
                Mesh3d(cube.clone()),
                MeshMaterial3d(dark.clone()),
                Transform::from_xyz(x, 28.0, z)
                    .with_scale(Vec3::new(230.0, 18.0, 105.0))
                    .with_rotation(Quat::from_rotation_x(-0.18)),
            ));
        }
    }
    for (position, scale) in [
        (
            Vec3::new(-620.0, 95.0, 70.0),
            Vec3::new(230.0, 190.0, 250.0),
        ),
        (
            Vec3::new(620.0, 150.0, 650.0),
            Vec3::new(320.0, 300.0, 300.0),
        ),
        (
            Vec3::new(950.0, 65.0, -420.0),
            Vec3::new(170.0, 130.0, 220.0),
        ),
    ] {
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(white.clone()),
            Transform::from_translation(position).with_scale(scale),
        ));
    }
}

fn shadow_material(
    materials: &mut Assets<ShadowGroundMaterial>,
    shadow_map: &NimbusCloudShadowMap,
    color: Color,
) -> Handle<ShadowGroundMaterial> {
    materials.add(ExtendedMaterial {
        base: StandardMaterial {
            base_color: color,
            perceptual_roughness: 0.95,
            unlit: true,
            ..default()
        },
        extension: ShadowGround {
            shadow: shadow_map.texture.clone(),
            params: ShadowGroundParams {
                direction_amount: SUN_DIRECTION_TO_LIGHT.normalize().extend(0.75),
            },
        },
    })
}
