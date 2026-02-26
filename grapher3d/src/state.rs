use std::sync::Arc;
use wgpu::SurfaceError;
use winit::{dpi::PhysicalSize, window::Window};

use crate::gpu_resource::{FrameContext, GpuResource};

pub struct State {
    gpu_res: GpuResource,
}

impl State {
    pub async fn new(window: Arc<Window>) -> Result<Self, String> {
        let gpu_res = GpuResource::new(window).await?;

        Ok(Self { gpu_res })
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.gpu_res.resize(new_size);
    }

    pub fn render(&mut self) -> Result<(), SurfaceError> {
        let frame: FrameContext = self.gpu_res.begin_frame()?;
        self.gpu_res.submit_frame(frame);
        Ok(())
    }
}
