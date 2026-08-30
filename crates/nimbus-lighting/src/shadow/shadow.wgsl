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
    let receiver = vec3<f32>((in.uv - 0.5) * SHADOW_WORLD_SIZE, 0.0);
    let entry_distance = max((bottom - receiver.y) / light_direction.y, 0.0);
    let origin = receiver + light_direction * entry_distance;
    let path_length = min((top - bottom) / light_direction.y, 32000.0);
    let vertical_samples = max(f32(clouds.view_steps) * 0.5, 12.0);
    let fine_step = clamp((top - bottom) / vertical_samples, 40.0, 100.0);
    let pixel = vec2<u32>(in.position.xy);
    let jitter = f32(hash_u32(pixel.x ^ pixel.y * 0x9e3779b9u) & 0x00ffffffu)
        / 16777216.0;
    var distance = fine_step * jitter;
    var transmittance = 1.0;
    for (var iteration = 0u; iteration < 64u; iteration++) {
        if (distance >= path_length || transmittance <= 0.01) {
            break;
        }
        let position = origin + light_direction * distance;
        let density = density_at(position, bottom, top);
        if (density > 0.001) {
            transmittance *= exp(-density * fine_step * EXTINCTION);
        }
        distance += fine_step;
    }
    return vec4<f32>(transmittance, 0.0, 0.0, 1.0);
}
