struct DirectionalLight {
    direction_to_light: vec3<f32>,
    strength: f32,
    color: vec3<f32>,
}

struct CloudUniform {
    camera_pos: vec3<f32>,
    elapsed_seconds: f32,
    camera_forward: vec3<f32>,
    directional_light_count: u32,
    camera_right: vec3<f32>,
    ambient_strength: f32,
    camera_up: vec3<f32>,
    density: f32,
    ambient_color: vec3<f32>,
    scale: f32,
    movement: vec2<f32>,
    base_altitude: f32,
    thickness: f32,
    base_color: vec3<f32>,
    view_steps: u32,
    directional_lights: array<DirectionalLight, 4>,
    light_steps: u32,
    resolution_divisor: u32,
}

@group(0) @binding(0) var<uniform> clouds: CloudUniform;
@group(0) @binding(1) var shape_texture: texture_3d<f32>;
@group(0) @binding(2) var detail_texture: texture_3d<f32>;
@group(0) @binding(3) var weather_texture: texture_2d<f32>;
@group(0) @binding(4) var noise_sampler: sampler;

const EXTINCTION: f32 = 0.0045;

fn saturate(value: f32) -> f32 {
    return clamp(value, 0.0, 1.0);
}

fn remap_density(value: f32, coverage: f32) -> f32 {
    return saturate((value - (1.0 - coverage)) / max(coverage, 0.0001));
}

const WEATHER_WORLD_SIZE: f32 = 60000.0;
const SHAPE_WORLD_SIZE_XZ: f32 = 10000.0;
const SHAPE_WORLD_SIZE_Y: f32 = 4000.0;
const DETAIL_MULTIPLIER: f32 = 6.0;

fn cloud_height_gradient(height: f32, cloud_type: f32) -> f32 {
    let stratus = smoothstep(0.00, 0.08, height)
        * (1.0 - smoothstep(0.22, 0.35, height));
    let stratocumulus = smoothstep(0.00, 0.12, height)
        * (1.0 - smoothstep(0.45, 0.70, height));
    let cumulus = smoothstep(0.00, 0.10, height)
        * (1.0 - smoothstep(0.78, 1.00, height));
    let low_type = mix(stratus, stratocumulus, saturate(cloud_type * 2.0));
    return mix(low_type, cumulus, saturate((cloud_type - 0.5) * 2.0));
}

fn density_at(
    position: vec3<f32>,
    bounds_min: f32,
    bounds_max: f32,
    sample_length: f32,
) -> f32 {
    let height = saturate((position.y - bounds_min) / max(bounds_max - bounds_min, 1.0));
    let drift = clouds.movement * clouds.elapsed_seconds;
    let footprint = clamp(sample_length, 1.0, 80.0);

    let weather_uv = (position.xz + drift) / WEATHER_WORLD_SIZE;
    let weather_lod = clamp(log2(max(footprint * 512.0 / WEATHER_WORLD_SIZE, 1.0)), 0.0, 9.0);
    let weather = textureSampleLevel(weather_texture, noise_sampler, weather_uv, weather_lod);
    let coverage = saturate(clouds.density + 0.10 + (weather.r - 0.5) * 0.45);

    let shape_uv = vec3<f32>(
        (position.x + drift.x) / SHAPE_WORLD_SIZE_XZ,
        position.y / SHAPE_WORLD_SIZE_Y,
        (position.z + drift.y) / SHAPE_WORLD_SIZE_XZ,
    ) * clouds.scale;
    let shape_lod = clamp(
        log2(max(footprint * 128.0 * clouds.scale / SHAPE_WORLD_SIZE_XZ, 1.0)),
        0.0,
        7.0,
    );
    let shape = textureSampleLevel(shape_texture, noise_sampler, shape_uv, shape_lod);
    let low_fbm = dot(shape.gba, vec3<f32>(0.625, 0.25, 0.125));
    let base_noise = saturate((shape.r + 1.0 - low_fbm) / max(2.0 - low_fbm, 0.0001));
    let cloud_type = saturate(weather.g * 0.5 + (1.0 - clouds.density) * 1.4);
    let height_gradient = cloud_height_gradient(height, cloud_type);
    var density = remap_density(base_noise, coverage) * height_gradient;
    if (density <= 0.001) {
        return 0.0;
    }

    let detail_uv = shape_uv * DETAIL_MULTIPLIER;
    let detail_world_size = SHAPE_WORLD_SIZE_XZ / (DETAIL_MULTIPLIER * clouds.scale);
    let detail_lod = clamp(
        log2(max(footprint * 32.0 / detail_world_size, 1.0)),
        0.0,
        5.0,
    );
    let detail = textureSampleLevel(detail_texture, noise_sampler, detail_uv, detail_lod);
    let detail_fbm = dot(detail.rgb, vec3<f32>(0.625, 0.25, 0.125));
    let detail_modifier = mix(detail_fbm, 1.0 - detail_fbm, saturate(height * 10.0));
    let erosion_floor = detail_modifier * mix(0.10, 0.25, saturate(height * 4.0));
    return saturate((density - erosion_floor) / max(1.0 - erosion_floor, 0.001));
}
