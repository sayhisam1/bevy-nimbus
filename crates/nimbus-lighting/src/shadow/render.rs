//! GPU world-shadow pipeline and render pass.

use super::NimbusCloudShadowMap;
use bevy::{
    core_pipeline::FullscreenShader,
    prelude::*,
    render::{
        diagnostic::RecordDiagnostics,
        render_asset::RenderAssets,
        render_resource::{
            BindGroupEntries, BindGroupLayoutDescriptor, BindGroupLayoutEntries,
            CachedRenderPipelineId, ColorTargetState, ColorWrites, FragmentState, LoadOp,
            Operations, PipelineCache, RenderPassColorAttachment, RenderPassDescriptor,
            RenderPipelineDescriptor, SamplerBindingType, ShaderStages, StoreOp, TextureSampleType,
            TextureView,
            binding_types::{sampler, texture_2d, texture_3d, uniform_buffer},
        },
        renderer::{RenderContext, ViewQuery},
        texture::GpuImage,
    },
};
use nimbus_core::{CloudsUniform, CloudsUniformBuffer, NimbusCloudView};
use nimbus_volume::{NubisTextures, gpu_nubis_textures};

#[derive(Resource)]
pub(super) struct ShadowShader(pub Handle<Shader>);

#[derive(Resource)]
pub(super) struct ShadowPipeline {
    pipeline: CachedRenderPipelineId,
    layout: BindGroupLayoutDescriptor,
}

fn layout() -> BindGroupLayoutDescriptor {
    BindGroupLayoutDescriptor::new(
        "nimbus_volume_shadow_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                uniform_buffer::<CloudsUniform>(true),
                texture_3d(TextureSampleType::Float { filterable: true }),
                texture_3d(TextureSampleType::Float { filterable: true }),
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
            ),
        ),
    )
}

#[expect(clippy::needless_pass_by_value, reason = "Bevy system parameters")]
pub(super) fn init_shadow_pipeline(
    mut commands: Commands,
    cache: Res<PipelineCache>,
    fullscreen: Res<FullscreenShader>,
    shader: Res<ShadowShader>,
) {
    let layout = layout();
    let pipeline = cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some("nimbus_volume_shadow".into()),
        layout: vec![layout.clone()],
        vertex: fullscreen.to_vertex_state(),
        fragment: Some(FragmentState {
            shader: shader.0.clone(),
            entry_point: Some("fs_main".into()),
            targets: vec![Some(ColorTargetState {
                format: bevy::render::render_resource::TextureFormat::R8Unorm,
                blend: None,
                write_mask: ColorWrites::ALL,
            })],
            ..default()
        }),
        ..default()
    });
    commands.insert_resource(ShadowPipeline { pipeline, layout });
}

fn attachment(view: &TextureView) -> RenderPassColorAttachment<'_> {
    RenderPassColorAttachment {
        view,
        depth_slice: None,
        resolve_target: None,
        ops: Operations {
            load: LoadOp::Clear(LinearRgba::WHITE.into()),
            store: StoreOp::Store,
        },
    }
}

fn clear_shadow_target(context: &mut RenderContext, view: &TextureView) {
    let _pass = context
        .command_encoder()
        .begin_render_pass(&RenderPassDescriptor {
            label: Some("clear_nimbus_volume_shadow"),
            color_attachments: &[Some(attachment(view))],
            ..default()
        });
}

#[expect(
    clippy::needless_pass_by_value,
    clippy::too_many_arguments,
    reason = "the render graph supplies explicit system parameters"
)]
pub(super) fn render_shadow_coverage(
    _view: ViewQuery<&NimbusCloudView>,
    pipelines: Option<Res<ShadowPipeline>>,
    pipeline_cache: Res<PipelineCache>,
    buffer: Option<Res<CloudsUniformBuffer>>,
    map: Option<Res<NimbusCloudShadowMap>>,
    volume_handle: Option<Res<NubisTextures>>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    mut context: RenderContext,
) {
    let (Some(pipelines), Some(map)) = (pipelines, map) else {
        return;
    };
    let Some(target) = gpu_images.get(&map.texture) else {
        return;
    };
    if !map.enabled {
        clear_shadow_target(&mut context, &target.texture_view);
        return;
    }
    let (Some(buffer), Some(volume)) = (buffer, volume_handle) else {
        clear_shadow_target(&mut context, &target.texture_view);
        return;
    };
    let (Some(pipeline), Some(volume_images), Some(uniform_binding)) = (
        pipeline_cache.get_render_pipeline(pipelines.pipeline),
        gpu_nubis_textures(&volume, &gpu_images),
        buffer.0.binding(),
    ) else {
        clear_shadow_target(&mut context, &target.texture_view);
        return;
    };
    let group = context.render_device().create_bind_group(
        "nimbus_volume_shadow_group",
        &pipeline_cache.get_bind_group_layout(&pipelines.layout),
        &BindGroupEntries::sequential((
            uniform_binding,
            &volume_images[0].texture_view,
            &volume_images[1].texture_view,
            &volume_images[2].texture_view,
            &volume_images[0].sampler,
        )),
    );
    let diagnostics = context.diagnostic_recorder();
    let recorder = diagnostics.as_deref();
    let mut pass = context
        .command_encoder()
        .begin_render_pass(&RenderPassDescriptor {
            label: Some("nimbus_volume_shadow"),
            color_attachments: &[Some(attachment(&target.texture_view))],
            ..default()
        });
    let timer = recorder.pass_span(&mut pass, "nimbus_cloud_shadow");
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, &group, &[0]);
    pass.draw(0..3, 0..1);
    timer.end(&mut pass);
}
