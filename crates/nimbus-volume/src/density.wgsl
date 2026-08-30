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

fn low_height_gradient(height: f32) -> f32 {
    let base_rise = smoothstep(0.0, 0.2, height);
    let top_taper = 1.0 - smoothstep(0.55, 1.0, height);
    return base_rise * top_taper;
}

fn density_at(position: vec3<f32>, bounds_min: f32, bounds_max: f32) -> f32 {
    let height = saturate((position.y - bounds_min) / max(bounds_max - bounds_min, 1.0));
    let drift = clouds.movement * clouds.elapsed_seconds;
    let weather_uv = (position.xz + drift) / 3000.0;
    let weather = textureSampleLevel(weather_texture, noise_sampler, weather_uv, 0.0);
    let support = smoothstep(0.25, 0.52, weather.r);
    // Dense presets widen the weather response instead of filling weak-weather regions uniformly.
    let deck_contrast = smoothstep(0.70, 0.90, clouds.density);
    let coverage = saturate(
        0.44 + 0.70 * (clouds.density - 0.52) - 0.10 * deck_contrast
            + (0.18 + 0.12 * deck_contrast) * support,
    );
    let shape_uv = vec3<f32>(
        (position.x + drift.x) * 0.00045 * clouds.scale,
        height * 0.72 + clouds.elapsed_seconds * 0.00013,
        (position.z + drift.y) * 0.00045 * clouds.scale,
    );
    let shape = textureSampleLevel(shape_texture, noise_sampler, shape_uv, 0.0);
    let low_fbm = dot(shape.gba, vec3<f32>(0.625, 0.25, 0.125));
    let base_noise = saturate((shape.r + 1.0 - low_fbm) / max(2.0 - low_fbm, 0.0001));
    let base_density = remap_density(base_noise, coverage) * low_height_gradient(height);
    if (base_density <= 0.001) {
        return 0.0;
    }
    let detail_uv = shape_uv * 7.0 + vec3<f32>(0.0, -clouds.elapsed_seconds * 0.0007, 0.0);
    let detail = textureSampleLevel(detail_texture, noise_sampler, detail_uv, 0.0);
    let detail_fbm = dot(detail.rgb, vec3<f32>(0.625, 0.25, 0.125));
    let edge = 1.0 - base_density;
    let erosion_enable = smoothstep(0.04, 0.16, height);
    let erosion = (1.0 - detail_fbm) * edge * edge * edge * 0.05 * erosion_enable;
    let density = saturate((base_density - erosion) * 3.80);
    return density * smoothstep(0.06, 0.16, density);
}
