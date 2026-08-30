#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

const SHADOW_WORLD_SIZE: f32 = 8192.0;

fn hash_u32(value: u32) -> u32 {
    var hash = value;
    hash = (hash ^ 61u) ^ (hash >> 16u);
    hash *= 9u;
    hash ^= hash >> 4u;
    hash *= 0x27d4eb2du;
    return hash ^ (hash >> 15u);
}

@fragment
fn fs_main(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    if (
        clouds.directional_light_count == 0u
        || clouds.directional_lights[0].strength <= 0.0
    ) {
        return vec4<f32>(1.0, 0.0, 0.0, 1.0);
    }
    let light_direction = normalize(clouds.directional_lights[0].direction_to_light);
    if (light_direction.y <= 0.001) {
        return vec4<f32>(1.0, 0.0, 0.0, 1.0);
    }
    let bottom = clouds.base_altitude;
    let top = bottom + max(clouds.thickness, 800.0);
    let receiver_xz = (in.uv - vec2<f32>(0.5)) * SHADOW_WORLD_SIZE;
    let receiver = vec3<f32>(receiver_xz.x, 0.0, receiver_xz.y);
    let sample_count = clamp(clouds.view_steps / 2u, 16u, 32u);
    let vertical_step = (top - bottom) / f32(sample_count);
    let ray_step = vertical_step / light_direction.y;
    let pixel = vec2<u32>(in.position.xy);
    let jitter = f32(hash_u32(pixel.x ^ pixel.y * 0x9e3779b9u) & 0x00ffffffu)
        / 16777216.0;
    var optical_depth = 0.0;
    for (var index = 0u; index < 32u; index++) {
        if (index >= sample_count || optical_depth * EXTINCTION >= 4.6) {
            break;
        }
        let sample_y = bottom + (f32(index) + jitter) * vertical_step;
        let distance = (sample_y - receiver.y) / light_direction.y;
        let position = receiver + light_direction * distance;
        optical_depth += density_at(position, bottom, top, ray_step) * ray_step;
    }
    let transmittance = exp(-optical_depth * EXTINCTION);
    return vec4<f32>(transmittance, 0.0, 0.0, 1.0);
}
