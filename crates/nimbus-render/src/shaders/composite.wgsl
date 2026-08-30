#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var cloud_layer: texture_2d<f32>;
@group(0) @binding(1) var cloud_sampler: sampler;
@group(0) @binding(2) var scene_depth: texture_depth_2d;

@fragment
fn fs_main(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let pixel = vec2<i32>(in.position.xy);
    if (textureLoad(scene_depth, pixel, 0) > 0.0) {
        return vec4<f32>(0.0);
    }
    return textureSample(cloud_layer, cloud_sampler, in.uv);
}
