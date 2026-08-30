//! Cloud lighting with one warm primary sun, one cool secondary light, and ambient sky light.

mod common;

use bevy::prelude::*;
use bevy_nimbus::{
    CloudPreset, CloudSettings, NimbusAmbientLight, NimbusCloudLight, NimbusCloudLightKind,
    NimbusCloudVolume,
};

fn main() {
    let mut app = common::app_with_clear(
        "bevy-nimbus — sunset cloud lighting",
        Color::srgb(0.28, 0.07, 0.055),
    );
    app.add_systems(Startup, setup).run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let volume = commands
        .spawn((
            NimbusCloudVolume,
            CloudPreset::Scattered,
            CloudSettings {
                density: 0.72,
                thickness: 1_700.0,
                movement: Vec2::new(12.0, 3.0),
                ..default()
            },
            NimbusAmbientLight {
                strength: 0.45,
                color: Color::srgb(0.35, 0.48, 1.0),
            },
        ))
        .id();
    common::spawn_view_at(
        &mut commands,
        volume,
        Transform::from_xyz(-260.0, 120.0, -560.0)
            .looking_at(Vec3::new(90.0, 700.0, 1_500.0), Vec3::Y),
    );

    let primary_direction = Vec3::new(0.58, 0.18, 0.79).normalize();
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 0.34, 0.10),
            illuminance: 14_000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_arc(Vec3::NEG_Z, -primary_direction)),
        NimbusCloudLight::new(volume, NimbusCloudLightKind::Primary).with_strength(1.2),
    ));
    let secondary_direction = Vec3::new(-0.72, 0.30, -0.62).normalize();
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(0.28, 0.42, 1.0),
            illuminance: 2_000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_arc(Vec3::NEG_Z, -secondary_direction)),
        NimbusCloudLight::new(volume, NimbusCloudLightKind::Secondary).with_strength(0.2),
    ));

    let sand = common::material(&mut materials, Color::srgb(0.42, 0.20, 0.07));
    common::spawn_ground(&mut commands, &mut meshes, sand, 5_000.0);
    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let stone = common::material(&mut materials, Color::srgb(0.48, 0.25, 0.12));
    let basalt = common::material(&mut materials, Color::srgb(0.025, 0.035, 0.055));
    for z in [180.0, 440.0] {
        for x in [-240.0, -90.0, 60.0, 210.0, 360.0] {
            commands.spawn((
                Mesh3d(cube.clone()),
                MeshMaterial3d(stone.clone()),
                Transform::from_xyz(x, 125.0, z).with_scale(Vec3::new(42.0, 250.0, 42.0)),
            ));
        }
    }
    commands.spawn((
        Mesh3d(cube.clone()),
        MeshMaterial3d(stone),
        Transform::from_xyz(60.0, 270.0, 310.0).with_scale(Vec3::new(720.0, 38.0, 55.0)),
    ));
    commands.spawn((
        Mesh3d(cube),
        MeshMaterial3d(basalt),
        Transform::from_xyz(50.0, 4.0, 40.0).with_scale(Vec3::new(260.0, 8.0, 700.0)),
    ));
}
