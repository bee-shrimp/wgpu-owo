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
use world::{Pos, World};

mod input;
use input::InputHandler;

// ------------------------------------------------------------------- consts

const EPSILON: f32 = 0.001;

// ------------------------------------------------------------------- App struct

#[derive(Default)]
struct App {
    renderer: Option<Renderer>,
    world: World,
    input: InputHandler,
    last_frame_time: Option<Instant>,
    last_draw_data: Pos,
    need_redraw: bool,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // ----------------------------------------------------------- create window object

        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .expect("failed to create window"),
        );

        // ----------------------------------------------------------- create renderer

        self.renderer = Some(
            pollster::block_on(Renderer::new(window, event_loop))
                .expect("failed to create renderer"),
        );

        // ----------------------------------------------------------- init other fields

        self.world = World::new();
        self.input = InputHandler::new();

        self.last_frame_time = Some(Instant::now());

        self.last_draw_data = Pos::default();

        self.renderer.as_mut().unwrap().update(Pos::default());

        self.need_redraw = true;
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
            WindowEvent::RedrawRequested => {
                if self.need_redraw {
                    match renderer.render() {
                        Ok(_) => {}
                        Err(e) => {
                            log::error!("{e}");
                            event_loop.exit();
                        }
                    }
                }
            }

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
            } => match state {
                ElementState::Pressed => {
                    self.input.insert(code);
                }
                ElementState::Released => {
                    self.input.remove(&code);
                }
            },

            _ => (),
        }
    }

    // --------------------------------------------------------------- things to do after everyting else

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        //
        // ----------------------------------------------------------- dt

        let now = Instant::now();
        let dt = if let Some(last) = self.last_frame_time {
            now.duration_since(last).as_secs_f32().min(0.1)
        } else {
            0.1
        };

        self.last_frame_time = Some(now);

        // ----------------------------------------------------------- return if paused or no input

        let keys = self.input.get_pressed_keys();

        if keys.contains(&KeyCode::KeyQ) {
            event_loop.exit();
        }

        if keys.is_empty() {
            self.need_redraw = false;
            return;
        }

        if !self.world.is_running() && !keys.contains(&KeyCode::Space) {
            return;
        }

        if keys.contains(&KeyCode::Space) {
            self.world = self.world.toggle_running();
            return;
        }

        if !self.world.is_running() {
            return;
        }

        // ----------------------------------------------------------- update

        let directions = self.input.get_directions();
        self.world.update(dt, directions);

        let rect_pos = self.world.get_rect_pos();

        if has_moved(&self.last_draw_data, &rect_pos) {
            self.need_redraw = true;
            self.last_draw_data = rect_pos;
            let renderer = self.renderer.as_mut().unwrap();
            renderer.update(rect_pos);
            renderer.get_window().request_redraw()
        } else {
            self.need_redraw = false;
        }
    }
}

// ------------------------------------------------------------------- check if redraw is needed

fn has_moved(last_draw_data: &Pos, pos: &Pos) -> bool {
    let x_dist = last_draw_data.x - pos.x;
    let y_dist = last_draw_data.y - pos.y;

    x_dist.abs() >= EPSILON || y_dist.abs() >= EPSILON
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
