#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}

const SHADOW_WORLD_SIZE: f32 = 8192.0;

struct ShadowGroundParams {
    direction_amount: vec4<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(100)
var shadow_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(101)
var shadow_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(102)
var<uniform> shadow: ShadowGroundParams;

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    let sun = normalize(shadow.direction_amount.xyz);
    if (sun.y > 0.001) {
        // Project elevated receivers back to the map's y=0 reference plane.
        let reference_xz = in.world_position.xz - sun.xz * (in.world_position.y / sun.y);
        let uv = reference_xz / SHADOW_WORLD_SIZE + vec2<f32>(0.5);
        let inside = all(uv >= vec2<f32>(0.0)) && all(uv <= vec2<f32>(1.0));
        let sampled = textureSampleLevel(
            shadow_texture,
            shadow_sampler,
            clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0)),
            0.0,
        ).r;
        let transmittance = select(1.0, sampled, inside);
        // The unlit diagnostic material visualizes raw direct-sun transmittance.
        let direct_sun = mix(1.0 - shadow.direction_amount.w, 1.0, transmittance);
        pbr_input.material.base_color *= vec4<f32>(vec3<f32>(direct_sun), 1.0);
    }
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}
