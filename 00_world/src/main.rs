#![deny(clippy::all)]
#![forbid(unsafe_code)]

use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
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
        self.renderer = Some(pollster::block_on(Renderer::new(window)).unwrap());

        // --------------------------------------------------------------------------- init other fields
        self.time = 0.0;
        self.input = 0.0;
    }

    // ------------------------------------------------------------------------------- handle window events
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let renderer = match &mut self.renderer {
            Some(canvas) => canvas,
            None => return,
        };

        match event {
            // ----------------------------------------------------------------------- close window
            WindowEvent::CloseRequested => {
                println!("close requested; stopping");
                event_loop.exit();
            }

            // ----------------------------------------------------------------------- resize
            WindowEvent::Resized(size) => renderer.resize(size.width, size.height),

            // ----------------------------------------------------------------------- redraw
            WindowEvent::RedrawRequested => match renderer.render() {
                Ok(_) => {}
                Err(e) => {
                    log::error!("{e}");
                    event_loop.exit();
                }
            },

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

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let event_loop = EventLoop::new()?;

    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();

    Ok(())
}
