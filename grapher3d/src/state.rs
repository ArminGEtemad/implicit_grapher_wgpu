use std::sync::Arc;
use wgpu::SurfaceError;
use winit::{dpi::PhysicalSize, event::MouseScrollDelta, keyboard::KeyCode, window::Window};

use crate::{
    gpu_connector::GpuConnector,
    gpu_resource::{FrameContext, GpuResource},
    shader_watcher::ShaderWatcher,
};

pub struct State {
    gpu_res: GpuResource,
    connector: GpuConnector,
    camera_spherical_coord: [f32; 3], // r, thera, phi
    camera_pointing_at: [f32; 3],     // x, y, z
    shader_watcher: ShaderWatcher,
}

impl State {
    pub async fn new(window: Arc<Window>) -> Result<Self, String> {
        let gpu_res = GpuResource::new(window).await?;
        let connector = GpuConnector::new(&gpu_res);

        let shader_path = format!("{}/shaders", env!("CARGO_MANIFEST_DIR"));
        println!("Watching shaders at: {}", shader_path);

        // r theta phi
        let camera_spherical_coord = [5.19, 3.1 / 4.0, 3.1 / 4.0];
        let camera_pointing_at = [0.0, 0.0, 0.0];

        let shader_watcher = ShaderWatcher::new(shader_path);

        Ok(Self {
            gpu_res,
            connector,
            camera_spherical_coord,
            camera_pointing_at,
            shader_watcher,
        })
    }

    fn converter_coord_for_gpu(&self) {
        let r = self.camera_spherical_coord[0];
        let theta = self.camera_spherical_coord[1];
        let phi = self.camera_spherical_coord[2];

        // (r, theta, phi) -> (x, y, z)
        // x = r * sin(theta) * cos(phi)
        // y = r * sin(theta) * sin(phi)
        // z = r * cos(theta)

        let x = r * phi.sin() * theta.cos() + self.camera_pointing_at[0];
        let y = r * phi.cos() + self.camera_pointing_at[1];
        let z = r * phi.sin() * theta.sin() + self.camera_pointing_at[2];

        self.connector
            .update_camera_pos(&self.gpu_res, [x, y, z], self.camera_pointing_at);
    }

    pub fn update_camera_input_keyboard(&mut self, key: KeyCode) {
        let sensitivity = 0.05;

        match key {
            KeyCode::KeyA => self.camera_spherical_coord[1] += sensitivity,
            KeyCode::KeyD => self.camera_spherical_coord[1] -= sensitivity,
            KeyCode::KeyW => {
                self.camera_spherical_coord[2] =
                    (self.camera_spherical_coord[2] - sensitivity).clamp(0.01, 3.1) // needed against gimbal lock
            }
            KeyCode::KeyS => {
                self.camera_spherical_coord[2] =
                    (self.camera_spherical_coord[2] + sensitivity).clamp(0.01, 3.1) // needed against gimbal lock
            }
            KeyCode::ArrowRight => self.camera_pointing_at[0] += sensitivity,
            KeyCode::ArrowLeft => self.camera_pointing_at[0] -= sensitivity,
            KeyCode::ArrowUp => self.camera_pointing_at[1] += sensitivity,
            KeyCode::ArrowDown => self.camera_pointing_at[1] -= sensitivity,
            KeyCode::KeyO => self.camera_pointing_at = [0.0, 0.0, 0.0],
            _ => return,
        }

        self.converter_coord_for_gpu();
    }

    pub fn update_camera_input_mouse(&mut self, delta: MouseScrollDelta) {
        let scroll_d = match delta {
            MouseScrollDelta::LineDelta(_, y) => y * -0.05,
            MouseScrollDelta::PixelDelta(pos) => pos.y as f32 * -0.05,
        };

        self.camera_spherical_coord[0] += scroll_d;
        self.converter_coord_for_gpu();
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
