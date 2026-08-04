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

mod world;
use world::{Direction, Rect};

#[derive(Default)]
struct App {
    renderer: Option<Renderer>,
    key_table: Box<[bool]>,
    rect: Option<Rect>,
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
        self.key_table = vec![false; 255].into_boxed_slice(); //Before event loop
        self.rect = Some(Rect::new());
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
                if let PhysicalKey::Code(code) = event.physical_key {
                    self.key_table[code as usize] = event.state.is_pressed();
                }
            }

            _ => (),
        }
    }

    // ------------------------------------------------------------------------------- things to do after everyting else
    #[allow(unused_variables)]
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let renderer = self.renderer.as_mut().unwrap();
        let rect = self.rect.as_mut().unwrap();
        let mut direction_x: Direction = Direction::Still;
        let mut direction_y: Direction = Direction::Still;

        if self.key_table[KeyCode::ArrowLeft as usize] {
            direction_x = Direction::Left
        }

        if self.key_table[KeyCode::ArrowRight as usize] {
            direction_x = Direction::Right
        }

        if self.key_table[KeyCode::ArrowUp as usize] {
            direction_y = Direction::Up
        }

        if self.key_table[KeyCode::ArrowDown as usize] {
            direction_y = Direction::Down
        }

        let direction = (direction_x, direction_y);
        let rect_pos = rect.update(direction);
        renderer.update(rect_pos);
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
