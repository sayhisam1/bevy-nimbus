//! Cloud weather-field controls. Press Space to cycle through cloud presets.

mod common;

use bevy::prelude::*;
use bevy_nimbus::{CloudPreset, CloudSettings, NimbusCloudVolume};

fn main() {
    let mut app = common::app_with_clear(
        "bevy-nimbus — forecast station (Space changes preset)",
        Color::srgb(0.36, 0.48, 0.55),
    );
    app.add_systems(Startup, setup)
        .add_systems(Update, cycle_preset)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let volume = commands
        .spawn((
            NimbusCloudVolume,
            CloudPreset::Overcast,
            CloudSettings {
                base_altitude: 1_250.0,
                thickness: 1_500.0,
                movement: Vec2::new(24.0, 6.0),
                density: 0.52,
                scale: 0.8,
            },
        ))
        .id();
    common::spawn_view_at(
        &mut commands,
        volume,
        Transform::from_xyz(-420.0, 190.0, -520.0)
            .looking_at(Vec3::new(20.0, 410.0, 900.0), Vec3::Y),
    );
    common::spawn_primary_light_with(
        &mut commands,
        volume,
        Vec3::new(-0.35, 0.88, 0.32).normalize(),
        Color::srgb(1.0, 0.96, 0.88),
        16_000.0,
    );

    let prairie = common::material(&mut materials, Color::srgb(0.42, 0.30, 0.13));
    common::spawn_ground(&mut commands, &mut meshes, prairie, 5_000.0);
    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let white = common::material(&mut materials, Color::srgb(0.82, 0.82, 0.74));
    let red = common::material(&mut materials, Color::srgb(0.62, 0.06, 0.035));
    let dark = common::material(&mut materials, Color::srgb(0.08, 0.09, 0.09));
    for (translation, scale, material) in [
        (
            Vec3::new(0.0, 35.0, 180.0),
            Vec3::new(110.0, 70.0, 90.0),
            white.clone(),
        ),
        (
            Vec3::new(0.0, 82.0, 180.0),
            Vec3::new(125.0, 24.0, 105.0),
            dark.clone(),
        ),
        (
            Vec3::new(145.0, 115.0, 220.0),
            Vec3::new(9.0, 230.0, 9.0),
            red.clone(),
        ),
        (
            Vec3::new(145.0, 205.0, 220.0),
            Vec3::new(75.0, 8.0, 8.0),
            white.clone(),
        ),
        (
            Vec3::new(-180.0, 26.0, 250.0),
            Vec3::new(22.0, 52.0, 22.0),
            dark.clone(),
        ),
        (
            Vec3::new(-115.0, 38.0, 270.0),
            Vec3::new(22.0, 76.0, 22.0),
            red.clone(),
        ),
        (
            Vec3::new(-50.0, 50.0, 290.0),
            Vec3::new(22.0, 100.0, 22.0),
            white.clone(),
        ),
    ] {
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(material),
            Transform::from_translation(translation).with_scale(scale),
        ));
    }
}

#[expect(clippy::needless_pass_by_value, reason = "Bevy system parameters")]
fn cycle_preset(keys: Res<ButtonInput<KeyCode>>, mut presets: Query<&mut CloudPreset>) {
    if !keys.just_pressed(KeyCode::Space) {
        return;
    }
    for mut preset in &mut presets {
        *preset = match *preset {
            CloudPreset::Clear => CloudPreset::Scattered,
            CloudPreset::Scattered => CloudPreset::Overcast,
            CloudPreset::Overcast => CloudPreset::Storm,
            CloudPreset::Storm => CloudPreset::Clear,
        };
        info!("Cloud preset: {:?}", *preset);
    }
}
