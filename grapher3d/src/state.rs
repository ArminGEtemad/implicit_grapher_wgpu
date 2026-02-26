use std::sync::Arc;
use wgpu::SurfaceError;
use winit::{dpi::PhysicalSize, window::Window};

use crate::{
    gpu_connector::GpuConnector,
    gpu_resource::{FrameContext, GpuResource},
};

pub struct State {
    gpu_res: GpuResource,
    connector: GpuConnector,
}

impl State {
    pub async fn new(window: Arc<Window>) -> Result<Self, String> {
        let gpu_res = GpuResource::new(window).await?;
        let connector = GpuConnector::new(&gpu_res);

        Ok(Self { gpu_res, connector })
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.gpu_res.resize(new_size);
    }

    pub fn render(&mut self) -> Result<(), SurfaceError> {
        let mut frame: FrameContext = self.gpu_res.begin_frame()?;
        self.connector.render_pass(&mut frame);
        self.gpu_res.submit_frame(frame);
        Ok(())
    }
}
