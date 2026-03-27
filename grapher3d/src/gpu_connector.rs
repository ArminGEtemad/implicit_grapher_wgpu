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

use crate::{
    gpu_resource::{FrameContext, GpuResource},
    parser::{Lexer, Parser},
};

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
    pub target: [f32; 3],
    pub _pad0: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct PlotConfigUniform {
    pub min_bounds: [f32; 3],
    pub _pad0: f32,

    pub max_bounds: [f32; 3],
    pub _pad1: f32,
}

pub struct GpuConnector {
    camera_buffer: Buffer,
    plot_config_buffer: Buffer,
    render_bgl: BindGroupLayout,
    render_bg: BindGroup,
    render_pipeline: RenderPipeline,
}

impl GpuConnector {
    fn create_shader_source(user_input: &str) -> String {
        let shader = load_shader("shaders/render_shader.wgsl");

        let lexer = Lexer::new(user_input);
        let mut parser = Parser::new(lexer.tokens);
        let ast = parser.weak_handle();
        let wgsl_expr = ast.to_wgsl_code();

        shader.replace("USER_INPUT", &wgsl_expr)
    }

    pub fn new(gpu_res: &GpuResource, implicit_formula: &str) -> Self {
        let device = &gpu_res.device;
        let format = gpu_res.config.format;
        let aspect_ratio = gpu_res.config.width as f32 / gpu_res.config.height as f32;

        let initial_shader = Self::create_shader_source(implicit_formula);

        // connection to the shader
        //let render_source = load_shader("shaders/render_shader.wgsl");
        let render_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Render shader"),
            source: wgpu::ShaderSource::Wgsl(initial_shader.into()),
        });

        // uniform buffer (camera and plot config)
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

        // Plot config
        let plot_config_contents = PlotConfigUniform {
            min_bounds: [0.0, 0.0, 0.0],
            _pad0: 0.0,
            max_bounds: [0.0, 0.0, 0.0],
            _pad1: 0.0,
        };

        let plot_config_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Plot Config Buffer"),
            contents: cast_slice(&[plot_config_contents]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        // BGL
        let render_bgl = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Render BGL"),
            entries: &[
                BindGroupLayoutEntry {
                    // camera uniform
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    // plot config uniform
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // BG
        let render_bg = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Render BG"),
            layout: &render_bgl,
            entries: &[
                BindGroupEntry {
                    // camera
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    // plot config
                    binding: 1,
                    resource: plot_config_buffer.as_entire_binding(),
                },
            ],
        });

        // render layout
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Fullscreen pipeline layout"),
            bind_group_layouts: &[&render_bgl],
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
            plot_config_buffer,
            render_bgl,
            render_bg,
            render_pipeline: fullscreen_pipeline,
        }
    }

    // get the limits
    pub fn plot_limits(&self, gpu_res: &GpuResource, min_bounds: [f32; 3], max_bounds: [f32; 3]) {
        let plot_buffer = PlotConfigUniform {
            min_bounds,
            _pad0: 0.0,
            max_bounds,
            _pad1: 0.0,
        };

        gpu_res
            .queue
            .write_buffer(&self.plot_config_buffer, 0, cast_slice(&[plot_buffer]));
    }

    // update camera
    pub fn update_camera(&self, gpu_res: &GpuResource, width: u32, height: u32) {
        let aspect_ratio = width as f32 / height as f32;

        let updated_camera_buffer_contents = CameraUniform {
            position: [10.0, 10.0, 10.0],
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
    fn reload_render_pipeline(&mut self, gpu_res: &GpuResource, user_input: &str) {
        // connection to the shader
        let input_source = Self::create_shader_source(user_input);
        let render_shader = gpu_res.device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Render shader (Hot reload)"),
            source: wgpu::ShaderSource::Wgsl(input_source.into()),
        });

        // rebuild pipeline
        let pipeline_layout = gpu_res
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Fullscreen pipeline layout (hot reload)"),
                bind_group_layouts: &[&self.render_bgl],
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

    pub fn rebuild_pipeline(&mut self, gpu_res: &GpuResource, formula: &str) {
        println!("Rebuilding render pipeline…");
        self.reload_render_pipeline(gpu_res, formula);
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
        rpass.set_bind_group(0, &self.render_bg, &[]);
        rpass.draw(0..3, 0..1);
    }
}
