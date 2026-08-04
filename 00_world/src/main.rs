#![deny(clippy::all)]
#![forbid(unsafe_code)]

use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, OwnedDisplayHandle},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

mod renderer;
use renderer::Renderer;

#[derive(Default)]
struct App {
    renderer: Option<Renderer>,
    time: f32,
    input: f32,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // --------------------------------------------------------------------------- create window object
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        // --------------------------------------------------------------------------- create renderer
        let renderer = pollster::block_on(Renderer::new(window.clone()));
        self.renderer = Some(renderer);
        self.time = 0.0;
        self.input = 0.0;

        // --------------------------------------------------------------------------- redraw
        window.request_redraw();
    }

    // ------------------------------------------------------------------------------- handle window events
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let renderer = self.renderer.as_mut().unwrap();

        match event {
            // ----------------------------------------------------------------------- close window
            WindowEvent::CloseRequested => {
                println!("close requested; stopping");
                event_loop.exit();
            }
            // ----------------------------------------------------------------------- redraw
            WindowEvent::RedrawRequested => {
                renderer.render();
            }

            // ----------------------------------------------------------------------- resize
            WindowEvent::Resized(size) => {
                // this event is always followed up by redraw request.
                renderer.resize(size);
            }
            // ----------------------------------------------------------------------- handle key inputs
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(code) = event.physical_key
                    && event.state.is_pressed()
                {
                    match code {
                        KeyCode::ArrowUp => {
                            self.input += 0.1;
                        }
                        KeyCode::ArrowDown => {
                            self.input -= 0.1;
                        }
                        _ => (),
                    }
                }
            }

            _ => (),
        }
    }

    // ------------------------------------------------------------------------------- things to do after everyting else
    #[allow(unused_variables)]
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let renderer = self.renderer.as_mut().unwrap();

        self.time += 0.1;
        renderer.update(self.time, self.input);

        renderer.get_window().request_redraw()
    }
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}
