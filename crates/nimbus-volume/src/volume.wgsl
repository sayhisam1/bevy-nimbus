#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(5) var scene_depth: texture_depth_2d;

@group(0) @binding(6) var<uniform> temporal: NimbusMotionUniform;
@group(0) @binding(7) var history_color: texture_2d<f32>;
@group(0) @binding(8) var history_guide: texture_2d<f32>;
@group(0) @binding(9) var history_sampler: sampler;

struct VolumeOutput {
    @location(0) color: vec4<f32>,
    @location(1) guide: vec2<f32>,
}

fn ray_slab(origin: vec3<f32>, direction: vec3<f32>, bottom: f32, top: f32) -> vec2<f32> {
    if (abs(direction.y) < 0.0001) {
        return vec2<f32>(1.0, 0.0);
    }
    let first = (bottom - origin.y) / direction.y;
    let second = (top - origin.y) / direction.y;
    let near = max(min(first, second), 0.0);
    let far = min(max(first, second), 32000.0);
    return vec2<f32>(near, max(far - near, 0.0));
}

fn hash_u32(value: u32) -> u32 {
    var hash = value;
    hash = (hash ^ 61u) ^ (hash >> 16u);
    hash *= 9u;
    hash ^= hash >> 4u;
    hash *= 0x27d4eb2du;
    return hash ^ (hash >> 15u);
}

fn hash_jitter(pixel: vec2<f32>) -> f32 {
    let coordinate = vec2<u32>(pixel);
    let seed = coordinate.x * 0x9e3779b9u
        ^ coordinate.y * 0x85ebca6bu
        ^ temporal.frame * 0xc2b2ae35u;
    return f32(hash_u32(seed) & 0x00ffffffu) / 16777216.0;
}

fn light_transmittance(
    position: vec3<f32>,
    bottom: f32,
    top: f32,
    direction_to_light: vec3<f32>,
) -> f32 {
    if (clouds.light_steps == 0u) {
        return 1.0;
    }
    let light_direction = normalize(direction_to_light);
    if (light_direction.y <= 0.001) {
        return 1.0;
    }
    let light_distance = min((top - position.y) / light_direction.y, 8000.0);
    let light_steps = select(2u, 4u, clouds.light_steps >= 4u);
    let segment_length = light_distance / f32(light_steps);
    var optical_depth = 0.0;
    let four_step_fractions = array<f32, 4>(0.05, 0.15, 0.35, 0.70);
    for (var index = 0u; index < 4u; index++) {
        if (index >= light_steps) {
            break;
        }
        var fraction = select(0.20, 0.65, index == 1u);
        if (light_steps == 4u) {
            fraction = four_step_fractions[index];
        }
        let sample_position = position + light_direction * light_distance * fraction;
        optical_depth += density_at(sample_position, bottom, top, segment_length)
            * segment_length;
    }
    return exp(-optical_depth * EXTINCTION * 1.1);
}

fn resolve_history(
    current: vec4<f32>,
    current_guide: vec2<f32>,
    direction: vec3<f32>,
    fine_step: f32,
    current_uv: vec2<f32>,
) -> vec4<f32> {
    if (temporal.history_valid == 0u || current.a <= 0.001 || current_guide.x <= 0.0) {
        return current;
    }
    let current_world = clouds.camera_pos + direction * current_guide.x;
    let reprojected = nimbus_reproject_history(current_world, current_uv);
    let uv = reprojected.xy;
    let motion = reprojected.z;
    if (
        any(uv <= vec2<f32>(0.001))
        || any(uv >= vec2<f32>(0.999))
        || motion > 0.25
    ) {
        return current;
    }
    let elapsed = max(clouds.elapsed_seconds - temporal.previous_elapsed_seconds, 0.0);
    let previous_world = current_world
        + vec3<f32>(clouds.movement.x, 0.0, clouds.movement.y) * elapsed;
    let delta = previous_world - temporal.previous_camera_pos;
    let dimensions = vec2<i32>(textureDimensions(history_guide));
    let guide_texel = clamp(
        vec2<i32>(uv * vec2<f32>(dimensions)),
        vec2<i32>(0),
        dimensions - vec2<i32>(1),
    );
    let previous_guide = textureLoad(history_guide, guide_texel, 0).rg;
    let previous = textureSampleLevel(history_color, history_sampler, uv, 0.0);
    let expected_distance = length(delta);
    let rejection_distance = max(
        80.0 + 4.0 * (previous_guide.y + current_guide.y),
        max(3.0 * fine_step, 0.10 * expected_distance),
    );
    let alpha_distance = abs(previous.a - current.a);
    // At 48 view steps, alpha is stable enough to reject stale edge history more precisely.
    if (
        previous.a <= 0.001
        || abs(previous_guide.x - expected_distance) > rejection_distance
        || alpha_distance > max(0.16, 0.55 * current.a)
    ) {
        return current;
    }
    let history_weight = min(
        temporal.history_blend,
        clamp(0.90 - 2.0 * motion, 0.60, 0.90),
    );
    return mix(current, previous, history_weight);
}

