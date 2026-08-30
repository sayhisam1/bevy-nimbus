//! Cloud weather-field controls. Press Space to cycle through cloud presets.

mod common;

use bevy::prelude::*;
use bevy_nimbus::{CloudPreset, CloudSettings, NimbusCloudVolume};

fn main() {
    let mut app = common::app("bevy-nimbus — cloud settings (Space changes preset)");
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
            CloudPreset::Scattered,
            CloudSettings {
                base_altitude: 1_500.0,
                thickness: 1_200.0,
                movement: Vec2::new(18.0, 5.0),
                density: 0.52,
                scale: 1.0,
            },
        ))
        .id();
    common::spawn_view(&mut commands, volume);
    common::spawn_primary_light(&mut commands, volume);
    common::spawn_reference_ground(&mut commands, &mut meshes, &mut materials);
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
