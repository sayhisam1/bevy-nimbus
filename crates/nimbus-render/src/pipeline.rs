//! Volumetric render-pipeline registration.

mod layouts;

use bevy::{
    core_pipeline::FullscreenShader,
    prelude::*,
    render::render_resource::{
        BindGroupLayoutDescriptor, BlendState, CachedRenderPipelineId, ColorTargetState,
        ColorWrites, FragmentState, PipelineCache, RenderPipelineDescriptor, TextureFormat,
    },
};

pub(crate) const COMPOSITE_WGSL: &str = include_str!("shaders/composite.wgsl");

#[derive(Resource)]
pub(super) struct CloudShaders {
    pub(super) volume: Handle<Shader>,
    pub(super) composite: Handle<Shader>,
}

fn queue_volume_pipeline(
    cache: &PipelineCache,
    fullscreen: &FullscreenShader,
    layout: &BindGroupLayoutDescriptor,
    shader: &Handle<Shader>,
) -> CachedRenderPipelineId {
    let target = |format| {
        Some(ColorTargetState {
            format,
            blend: None,
            write_mask: ColorWrites::ALL,
        })
    };
    cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some("nimbus_volume".into()),
        layout: vec![layout.clone()],
        vertex: fullscreen.to_vertex_state(),
        fragment: Some(FragmentState {
            shader: shader.clone(),
            entry_point: Some("fs_main".into()),
            targets: vec![
                target(TextureFormat::Rgba16Float),
                target(TextureFormat::Rg16Float),
            ],
            ..default()
        }),
        ..default()
    })
}

pub(crate) fn init_pipelines(
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    fullscreen_shader: Res<FullscreenShader>,
    shaders: Res<CloudShaders>,
) {
    let volume_layout = layouts::volume();
    let composite_layout = layouts::composite();
    let queue = |layout: &BindGroupLayoutDescriptor,
                 shader: &Handle<Shader>,
                 label: &'static str,
                 format: TextureFormat,
                 blend: Option<BlendState>| {
        pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
            label: Some(label.into()),
            layout: vec![layout.clone()],
            vertex: fullscreen_shader.to_vertex_state(),
            fragment: Some(FragmentState {
                shader: shader.clone(),
                entry_point: Some("fs_main".into()),
                targets: vec![Some(ColorTargetState {
                    format,
                    blend,
                    write_mask: ColorWrites::ALL,
                })],
                ..default()
            }),
            ..default()
        })
    };
    commands.insert_resource(NimbusPipelines {
        volume: queue_volume_pipeline(
            &pipeline_cache,
            &fullscreen_shader,
            &volume_layout,
            &shaders.volume,
        ),
        composite: queue(
            &composite_layout,
            &shaders.composite,
            "nimbus_cloud_composite",
            TextureFormat::Rgba16Float,
            Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
        ),
        composite_layout,
        volume_layout,
    });
}

#[derive(Resource)]
pub(crate) struct NimbusPipelines {
    pub(crate) volume: CachedRenderPipelineId,
    pub(crate) composite: CachedRenderPipelineId,
    pub(crate) composite_layout: BindGroupLayoutDescriptor,
    pub(crate) volume_layout: BindGroupLayoutDescriptor,
}
