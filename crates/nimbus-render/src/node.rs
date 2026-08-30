//! Main-world extraction and render-graph installation.

use crate::{pipeline::init_pipelines, render};
use bevy::{
    core_pipeline::{Core3dSystems, schedule::Core3d},
    prelude::*,
    render::{Render, RenderApp, RenderStartup, RenderSystems, render_resource::TextureUsages},
    transform::TransformSystems,
};
use nimbus_core::{
    CloudPreset, CloudSettings, CloudsUniform, CloudsUniformBuffer, NimbusCloudView,
    NimbusCloudVolume, NimbusSampling,
};
use nimbus_lighting::{
    NimbusAmbientLight, NimbusCloudLight, NimbusCloudShadowSource, resolve as resolve_lighting,
};

pub(crate) fn install(app: &mut App) {
    app.add_systems(PostUpdate, sync_clouds.after(TransformSystems::Propagate));
    let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
        return;
    };
    render_app
        .init_resource::<CloudsUniformBuffer>()
        .add_systems(RenderStartup, init_pipelines)
        .add_systems(
            Render,
            render::prepare_uniforms.in_set(RenderSystems::PrepareResources),
        )
        .add_systems(
            Core3d,
            render::render_volume_clouds
                .after(Core3dSystems::MainPass)
                .before(Core3dSystems::EarlyPostProcess),
        );
}

fn sync_clouds(
    mut uniform: ResMut<CloudsUniform>,
    views: Query<(&NimbusCloudView, &GlobalTransform, &Projection), With<Camera>>,
    volumes: Query<
        (
            &CloudSettings,
            &CloudPreset,
            &NimbusSampling,
            &NimbusAmbientLight,
        ),
        With<NimbusCloudVolume>,
    >,
    shadow_sources: Query<&NimbusCloudShadowSource>,
    mut camera_3ds: Query<&mut Camera3d, With<NimbusCloudView>>,
    cloud_lights: Query<(
        Entity,
        &NimbusCloudLight,
        &DirectionalLight,
        &GlobalTransform,
        &InheritedVisibility,
    )>,
    frame: Res<Time>,
) {
    for mut camera in &mut camera_3ds {
        let raw: TextureUsages = camera.depth_texture_usages.into();
        if !raw.contains(TextureUsages::TEXTURE_BINDING) {
            camera.depth_texture_usages = (raw | TextureUsages::TEXTURE_BINDING).into();
        }
    }

    let mut views = views.iter();
    let Some((view, transform, projection)) = views.next() else {
        return;
    };
    assert!(
        views.next().is_none(),
        "Nimbus currently supports one NimbusCloudView camera"
    );
    for source in &shadow_sources {
        assert_eq!(
            source.volume, view.volume,
            "NimbusCloudView and NimbusCloudShadowSource must reference the same volume"
        );
    }
    let (settings, preset, sampling, ambient) = volumes.get(view.volume).unwrap_or_else(|error| {
        panic!(
            "NimbusCloudView references invalid cloud volume {:?}: {error}",
            view.volume
        )
    });
    let profile = preset.profile();
    let base_altitude = finite_or(settings.base_altitude, 1500.0);
    let thickness = positive_or(settings.thickness, 1000.0).max(800.0);
    let resolved = resolve_lighting(
        view.volume,
        ambient,
        base_altitude,
        thickness,
        &cloud_lights,
    );
    let forward = transform.forward();
    let right = transform.right();
    let up = transform.up();
    let (tan_half, aspect) = match projection {
        Projection::Perspective(perspective) => {
            ((0.5 * perspective.fov).tan(), perspective.aspect_ratio)
        }
        _ => (0.577_350_26, 1.0),
    };
    *uniform = CloudsUniform {
        camera_pos: transform.translation(),
        elapsed_seconds: frame.elapsed_secs().rem_euclid(8192.0),
        camera_forward: forward.as_vec3(),
        directional_light_count: resolved.directional_light_count,
        camera_right: right.as_vec3() * tan_half * aspect,
        ambient_strength: resolved.ambient_strength,
        camera_up: up.as_vec3() * tan_half,
        density: (profile.density + settings.density.clamp(0.0, 1.0) - 0.52).clamp(0.0, 1.0),
        ambient_color: resolved.ambient_color,
        scale: positive_or(settings.scale, 1.0),
        movement: if settings.movement.is_finite() {
            settings.movement
        } else {
            Vec2::ZERO
        },
        base_altitude,
        thickness,
        base_color: profile.base_color,
        view_steps: sampling.view_steps.clamp(24, 64),
        directional_lights: resolved.directional_lights,
        light_steps: match sampling.light_steps {
            0 => 0,
            1..=3 => 2,
            _ => 4,
        },
        resolution_divisor: sampling.resolution_divisor.clamp(2, 8),
    };
}

fn finite_or(value: f32, fallback: f32) -> f32 {
    if value.is_finite() { value } else { fallback }
}

fn positive_or(value: f32, fallback: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        fallback
    }
}
