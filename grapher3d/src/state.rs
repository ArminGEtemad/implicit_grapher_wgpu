use std::sync::Arc;
use wgpu::SurfaceError;
use winit::{dpi::PhysicalSize, event::MouseScrollDelta, keyboard::KeyCode, window::Window};

use crate::{
    gpu_connector::GpuConnector,
    gpu_resource::{FrameContext, GpuResource},
    shader_watcher::ShaderWatcher,
};

const ORIGIN: [f32; 3] = [0.0, 0.0, 0.0];

pub struct State {
    gpu_res: GpuResource,
    connector: GpuConnector,
    camera_spherical_coord: [f32; 3], // r, thera, phi
    camera_pointing_at: [f32; 3],     // x, y, z
    plot_limits_min: [f32; 3],        // x, y, z
    plot_limits_max: [f32; 3],        // x, y, z
    current_formula: String,
    shader_watcher: ShaderWatcher,
}

impl State {
    pub async fn new(window: Arc<Window>) -> Result<Self, String> {
        let gpu_res = GpuResource::new(window).await?;
        // implicit fomula
        let implicit_fomula = "y - sin(x) - sin(z)";
        let connector = GpuConnector::new(&gpu_res, implicit_fomula);

        let shader_path = format!("{}/shaders", env!("CARGO_MANIFEST_DIR"));
        println!("Watching shaders at: {}", shader_path);

        // r theta phi
        let camera_spherical_coord = [17.32, 3.1 / 4.0, 3.1 / 4.0];
        let camera_pointing_at = ORIGIN;

        // plot limits
        let plot_limits_min = [-15.0, -15.0, -15.0];
        let plot_limits_max = [15.0, 15.0, 15.0];

        let shader_watcher = ShaderWatcher::new(shader_path);
        let current_formula = implicit_fomula.to_string();

        let mut state = Self {
            gpu_res,
            connector,
            camera_spherical_coord,
            camera_pointing_at,
            plot_limits_min,
            plot_limits_max,
            current_formula,
            shader_watcher,
        };

        state.plot_limits_config();
        state.converter_coord_for_gpu();

        Ok(state)
    }

    fn plot_limits_config(&mut self) {
        self.connector
            .plot_limits(&self.gpu_res, self.plot_limits_min, self.plot_limits_max);
    }

    fn update_formula(&mut self, new_equation: &str) {
        println!("Injecting the new equation: {}", new_equation);
        self.current_formula = new_equation.to_string();
        self.connector
            .rebuild_pipeline(&self.gpu_res, &self.current_formula);
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
            // TODO: Just to make sure the update formula works. needs a new place later ---------
            KeyCode::KeyG => self.update_formula("x^2 + y^2 + z^2 - 4.0"),
            KeyCode::KeyH => self
                .update_formula("(x^2 + y^2 + z^2 + 0.5^2 - 0.2^2)^2 - 4.0 * 0.5^2 * (x^2 + y^2)"),
            // -------------------------------------------------------------------------------------
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
            KeyCode::KeyO => self.camera_pointing_at = ORIGIN,
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
        self.plot_limits_config();
        while let Ok(path) = self.shader_watcher.reciever_x.try_recv() {
            println!("Shader changed: {:?}", path);
            self.connector
                .rebuild_pipeline(&self.gpu_res, &self.current_formula);
        }

        let mut frame: FrameContext = self.gpu_res.begin_frame()?;
        self.connector.render_pass(&mut frame);
        self.gpu_res.submit_frame(frame);
        Ok(())
    }
}
