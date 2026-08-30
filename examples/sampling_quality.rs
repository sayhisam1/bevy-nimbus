//! Nimbus sampling quality. Press Space to cycle low, default, and high quality.

mod common;

use bevy::prelude::*;
use bevy_nimbus::{NimbusCloudVolume, NimbusSampling};

const QUALITY_LEVELS: [NimbusSampling; 3] = [
    NimbusSampling {
        view_steps: 24,
        light_steps: 0,
        resolution_divisor: 8,
    },
    NimbusSampling {
        view_steps: 48,
        light_steps: 2,
        resolution_divisor: 4,
    },
    NimbusSampling {
        view_steps: 64,
        light_steps: 4,
        resolution_divisor: 2,
    },
];

fn main() {
    let mut app = common::app_with_clear(
        "bevy-nimbus — observatory quality lab (Space changes quality)",
        Color::srgb(0.045, 0.025, 0.12),
    );
    app.add_systems(Startup, setup)
        .add_systems(Update, cycle_quality)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let volume = commands
        .spawn((NimbusCloudVolume, NimbusSampling::default()))
        .id();
    common::spawn_view_at(
        &mut commands,
        volume,
        Transform::from_xyz(0.0, 340.0, -620.0).looking_at(Vec3::new(0.0, 760.0, 1_500.0), Vec3::Y),
    );
    common::spawn_primary_light_with(
        &mut commands,
        volume,
        Vec3::new(-0.55, 0.35, 0.76).normalize(),
        Color::srgb(0.48, 0.62, 1.0),
        5_000.0,
    );

    let ridge = common::material(&mut materials, Color::srgb(0.035, 0.028, 0.055));
    common::spawn_ground(&mut commands, &mut meshes, ridge.clone(), 4_000.0);
    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let observatory = common::material(&mut materials, Color::srgb(0.16, 0.18, 0.22));
    let marker = common::material(&mut materials, Color::srgb(0.08, 0.65, 0.85));
    for (translation, scale, material, rotation) in [
        (
            Vec3::new(-430.0, 90.0, 360.0),
            Vec3::new(700.0, 180.0, 260.0),
            ridge.clone(),
            -0.12,
        ),
        (
            Vec3::new(420.0, 65.0, 450.0),
            Vec3::new(620.0, 130.0, 300.0),
            ridge.clone(),
            0.16,
        ),
        (
            Vec3::new(40.0, 75.0, 240.0),
            Vec3::new(150.0, 150.0, 150.0),
            observatory.clone(),
            0.0,
        ),
        (
            Vec3::new(40.0, 170.0, 240.0),
            Vec3::new(190.0, 40.0, 190.0),
            observatory.clone(),
            0.0,
        ),
        (
            Vec3::new(190.0, 155.0, 260.0),
            Vec3::new(8.0, 210.0, 8.0),
            observatory.clone(),
            0.0,
        ),
        (
            Vec3::new(190.0, 252.0, 260.0),
            Vec3::new(95.0, 7.0, 7.0),
            marker.clone(),
            0.0,
        ),
    ] {
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(material),
            Transform::from_translation(translation)
                .with_scale(scale)
                .with_rotation(Quat::from_rotation_z(rotation)),
        ));
    }
}

#[expect(clippy::needless_pass_by_value, reason = "Bevy system parameters")]
fn cycle_quality(keys: Res<ButtonInput<KeyCode>>, mut sampling: Query<&mut NimbusSampling>) {
    if !keys.just_pressed(KeyCode::Space) {
        return;
    }
    for mut settings in &mut sampling {
        let current = QUALITY_LEVELS.iter().position(|level| *level == *settings);
        let next = (current.unwrap_or(QUALITY_LEVELS.len() - 1) + 1) % QUALITY_LEVELS.len();
        *settings = QUALITY_LEVELS[next];
        info!("Nimbus sampling: {:?}", *settings);
    }
}
