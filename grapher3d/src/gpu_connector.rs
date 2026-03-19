use std::{fs, path::PathBuf};

use bytemuck::{Pod, Zeroable, cast_slice};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferUsages, Color, ColorTargetState, ColorWrites,
    FragmentState, MultisampleState, Operations, PipelineCompilationOptions,
    PipelineLayoutDescriptor, PrimitiveState, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderStages, VertexState,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::gpu_resource::{FrameContext, GpuResource};

// helper function
fn load_shader(rel_path: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel_path);
    fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read shader {:?}\nError: {}", path, e))
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct CameraUniform {
    pub position: [f32; 3],
    pub aspect_ratio: f32,
    pub target: [f32; 3], // direction the camera is pointing at
    _pad0: f32,
}

pub struct GpuConnector {
    camera_buffer: Buffer,
    camera_bgl: BindGroupLayout,
    camera_bg: BindGroup,
    render_pipeline: RenderPipeline,
}

impl GpuConnector {
    pub fn new(gpu_res: &GpuResource) -> Self {
        let device = &gpu_res.device;
        let format = gpu_res.config.format;
        let aspect_ratio = gpu_res.config.width as f32 / gpu_res.config.height as f32;

        // connection to the shader
        let render_source = load_shader("shaders/render_shader.wgsl");
        let render_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Render shader"),
            source: wgpu::ShaderSource::Wgsl(render_source.into()),
        });

        // camera
        let camera_buffer_contents = CameraUniform {
            position: [0.0, 0.0, 0.0],
            aspect_ratio: aspect_ratio,
            target: [0.0, 0.0, 0.0],
            _pad0: 0.0,
        };

        let camera_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: cast_slice(&[camera_buffer_contents]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let camera_bgl = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Camera BGL"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let camera_bg = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Camera BG"),
            layout: &camera_bgl,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        // render layout
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Fullscreen pipeline layout"),
            bind_group_layouts: &[&camera_bgl],
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
            camera_buffer,
            camera_bgl,
            camera_bg,
            render_pipeline: fullscreen_pipeline,
        }
    }

    // update camera
    pub fn update_camera(&self, gpu_res: &GpuResource, width: u32, height: u32) {
        let aspect_ratio = width as f32 / height as f32;

        let updated_camera_buffer_contents = CameraUniform {
            position: [3.0, 3.0, 3.0],
            aspect_ratio: aspect_ratio,
            target: [0.0, 0.0, 0.0],
            _pad0: 0.0,
        };

        gpu_res.queue.write_buffer(
            &self.camera_buffer,
            0,
            cast_slice(&[updated_camera_buffer_contents]),
        );
    }

    // handling position with keyboard
    pub fn update_camera_pos(&self, gpu_res: &GpuResource, pos: [f32; 3], target: [f32; 3]) {
        let aspect_ratio = gpu_res.config.width as f32 / gpu_res.config.height as f32;

        let updated_camera_pos_buffer_contents = CameraUniform {
            position: pos,
            aspect_ratio,
            target,
            _pad0: 0.0,
        };

        gpu_res.queue.write_buffer(
            &self.camera_buffer,
            0,
            cast_slice(&[updated_camera_pos_buffer_contents]),
        );
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
                bind_group_layouts: &[&self.camera_bgl],
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
        rpass.set_bind_group(0, &self.camera_bg, &[]);
        rpass.draw(0..3, 0..1);
    }
}
