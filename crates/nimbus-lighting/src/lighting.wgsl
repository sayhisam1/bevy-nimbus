// Accepted Nubis dual-lobe phase model. These are model coefficients, not artist controls.
const LIGHTING_PI: f32 = 3.14159265;
const FORWARD_ANISOTROPY: f32 = 0.65;
const BACKWARD_ANISOTROPY: f32 = -0.2;
const FORWARD_LOBE_WEIGHT: f32 = 0.75;
const DIRECT_BASE: f32 = 0.18;
const PHASE_GAIN: f32 = 5.0;

// Three scattering orders. Extinction and anisotropy halve per order; attenuation falls by 0.8.
// Higher orders recycle only the square of missing first-order light, keeping the integrated
// visibility envelope at or below one while leaving fully lit cloud unchanged.
const MS_SECOND_EXTINCTION: f32 = 0.5;
const MS_THIRD_EXTINCTION: f32 = 0.25;
const MS_SECOND_ATTENUATION: f32 = 0.8;
const MS_THIRD_ATTENUATION: f32 = 0.64;

fn henyey_greenstein(cosine: f32, anisotropy: f32) -> f32 {
    let squared = anisotropy * anisotropy;
    let denominator = max(1.0 + squared - 2.0 * anisotropy * cosine, 0.0001);
    return (1.0 - squared) / (4.0 * LIGHTING_PI * denominator * sqrt(denominator));
}

fn dual_lobe_phase(
    view_direction: vec3<f32>,
    light_direction: vec3<f32>,
    anisotropy_scale: f32,
) -> f32 {
    let cosine = clamp(dot(view_direction, normalize(light_direction)), -1.0, 1.0);
    return mix(
        henyey_greenstein(cosine, BACKWARD_ANISOTROPY * anisotropy_scale),
        henyey_greenstein(cosine, FORWARD_ANISOTROPY * anisotropy_scale),
        FORWARD_LOBE_WEIGHT,
    );
}

fn scattering_order(
    view_direction: vec3<f32>,
    light_direction: vec3<f32>,
    visibility: f32,
    anisotropy_scale: f32,
) -> f32 {
    let phase = dual_lobe_phase(view_direction, light_direction, anisotropy_scale);
    return visibility * (DIRECT_BASE + phase * PHASE_GAIN);
}

fn directional_cloud_radiance(
    view_direction: vec3<f32>,
    light: DirectionalLight,
    transmittance: f32,
) -> vec3<f32> {
    let first_visibility = clamp(transmittance, 0.0, 1.0);
    let second_visibility = sqrt(first_visibility);
    let third_visibility = sqrt(second_visibility);
    let unscattered_energy = 1.0 - first_visibility;
    let recycled_energy = unscattered_energy * unscattered_energy;
    let low_angle_scattering = 1.0 - smoothstep(
        0.45,
        0.70,
        abs(light.direction_to_light.y),
    );
    let scattering =
        scattering_order(
            view_direction,
            light.direction_to_light,
            first_visibility,
            1.0,
        )
        + low_angle_scattering * recycled_energy * (
            MS_SECOND_ATTENUATION * scattering_order(
                view_direction,
                light.direction_to_light,
                second_visibility,
                MS_SECOND_EXTINCTION,
            )
            + MS_THIRD_ATTENUATION * scattering_order(
                view_direction,
                light.direction_to_light,
                third_visibility,
                MS_THIRD_EXTINCTION,
            )
        );
    return light.color * light.strength * scattering;
}

fn ambient_cloud_radiance(
    color: vec3<f32>,
    strength: f32,
    height_fraction: f32,
    light_vertical: f32,
) -> vec3<f32> {
    let low_sun = 1.0 - smoothstep(0.08, 0.30, abs(light_vertical));
    let height = clamp(height_fraction, 0.0, 1.0);
    let top_down_ambient = 0.85 + 0.30 * height;
    // Both profiles integrate to one over the slab. The cubic low-sun profile concentrates
    // existing sky energy at the visible underside without increasing total ambient energy.
    let underside = 1.0 - height;
    let underside_ambient = 4.0 * underside * underside * underside;
    let directional_ambient = mix(top_down_ambient, underside_ambient, low_sun);
    return mix(color, vec3<f32>(1.0), 0.45) * strength * 0.55 * directional_ambient;
}
