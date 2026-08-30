# bevy-nimbus

[![CI](https://github.com/sayhisam1/bevy-nimbus/actions/workflows/ci.yml/badge.svg)](https://github.com/sayhisam1/bevy-nimbus/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/bevy-nimbus.svg)](https://crates.io/crates/bevy-nimbus)

True-volumetric Nubis clouds for Bevy 0.19. Nimbus uses a persistent
quarter-resolution temporal raymarch, embedded 3D density textures, multiple
cloud lights, and world-space cloud shadows.

## Features

- Continuous weather fields with clear, scattered, overcast, and storm presets.
- Configurable cloud altitude, depth, coverage, scale, and movement.
- Persistent reduced-resolution temporal rendering.
- One primary and up to three secondary directional cloud lights.
- A scene-wide cloud transmittance map for shadows on world geometry.
- Native rendering and browser WebGPU support.

## Compatibility

| bevy-nimbus | Bevy | Rust | Verified targets |
|---|---|---|---|
| 0.1 | 0.19 | 1.95+ | Native WebGPU APIs and browser WebGPU |

## Quick start

```rust,no_run
use bevy::{camera::Hdr, prelude::*};
use bevy_nimbus::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, NimbusPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    let volume = commands.spawn(NimbusCloudVolume).id();

    commands.spawn((
        Camera3d::default(),
        Hdr,
        Msaa::Off,
        Transform::from_xyz(0.0, 120.0, -300.0)
            .looking_to(Vec3::new(0.12, 0.18, 1.0), Vec3::Y),
        NimbusCloudView::new(volume),
    ));
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_rotation(Quat::from_rotation_x(-0.7)),
        NimbusCloudLight::new(volume, NimbusCloudLightKind::Primary),
    ));
}
```

A `NimbusCloudVolume` is a continuous weather field, not one visible cloud.
Connect a camera and directional lights to it with explicit entity references.
Nimbus currently renders one `NimbusCloudView`.

## Crate architecture

The top-level `bevy-nimbus` crate is a small public facade. Implementation
features stay isolated in focused workspace crates:

| Crate | Responsibility |
|---|---|
| `bevy-nimbus-core` | ECS settings and the shared GPU ABI |
| `bevy-nimbus-volume` | Nubis density textures and volume model |
| `bevy-nimbus-motion` | Temporal history and reprojection |
| `bevy-nimbus-lighting` | Cloud lights and world-shadow rendering |
| `bevy-nimbus-render` | Render graph, pipelines, and temporal composition |

Applications normally depend only on `bevy-nimbus`. The internal crates keep
feature boundaries explicit and reduce coupling in the renderer.

## Focused examples

Every example demonstrates one Nimbus API family and builds for native targets
and browser WebGPU.

| Example | Demonstrates |
|---|---|
| [`basic_clouds`](examples/basic_clouds.rs) | Minimal volume, view, and primary light |
| [`cloud_settings`](examples/cloud_settings.rs) | Weather presets and field controls |
| [`sampling_quality`](examples/sampling_quality.rs) | View, light, and resolution sampling |
| [`cloud_lighting`](examples/cloud_lighting.rs) | Ambient, primary, and secondary lights |
| [`cloud_shadows`](examples/cloud_shadows.rs) | Applying the cloud shadow map to geometry |

```sh
cargo run --release --example basic_clouds
```

The deployed gallery is available at
<https://sayhisam1.github.io/bevy-nimbus/>. It requires a browser with WebGPU.

## Configuration

### `CloudSettings`

| Field | Control | Safe input |
|---|---|---|
| `base_altitude` | Volume lower bound in world units | Any finite value |
| `thickness` | Vertical volume thickness | Positive; values below `800` render as `800` |
| `movement` | Horizontal X/Z field velocity in world units/second | Any finite `Vec2` |
| `density` | Offset from preset coverage | `0..1` |
| `scale` | World-to-density-texture tiling | Positive; invalid values become `1` |

`density = 0.52` preserves the selected `CloudPreset` coverage.

### `NimbusSampling`

| Field | Default | Runtime range | Effect |
|---|---:|---:|---|
| `view_steps` | 48 | 24–64 | View-march quality; lower values become 24 |
| `light_steps` | 2 | 0, 2, or 4 | Self-shadow samples; intermediate values normalize upward |
| `resolution_divisor` | 4 | 2–8 | Raymarch resolution divisor |

### Lighting and shadows

Each Bevy `DirectionalLight` tagged with `NimbusCloudLight` affects its
referenced volume. Use `NimbusCloudLightKind::Primary` once for the world
shadow light, then `Secondary` for additional cloud lights. `with_strength`
replaces the strength derived from Bevy illuminance. `NimbusAmbientLight` is
stored on the volume.

Add `NimbusCloudShadowSource::new(volume)` to the camera that owns the
scene-wide `NimbusCloudShadowMap`. The map stores primary-light transmittance
in its R channel over an 8192-unit square centered on the world origin.

## Runtime contracts

- The embedded shape, detail, and weather textures are required. A load failure panics with the asset error.
- `NimbusCloudView` requires an HDR, single-sample camera; it adds `Hdr` and `Msaa::Off` by default.
- The cloud pass runs after the opaque main pass and before early post-processing.
- External TAA is not required; Nimbus owns its reduced-resolution temporal history.
- Invalid references and ambiguous view or shadow ownership panic with a clear error.

## Licence

Licensed under either Apache-2.0 or MIT, at your option. See `LICENSE-APACHE`
and `LICENSE-MIT`.
