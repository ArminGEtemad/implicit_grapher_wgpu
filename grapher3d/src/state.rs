use std::sync::Arc;
use wgpu::SurfaceError;
use winit::{dpi::PhysicalSize, window::Window};

use crate::{
    gpu_connector::GpuConnector,
    gpu_resource::{FrameContext, GpuResource},
    shader_watcher::ShaderWatcher,
};

pub struct State {
    gpu_res: GpuResource,
    connector: GpuConnector,
    shader_watcher: ShaderWatcher,
}

impl State {
    pub async fn new(window: Arc<Window>) -> Result<Self, String> {
        let gpu_res = GpuResource::new(window).await?;
        let connector = GpuConnector::new(&gpu_res);

        let shader_path = format!("{}/shaders", env!("CARGO_MANIFEST_DIR"));
        println!("Watching shaders at: {}", shader_path);
        let shader_watcher = ShaderWatcher::new(shader_path);

        Ok(Self {
            gpu_res,
            connector,
            shader_watcher,
        })
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.gpu_res.resize(new_size);
        self.connector
            .update_camera(&self.gpu_res, new_size.width, new_size.height);
    }

    pub fn render(&mut self) -> Result<(), SurfaceError> {
        while let Ok(path) = self.shader_watcher.reciever_x.try_recv() {
            println!("Shader changed: {:?}", path);
            self.connector.rebuild_pipeline(&self.gpu_res);
        }

        let mut frame: FrameContext = self.gpu_res.begin_frame()?;
        self.connector.render_pass(&mut frame);
        self.gpu_res.submit_frame(frame);
        Ok(())
    }
}
