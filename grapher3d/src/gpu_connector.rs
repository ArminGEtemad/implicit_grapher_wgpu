use std::{fs, path::PathBuf};

use wgpu::{
    Color, ColorTargetState, ColorWrites, FragmentState, MultisampleState, Operations,
    PipelineCompilationOptions, PipelineLayoutDescriptor, PrimitiveState,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    ShaderModuleDescriptor, VertexState,
};

use crate::gpu_resource::{FrameContext, GpuResource};

// helper function
fn load_shader(rel_path: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel_path);
    fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read shader {:?}\nError: {}", path, e))
}

pub struct GpuConnector {
    render_pipeline: RenderPipeline,
}

impl GpuConnector {
    pub fn new(gpu_res: &GpuResource) -> Self {
        let device = &gpu_res.device;
        let format = gpu_res.config.format;

        // connection to the shader
        let render_source = load_shader("shaders/render_shader.wgsl");
        let render_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Render shader"),
            source: wgpu::ShaderSource::Wgsl(render_source.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Fullscreen pipeline layout"),
            bind_group_layouts: &[],
            immediate_size: 0,
        });

        let fullscreen_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Fullscreen pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &render_shader,
                entry_point: Some("vs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[],
            },
            primitive: PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            fragment: Some(FragmentState {
                module: &render_shader,
                entry_point: Some("fs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                targets: &[Some(ColorTargetState {
                    format,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        Self {
            render_pipeline: fullscreen_pipeline,
        }
    }

    // hot reload
    fn reload_render_pipeline(&mut self, gpu_res: &GpuResource) {
        // connection to the shader
        let render_source = load_shader("shaders/render_shader.wgsl");
        let render_shader = gpu_res.device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Render shader (Hot reload)"),
            source: wgpu::ShaderSource::Wgsl(render_source.into()),
        });

        // rebuild pipeline
        let pipeline_layout = gpu_res
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Fullscreen pipeline layout (hot reload)"),
                bind_group_layouts: &[],
                immediate_size: 0,
            });

        self.render_pipeline = gpu_res
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("Fullscreen pipeline (Hot relod)"),
                layout: Some(&pipeline_layout),
                vertex: VertexState {
                    module: &render_shader,
                    entry_point: Some("vs_main"),
                    compilation_options: PipelineCompilationOptions::default(),
                    buffers: &[],
                },
                primitive: PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: MultisampleState::default(),
                fragment: Some(FragmentState {
                    module: &render_shader,
                    entry_point: Some("fs_main"),
                    compilation_options: PipelineCompilationOptions::default(),
                    targets: &[Some(ColorTargetState {
                        format: gpu_res.config.format,
                        blend: None,
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                multiview_mask: None,
                cache: None,
            });
    }

    pub fn rebuild_pipeline(&mut self, gpu_res: &GpuResource) {
        println!("Rebuilding render pipeline…");
        self.reload_render_pipeline(gpu_res);
        println!("Render pipeline reloaded!");
    }

    pub fn render_pass(&mut self, frame: &mut FrameContext) {
        let mut rpass = frame.encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Render pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &frame.view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: wgpu::LoadOp::Clear(Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        rpass.set_pipeline(&self.render_pipeline);
        rpass.draw(0..3, 0..1);
    }
}
