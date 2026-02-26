use std::sync::Arc;
use wgpu::{
    CommandEncoder, CommandEncoderDescriptor, Device, DeviceDescriptor, ExperimentalFeatures,
    Features, Instance, InstanceDescriptor, Limits, MemoryHints, Queue, RequestAdapterOptions,
    Surface, SurfaceConfiguration, SurfaceError, SurfaceTexture, TextureFormat, TextureUsages,
    TextureView, wgt::TextureViewDescriptor,
};
use winit::{dpi::PhysicalSize, window::Window};

pub struct GpuResource {
    surface: Surface<'static>,
    pub device: Device,
    queue: Queue,
    pub config: SurfaceConfiguration,
    size: PhysicalSize<u32>,
}

pub struct FrameContext {
    surface_tex: SurfaceTexture,
    pub view: TextureView,
    pub encoder: CommandEncoder,
}

impl GpuResource {
    pub async fn new(window: Arc<Window>) -> Result<Self, String> {
        let size = window.inner_size();

        let instance = Instance::new(&InstanceDescriptor::default());
        let surface = instance
            .create_surface(window)
            .expect("Failed to create surface!");

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .expect("Failed to create adapter!");

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: Some("Declaring device!"),
                required_features: Features::default(),
                required_limits: Limits::defaults(),
                experimental_features: ExperimentalFeatures::default(),
                memory_hints: MemoryHints::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .expect("Failed to find a device and create queue!");

        let surface_cap = surface.get_capabilities(&adapter);
        let surface_format = surface_cap
            .formats
            .iter()
            .copied()
            .find(|f| {
                matches!(
                    f,
                    TextureFormat::Bgra8UnormSrgb | TextureFormat::Rgba8UnormSrgb
                )
            })
            .unwrap_or(surface_cap.formats[0]);

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_cap.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            size,
        })
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn begin_frame(&self) -> Result<FrameContext, SurfaceError> {
        let surface_tex = self
            .surface
            .get_current_texture()
            .expect("Failed to get current texture!");

        let view = surface_tex
            .texture
            .create_view(&TextureViewDescriptor::default());

        let encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Main Encoder"),
            });

        Ok(FrameContext {
            surface_tex,
            view,
            encoder,
        })
    }

    pub fn submit_frame(&self, frame: FrameContext) {
        self.queue.submit([frame.encoder.finish()]);
        frame.surface_tex.present();
    }
}
