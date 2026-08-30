//! Minimal Nimbus setup: one cloud field, one camera, and one sun.

mod common;

use bevy::prelude::*;
use bevy_nimbus::NimbusCloudVolume;

fn main() {
    let mut app = common::app_with_clear(
        "bevy-nimbus — basic coastal clouds",
        Color::srgb(0.08, 0.38, 0.68),
    );
    app.add_systems(Startup, setup).run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let volume = commands.spawn(NimbusCloudVolume).id();
    common::spawn_view(&mut commands, volume);
    common::spawn_primary_light(&mut commands, volume);

    let water = common::material(&mut materials, Color::srgb(0.025, 0.16, 0.24));
    common::spawn_ground(&mut commands, &mut meshes, water, 6_000.0);

    let rock_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let rock = common::material(&mut materials, Color::srgb(0.32, 0.27, 0.22));
    for (translation, scale, rotation) in [
        (
            Vec3::new(-220.0, 95.0, 420.0),
            Vec3::new(130.0, 190.0, 110.0),
            -0.18,
        ),
        (
            Vec3::new(130.0, 115.0, 500.0),
            Vec3::new(105.0, 230.0, 100.0),
            0.16,
        ),
        (
            Vec3::new(-45.0, 225.0, 470.0),
            Vec3::new(260.0, 55.0, 90.0),
            -0.04,
        ),
        (
            Vec3::new(470.0, 45.0, 760.0),
            Vec3::new(140.0, 90.0, 180.0),
            0.28,
        ),
        (
            Vec3::new(-620.0, 30.0, 900.0),
            Vec3::new(190.0, 60.0, 150.0),
            -0.22,
        ),
    ] {
        commands.spawn((
            Mesh3d(rock_mesh.clone()),
            MeshMaterial3d(rock.clone()),
            Transform::from_translation(translation)
                .with_scale(scale)
                .with_rotation(Quat::from_rotation_z(rotation)),
        ));
    }
}
