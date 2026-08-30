#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}

struct ShadowGroundParams {
    sun_strength: vec4<f32>,
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
    let sun = normalize(shadow.sun_strength.xyz);
    if (sun.y > 0.01) {
        let uv = in.world_position.xz / 8192.0 + 0.5;
        let transmittance = textureSample(shadow_texture, shadow_sampler, uv).r;
        let dapple = smoothstep(0.18, 0.82, transmittance);
        let light = mix(1.0 - shadow.sun_strength.w, 1.0, dapple);
        pbr_input.material.base_color *= vec4<f32>(vec3<f32>(light), 1.0);
    }
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}
