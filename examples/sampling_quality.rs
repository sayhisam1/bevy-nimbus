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
    let mut app = common::app("bevy-nimbus — sampling quality (Space changes quality)");
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
    common::spawn_view(&mut commands, volume);
    common::spawn_primary_light(&mut commands, volume);
    common::spawn_reference_ground(&mut commands, &mut meshes, &mut materials);
}

#[expect(clippy::needless_pass_by_value, reason = "Bevy system parameters")]
fn cycle_quality(
    keys: Res<ButtonInput<KeyCode>>,
    mut quality_index: Local<usize>,
    mut sampling: Query<&mut NimbusSampling>,
) {
    if !keys.just_pressed(KeyCode::Space) {
        return;
    }
    *quality_index = (*quality_index + 1) % QUALITY_LEVELS.len();
    for mut settings in &mut sampling {
        *settings = QUALITY_LEVELS[*quality_index];
        info!("Nimbus sampling: {:?}", *settings);
    }
}
