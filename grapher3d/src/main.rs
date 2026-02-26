use std::sync::Arc;

use winit::{
    application::ApplicationHandler, dpi::LogicalSize, event::WindowEvent, event_loop::EventLoop,
    window::Window,
};

use crate::state::State;

mod gpu_connector;
mod gpu_resource;
mod state;

fn main() {
    let event_loop = EventLoop::new().expect("Failed to create Event Loop");
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

    let mut app = App::default();
    let _ = event_loop.run_app(&mut app);
}

#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    state: Option<State>,
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
                println!("Keyboard Input: {:?}", event);
            }
            WindowEvent::RedrawRequested => {
                if let Some(st) = &mut self.state {
                    let _ = st.render();
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
