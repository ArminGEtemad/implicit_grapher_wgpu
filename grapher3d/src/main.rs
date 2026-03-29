use std::{
    sync::{
        Arc,
        mpsc::{self, Receiver},
    },
    thread,
};

use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, WindowEvent},
    event_loop::EventLoop,
    keyboard::PhysicalKey,
    window::Window,
};

use crate::state::State;

mod gpu_connector;
mod gpu_resource;
mod parser;
mod shader_watcher;
mod state;

fn main() {
    // waiting for the new equation on terminal
    let (tx, rx) = mpsc::channel::<String>();
    thread::spawn(move || {
        loop {
            let mut input = String::new();
            if std::io::stdin().read_line(&mut input).is_ok() {
                let trimmed = input.trim().to_string();
                if !trimmed.is_empty() {
                    tx.send(trimmed).unwrap();
                }
            }
        }
    });
    let event_loop = EventLoop::new().expect("Failed to create Event Loop");
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

    let mut app = App::new(rx);
    let _ = event_loop.run_app(&mut app);
}

struct App {
    window: Option<Arc<Window>>,
    state: Option<State>,
    rx: Receiver<String>,
}

impl App {
    fn new(rx: Receiver<String>) -> Self {
        Self {
            window: None,
            state: None,
            rx,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let attrs = Window::default_attributes()
            .with_title("Implicit Grapher")
            .with_inner_size(LogicalSize::new(1200.0_f64, 800.0_f64));
        let window = Arc::new(
            event_loop
                .create_window(attrs)
                .expect("Failed to create a window!"),
        );

        let state = pollster::block_on(State::new(window.clone())).expect("wgpu init failed");
        self.window = Some(window);
        self.state = Some(state);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("Closing window requested!");
                event_loop.exit()
            }
            WindowEvent::Resized(size) => {
                if let Some(st) = &mut self.state {
                    st.resize(size);
                    println!("Resizing: {:?}", size);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let ElementState::Pressed = event.state {
                    if let Some(st) = &mut self.state {
                        if let PhysicalKey::Code(code) = event.physical_key {
                            st.update_camera_input_keyboard(code);
                        }
                    }
                }
            }

            WindowEvent::MouseWheel { delta, .. } => {
                if let Some(st) = &mut self.state {
                    st.update_camera_input_mouse(delta);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(st) = &mut self.state {
                    let _ = st.render(&self.rx);
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }
}
