//! Cloud lighting with one warm primary sun, one cool secondary light, and ambient sky light.

mod common;

use bevy::prelude::*;
use bevy_nimbus::{NimbusAmbientLight, NimbusCloudLight, NimbusCloudLightKind, NimbusCloudVolume};

fn main() {
    let mut app = common::app("bevy-nimbus — cloud lighting");
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
            NimbusAmbientLight {
                strength: 0.45,
                color: Color::srgb(0.35, 0.55, 1.0),
            },
        ))
        .id();
    common::spawn_view(&mut commands, volume);
    common::spawn_reference_ground(&mut commands, &mut meshes, &mut materials);

    commands.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 0.62, 0.35),
            illuminance: 14_000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.3, -0.8, 0.0)),
        NimbusCloudLight::new(volume, NimbusCloudLightKind::Primary).with_strength(1.2),
    ));
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(0.35, 0.5, 1.0),
            illuminance: 2_000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.2, 2.2, 0.0)),
        NimbusCloudLight::new(volume, NimbusCloudLightKind::Secondary).with_strength(0.2),
    ));
}