@fragment
fn fs_main(in: FullscreenVertexOutput) -> VolumeOutput {
    let depth_size = vec2<f32>(textureDimensions(scene_depth));
    if (textureLoad(scene_depth, vec2<i32>(in.uv * depth_size), 0) > 0.0) {
        return VolumeOutput(vec4<f32>(0.0), vec2<f32>(0.0));
    }
    let direction = normalize(
        clouds.camera_forward + clouds.camera_right * (in.uv.x * 2.0 - 1.0)
            + clouds.camera_up * (1.0 - 2.0 * in.uv.y)
    );
    let bottom = clouds.base_altitude;
    let top = bottom + max(clouds.thickness, 800.0);
    let hit = ray_slab(clouds.camera_pos, direction, bottom, top);
    if (hit.y <= 0.0) {
        return VolumeOutput(vec4<f32>(0.0), vec2<f32>(0.0));
    }
    let step_count = clamp(clouds.view_steps, 24u, 64u);
    let slab_thickness = max(top - bottom, 1.0);
    let step_length = hit.y / f32(step_count);
    var distance = hit.x + step_length * hash_jitter(in.uv * depth_size);
    let far = hit.x + hit.y;
    var transmittance = 1.0;
    var radiance = vec3<f32>(0.0);
    var depth_weight = 0.0;
    var weighted_depth = 0.0;
    var weighted_depth_squared = 0.0;
    var cached_light_visibility = array<f32, 4>(1.0, 1.0, 1.0, 1.0);
    let ambient_light_vertical = select(
        1.0,
        clouds.directional_lights[0].direction_to_light.y,
        clouds.directional_light_count > 0u,
    );
    var dense_sample_index = 0u;
    for (var iteration = 0u; iteration < 64u; iteration++) {
        if (iteration >= step_count || distance >= far || transmittance <= 0.01) {
            break;
        }
        let position = clouds.camera_pos + direction * distance;
        let density = density_at(position, bottom, top, step_length);
        if (density > 0.001) {
            if ((dense_sample_index & 1u) == 0u) {
                for (var light_index = 0u; light_index < clouds.directional_light_count; light_index++) {
                    cached_light_visibility[light_index] = light_transmittance(
                        position,
                        bottom,
                        top,
                        clouds.directional_lights[light_index].direction_to_light,
                    );
                }
            }
            dense_sample_index += 1u;
            let beer = exp(-density * step_length * EXTINCTION);
            let powder = 1.0 - exp(-density * step_length * EXTINCTION * 2.0);
            let edge_attenuation = 1.0 - 0.18 * (1.0 - beer) * (1.0 - powder);
            let height_fraction = (position.y - bottom) / slab_thickness;
            var lighting = ambient_cloud_radiance(
                clouds.ambient_color,
                clouds.ambient_strength,
                height_fraction,
                ambient_light_vertical,
            );
            for (var light_index = 0u; light_index < clouds.directional_light_count; light_index++) {
                lighting += directional_cloud_radiance(
                    direction,
                    clouds.directional_lights[light_index],
                    cached_light_visibility[light_index],
                );
            }
            let alpha = 1.0 - beer;
            let contribution = transmittance * alpha;
            depth_weight += contribution;
            weighted_depth += contribution * distance;
            weighted_depth_squared += contribution * distance * distance;
            radiance += contribution * edge_attenuation * lighting * clouds.base_color;
            transmittance *= beer;
        }
        distance += step_length;
    }
    let alpha = 1.0 - transmittance;
    var guide = vec2<f32>(0.0);
    if (depth_weight > 0.0001) {
        let mean_depth = weighted_depth / depth_weight;
        let variance = max(weighted_depth_squared / depth_weight - mean_depth * mean_depth, 0.0);
        guide = vec2<f32>(mean_depth, sqrt(variance));
    }
    let current = vec4<f32>(radiance, alpha);
    let resolved = resolve_history(current, guide, direction, step_length, in.uv);
    return VolumeOutput(resolved, guide);
}
