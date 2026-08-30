#![allow(dead_code)]

use bevy::{
    camera::Hdr,
    core_pipeline::{prepass::DepthPrepass, tonemapping::Tonemapping},
    prelude::*,
    window::WindowResolution,
};
use bevy_nimbus::{NimbusCloudLight, NimbusCloudLightKind, NimbusCloudView};

pub fn app(title: &'static str) -> App {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.22, 0.48, 0.82)))
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: title.into(),
                    resolution: WindowResolution::new(1280, 720),
                    ..default()
                }),
                ..default()
            }),
            bevy_nimbus::NimbusPlugin,
        ));
    app
}

pub fn spawn_view(commands: &mut Commands, volume: Entity) -> Entity {
    commands
        .spawn((
            Camera3d::default(),
            Hdr,
            Msaa::Off,
            DepthPrepass,
            Tonemapping::TonyMcMapface,
            Transform::from_xyz(0.0, 120.0, -300.0).looking_to(Vec3::new(0.12, 0.18, 1.0), Vec3::Y),
            NimbusCloudView::new(volume),
        ))
        .id()
}

pub fn spawn_primary_light(commands: &mut Commands, volume: Entity) {
    commands.spawn((
        DirectionalLight {
            illuminance: 18_000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.65, -0.8, 0.0)),
        NimbusCloudLight::new(volume, NimbusCloudLightKind::Primary),
    ));
}

pub fn spawn_reference_ground(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(4_000.0, 4_000.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.22, 0.32, 0.18),
            perceptual_roughness: 0.95,
            ..default()
        })),
    ));
}
