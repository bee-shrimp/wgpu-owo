#![deny(clippy::all)]
#![forbid(unsafe_code)]

// ------------------------------------------------------------------- imports

use std::sync::Arc;
use std::time::Instant;

use anyhow::Context;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

mod renderer;
use renderer::Renderer;

mod world;
use world::World;

mod input;
use input::InputHandler;

// ------------------------------------------------------------------- App struct

#[derive(Default)]
struct App {
    renderer: Option<Renderer>,
    world: World,
    input: InputHandler,
    last_frame_time: Option<Instant>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // ----------------------------------------------------------- create window object

        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .expect("failed to create window"),
        );

        // ----------------------------------------------------------- create world

        self.world = World::default();
        let instances = self.world.get_instances();

        // ----------------------------------------------------------- create renderer

        self.renderer = Some(
            pollster::block_on(Renderer::new(window, event_loop, &instances))
                .expect("failed to create renderer"),
        );

        // ----------------------------------------------------------- init other fields

        self.input = InputHandler::new();

        self.last_frame_time = Some(Instant::now());

        self.renderer.as_mut().unwrap().update(&instances);
    }

    // --------------------------------------------------------------- handle window events

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let renderer = match &mut self.renderer {
            Some(canvas) => canvas,
            None => return,
        };

        match event {
            // ------------------------------------------------------- close window
            //
            WindowEvent::CloseRequested => {
                println!("close requested; stopping");
                event_loop.exit();
            }

            // ------------------------------------------------------- resize
            //
            WindowEvent::Resized(size) => renderer.resize(size.width, size.height),

            // ------------------------------------------------------- redraw
            //
            WindowEvent::RedrawRequested => match renderer.render() {
                Ok(_) => {}
                Err(e) => {
                    log::error!("{e}");
                    event_loop.exit();
                }
            },

            // ------------------------------------------------------- handle key inputs
            //
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state,
                        ..
                    },
                ..
            } => {
                match state {
                    ElementState::Pressed => self.input.insert(code),
                    ElementState::Released => self.input.remove(code),
                };
            }

            _ => (),
        }
    }

    // --------------------------------------------------------------- things to do after everyting else

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        //
        // ----------------------------------------------------------- calculate dt

        let now = Instant::now();
        let dt = if let Some(last) = self.last_frame_time {
            now.duration_since(last).as_secs_f32().min(0.1)
        } else {
            0.016 // fps60
        };

        self.last_frame_time = Some(now);

        // ----------------------------------------------------------- quit if key q is pressed

        if self.input.has(KeyCode::KeyQ) {
            event_loop.exit();
        }

        // ----------------------------------------------------------- return if paused and space is not pressed

        if !self.world.is_running() && !self.input.has(KeyCode::Space) {
            return;
        }

        // ----------------------------------------------------------- pause/resume if space is pressed

        if self.input.has(KeyCode::Space) {
            self.world.toggle_running();
            return;
        }

        // ----------------------------------------------------------- update

        self.world.update(dt);

        let renderer = self.renderer.as_mut().unwrap();
        let instances = self.world.get_instances();
        renderer.update(&instances);
        renderer.get_window().request_redraw()
    }
}

// ------------------------------------------------------------------- main

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let event_loop = EventLoop::new().context("failed to create event loop")?;

    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::default();
    event_loop.run_app(&mut app).context("failed to run app")?;

    Ok(())
}
