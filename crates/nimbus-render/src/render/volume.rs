//! Volumetric Nubis march, temporal resolve, and full-resolution composite.

use super::{CloudsUniformBuffer, attachment};
use crate::{pipeline::NimbusPipelines, temporal::VolumeViewState};
use bevy::{
    prelude::*,
    render::{
        diagnostic::RecordDiagnostics,
        render_asset::RenderAssets,
        render_resource::{BindGroupEntries, PipelineCache, RenderPassDescriptor},
        renderer::{RenderContext, RenderQueue, ViewQuery},
        texture::GpuImage,
        view::{ViewDepthTexture, ViewTarget},
    },
};
use nimbus_core::{CloudsUniform, NimbusCloudView};
use nimbus_volume::{NubisTextures, gpu_nubis_textures};
use std::collections::HashMap;

#[expect(
    clippy::too_many_arguments,
    reason = "the render graph supplies explicit system parameters"
)]
pub(crate) fn render_volume_clouds(
    view: ViewQuery<(&ViewTarget, &ViewDepthTexture, &NimbusCloudView)>,
    pipelines: Option<Res<NimbusPipelines>>,
    pipeline_cache: Res<PipelineCache>,
    buffer: Option<Res<CloudsUniformBuffer>>,
    uniform: Option<Res<CloudsUniform>>,
    volume_handle: Option<Res<NubisTextures>>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    render_queue: Res<RenderQueue>,
    all_views: Query<(), With<ViewTarget>>,
    mut views: Local<HashMap<Entity, VolumeViewState>>,
    mut context: RenderContext,
) {
    let (Some(pipelines), Some(buffer), Some(uniform), Some(volume)) =
        (pipelines, buffer, uniform, volume_handle)
    else {
        return;
    };
    let (Some(pipeline), Some(composite)) = (
        pipeline_cache.get_render_pipeline(pipelines.volume),
        pipeline_cache.get_render_pipeline(pipelines.composite),
    ) else {
        return;
    };
    let Some(volume_images) = gpu_nubis_textures(&volume, &gpu_images) else {
        return;
    };
    let Some(uniform_binding) = buffer.0.binding() else {
        return;
    };

    views.retain(|entity, _| all_views.contains(*entity));
    let view_entity = view.entity();
    let (target, depth, _) = view.into_inner();
    let full_size = target.main_texture().size();
    let divisor = uniform.resolution_divisor.max(1);
    let reduced_size = UVec2::new(
        full_size.width.div_ceil(divisor),
        full_size.height.div_ceil(divisor),
    );
    let state = views
        .entry(view_entity)
        .or_insert_with(|| VolumeViewState::new(context.render_device(), reduced_size));
    state.resize_if_needed(context.render_device(), reduced_size);
    state.begin_frame(&uniform);
    state
        .temporal_uniform
        .write_buffer(context.render_device(), &render_queue);
    let Some(temporal_binding) = state.temporal_uniform.binding() else {
        return;
    };
    let (history_color, history_guide) = state.read_views();
    let history_sampler = state.history_sampler();
    let (write_color, write_guide) = state.write_views();
    let depth_view = depth.view().clone();
    let group = context.render_device().create_bind_group(
        "nimbus_volume_group",
        &pipeline_cache.get_bind_group_layout(&pipelines.volume_layout),
        &BindGroupEntries::sequential((
            uniform_binding,
            &volume_images[0].texture_view,
            &volume_images[1].texture_view,
            &volume_images[2].texture_view,
            &volume_images[0].sampler,
            &depth_view,
            temporal_binding,
            history_color,
            history_guide,
            history_sampler,
        )),
    );
    let diagnostics = context.diagnostic_recorder();
    let recorder = diagnostics.as_deref();
    {
        let color_attachments = [
            Some(attachment(write_color, true)),
            Some(attachment(write_guide, true)),
        ];
        let mut pass = context
            .command_encoder()
            .begin_render_pass(&RenderPassDescriptor {
                label: Some("nimbus_volume_march"),
                color_attachments: &color_attachments,
                ..default()
            });
        let timer = recorder.pass_span(&mut pass, "nimbus_cloud_volume");
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &group, &[0]);
        pass.draw(0..3, 0..1);
        timer.end(&mut pass);
    }
    super::encode_composite(
        &mut context,
        recorder,
        &pipelines,
        &pipeline_cache,
        composite,
        write_color,
        &volume_images[0].sampler,
        &depth_view,
        target.main_texture_view(),
    );
    state.complete_frame();
}
