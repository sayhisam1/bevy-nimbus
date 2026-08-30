//! Render-world preparation and volumetric cloud passes.

mod volume;

use crate::pipeline::NimbusPipelines;
use bevy::{
    prelude::*,
    render::{
        diagnostic::{DiagnosticsRecorder, RecordDiagnostics},
        render_resource::{
            BindGroupEntries, LoadOp, Operations, PipelineCache, RenderPassColorAttachment,
            RenderPassDescriptor, RenderPipeline, Sampler, StoreOp, TextureView,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
    },
};
use nimbus_core::{CloudsUniform, CloudsUniformBuffer};

pub(crate) use volume::render_volume_clouds;

pub(super) fn attachment(view: &TextureView, clear: bool) -> RenderPassColorAttachment<'_> {
    RenderPassColorAttachment {
        view,
        depth_slice: None,
        resolve_target: None,
        ops: Operations {
            load: if clear {
                LoadOp::Clear(LinearRgba::NONE.into())
            } else {
                LoadOp::Load
            },
            store: StoreOp::Store,
        },
    }
}

pub(crate) fn prepare_uniforms(
    uniform: Option<Res<CloudsUniform>>,
    mut buffer: ResMut<CloudsUniformBuffer>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
) {
    if let Some(uniform) = uniform {
        buffer.0.set(uniform.clone());
    }
    buffer.0.write_buffer(&device, &queue);
}

pub(super) fn encode_composite(
    context: &mut RenderContext,
    recorder: Option<&DiagnosticsRecorder>,
    pipelines: &NimbusPipelines,
    cache: &PipelineCache,
    pipeline: &RenderPipeline,
    source: &TextureView,
    sampler: &Sampler,
    depth: &TextureView,
    target: &TextureView,
) {
    let group = context.render_device().create_bind_group(
        "nimbus_cloud_composite_group",
        &cache.get_bind_group_layout(&pipelines.composite_layout),
        &BindGroupEntries::sequential((source, sampler, depth)),
    );
    let mut pass = context
        .command_encoder()
        .begin_render_pass(&RenderPassDescriptor {
            label: Some("nimbus_cloud_composite"),
            color_attachments: &[Some(attachment(target, false))],
            ..default()
        });
    let timer = recorder.pass_span(&mut pass, "nimbus_cloud_composite");
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, &group, &[]);
    pass.draw(0..3, 0..1);
    timer.end(&mut pass);
}
