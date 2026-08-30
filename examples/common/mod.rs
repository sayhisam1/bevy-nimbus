#![allow(dead_code)]

use bevy::{
    camera::Hdr,
    core_pipeline::{prepass::DepthPrepass, tonemapping::Tonemapping},
    prelude::*,
    window::WindowResolution,
};
use bevy_nimbus::{NimbusCloudLight, NimbusCloudLightKind, NimbusCloudView};

pub fn app(title: &'static str) -> App {
    app_with_clear(title, Color::srgb(0.22, 0.48, 0.82))
}

pub fn app_with_clear(title: &'static str, clear_color: Color) -> App {
    let mut app = App::new();
    app.insert_resource(ClearColor(clear_color)).add_plugins((
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
    spawn_view_at(
        commands,
        volume,
        Transform::from_xyz(0.0, 450.0, -700.0)
            .looking_at(Vec3::new(100.0, 720.0, 1_500.0), Vec3::Y),
    )
}

pub fn spawn_view_at(commands: &mut Commands, volume: Entity, transform: Transform) -> Entity {
    commands
        .spawn((
            Camera3d::default(),
            Hdr,
            Msaa::Off,
            DepthPrepass,
            Tonemapping::TonyMcMapface,
            transform,
            NimbusCloudView::new(volume),
        ))
        .id()
}

pub fn spawn_primary_light(commands: &mut Commands, volume: Entity) -> Entity {
    spawn_primary_light_with(
        commands,
        volume,
        Vec3::new(0.45, 0.78, 0.43).normalize(),
        Color::WHITE,
        18_000.0,
    )
}

pub fn spawn_primary_light_with(
    commands: &mut Commands,
    volume: Entity,
    direction_to_light: Vec3,
    color: Color,
    illuminance: f32,
) -> Entity {
    commands
        .spawn((
            DirectionalLight {
                color,
                illuminance,
                ..default()
            },
            Transform::from_rotation(Quat::from_rotation_arc(
                Vec3::NEG_Z,
                -direction_to_light.normalize(),
            )),
            NimbusCloudLight::new(volume, NimbusCloudLightKind::Primary),
        ))
        .id()
}

pub fn material(
    materials: &mut Assets<StandardMaterial>,
    color: Color,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: color,
        perceptual_roughness: 0.9,
        ..default()
    })
}

pub fn spawn_ground(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: Handle<StandardMaterial>,
    size: f32,
) {
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(size, size))),
        MeshMaterial3d(material),
    ));
}

pub fn spawn_reference_ground(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let ground = material(materials, Color::srgb(0.22, 0.32, 0.18));
    spawn_ground(commands, meshes, ground, 4_000.0);
}
