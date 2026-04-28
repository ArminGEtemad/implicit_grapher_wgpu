use std::{
    collections::HashSet,
    sync::{Arc, mpsc},
    time,
};
use wgpu::SurfaceError;
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, MouseScrollDelta},
    keyboard::KeyCode,
    window::Window,
};

use crate::{
    gpu_connector::GpuConnector,
    gpu_resource::{FrameContext, GpuResource},
    shader_watcher::ShaderWatcher,
};

const ORIGIN: [f32; 3] = [0.0, 0.0, 0.0];
const PLOT_LIMIT_SAFTY: f32 = 0.1;

pub struct State {
    gpu_res: GpuResource,
    connector: GpuConnector,
    pressed_keys: HashSet<KeyCode>,
    camera_spherical_coord: [f32; 3], // r, thera, phi
    camera_pointing_at: [f32; 3],     // x, y, z
    plot_limits_min: [f32; 3],        // x, y, z
    plot_limits_max: [f32; 3],        // x, y, z
    start_time: time::Instant,
    frame_count: u32,
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

        let start_time = time::Instant::now();
        let frame_count = 0_u32;

        let shader_watcher = ShaderWatcher::new(shader_path);
        let current_formula = implicit_fomula.to_string();
        println!("Currently Showing: {}", current_formula);

        let mut state = Self {
            gpu_res,
            connector,
            pressed_keys: HashSet::new(),
            camera_spherical_coord,
            camera_pointing_at,
            plot_limits_min,
            plot_limits_max,
            start_time,
            frame_count,
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
        println!("Awaiting new implicit equation!");
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

    pub fn handle_key_event(&mut self, key: KeyCode, element_state: ElementState) {
        match element_state {
            ElementState::Pressed => {
                self.pressed_keys.insert(key);
                self.update_camera_input_keyboard(key);
            }
            ElementState::Released => {
                self.pressed_keys.remove(&key);
            }
        }
    }

    fn update_camera_input_keyboard(&mut self, key: KeyCode) {
        let shift_pressed = self.pressed_keys.contains(&KeyCode::ShiftLeft)
            || self.pressed_keys.contains(&KeyCode::ShiftRight);
        let ctrl_pressed = self.pressed_keys.contains(&KeyCode::ControlLeft)
            || self.pressed_keys.contains(&KeyCode::ControlRight);
        let x_pressed = self.pressed_keys.contains(&KeyCode::KeyX);
        let y_pressed = self.pressed_keys.contains(&KeyCode::KeyY);
        let z_pressed = self.pressed_keys.contains(&KeyCode::KeyZ);

        let sensitivity = if shift_pressed { 0.1 } else { 0.05 };

        match key {
            // moving along the axes
            KeyCode::ArrowRight => {
                if x_pressed {
                    self.camera_pointing_at[0] += sensitivity;
                } else if z_pressed {
                    self.camera_pointing_at[1] += sensitivity;
                } else if y_pressed {
                    self.camera_pointing_at[2] += sensitivity;
                } else {
                    return;
                }
            }

            KeyCode::ArrowLeft => {
                if x_pressed {
                    self.camera_pointing_at[0] -= sensitivity;
                } else if z_pressed {
                    self.camera_pointing_at[1] -= sensitivity;
                } else if y_pressed {
                    self.camera_pointing_at[2] -= sensitivity;
                } else {
                    return;
                }
            }

            KeyCode::ArrowUp => {
                if ctrl_pressed {
                    if x_pressed {
                        self.plot_limits_max[0] += sensitivity;
                    } else if z_pressed {
                        self.plot_limits_max[1] += sensitivity;
                    } else if y_pressed {
                        self.plot_limits_max[2] += sensitivity;
                    } else {
                        return;
                    }
                } else {
                    if x_pressed {
                        self.plot_limits_min[0] = (self.plot_limits_min[0] + sensitivity)
                            .min(self.plot_limits_max[0] - PLOT_LIMIT_SAFTY);
                    } else if z_pressed {
                        self.plot_limits_min[1] = (self.plot_limits_min[1] + sensitivity)
                            .min(self.plot_limits_max[1] - PLOT_LIMIT_SAFTY);
                    } else if y_pressed {
                        self.plot_limits_min[2] = (self.plot_limits_min[2] + sensitivity)
                            .min(self.plot_limits_max[2] - PLOT_LIMIT_SAFTY);
                    } else {
                        return;
                    }
                }
            }

            KeyCode::ArrowDown => {
                if ctrl_pressed {
                    if x_pressed {
                        self.plot_limits_max[0] = (self.plot_limits_max[0] - sensitivity)
                            .max(self.plot_limits_min[0] + PLOT_LIMIT_SAFTY);
                    } else if z_pressed {
                        self.plot_limits_max[1] = (self.plot_limits_max[1] - sensitivity)
                            .max(self.plot_limits_min[1] + PLOT_LIMIT_SAFTY);
                    } else if y_pressed {
                        self.plot_limits_max[2] = (self.plot_limits_max[2] - sensitivity)
                            .max(self.plot_limits_min[2] + PLOT_LIMIT_SAFTY);
                    } else {
                        return;
                    }
                } else {
                    if x_pressed {
                        self.plot_limits_min[0] -= sensitivity;
                    } else if z_pressed {
                        self.plot_limits_min[1] -= sensitivity;
                    } else if y_pressed {
                        self.plot_limits_min[2] -= sensitivity;
                    } else {
                        return;
                    }
                }
            }

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
            KeyCode::KeyO => self.camera_pointing_at = ORIGIN,

            _ => return,
        }

        self.converter_coord_for_gpu();
        self.plot_limits_config();
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

    pub fn render(&mut self, rx: &mpsc::Receiver<String>) -> Result<(), SurfaceError> {
        while let Ok(input_formula) = rx.try_recv() {
            self.update_formula(&input_formula);
        }

        while let Ok(path) = self.shader_watcher.reciever_x.try_recv() {
            println!("Shader changed: {:?}", path);
            self.connector
                .rebuild_pipeline(&self.gpu_res, &self.current_formula);
        }

        let elapsed = self.start_time.elapsed().as_secs_f32();
        self.frame_count += 1;

        self.connector
            .update_scene(&self.gpu_res, elapsed, self.frame_count);

        let mut frame: FrameContext = self.gpu_res.begin_frame()?;
        self.connector.render_pass(&mut frame);
        self.gpu_res.submit_frame(frame);
        Ok(())
    }
}
