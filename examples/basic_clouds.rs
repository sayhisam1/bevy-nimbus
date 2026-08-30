//! Minimal Nimbus setup: one cloud field, one camera, and one sun.

mod common;

use bevy::prelude::*;
use bevy_nimbus::NimbusCloudVolume;

fn main() {
    let mut app = common::app("bevy-nimbus — basic clouds");
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
    common::spawn_reference_ground(&mut commands, &mut meshes, &mut materials);
}
